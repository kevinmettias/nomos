//! Saying what the ledger answered, and which exit code that is.

use nomos_ledger::{
    AddRefusal, Claim_Refusal, ClaimRefusal, FileLedger, FinishRefusal, ItemId, ItemState,
    LedgerDocument, LedgerError, LedgerItem, SCHEMA_VERSION, Territory, Validate,
};
use nomos_platform::Timestamp;
use nomos_platform_std::{FileLock, StdFileSystem, SystemClock};

use super::ExitCode;

/// The label this item lists under, or nothing when the filter excludes it.
pub(super) fn Listed_As(
    document: &LedgerDocument,
    item: &LedgerItem,
    state: Option<&str>,
    now: Timestamp,
) -> Option<&'static str>
{
    let label = Listing_Label(document, item, now);
    if state.is_some_and(|wanted| return !label.eq_ignore_ascii_case(wanted))
    {
        return None;
    }

    return Some(label);
}

/// What to say when the listing printed nothing.
///
/// An empty result and a filter that matched nothing look identical otherwise, and the
/// user's next action differs.
pub(super) fn Nothing_Listed(state: Option<&str>, output: &mut impl std::io::Write)
{
    let _ = match state
    {
        Some(wanted) => writeln!(output, "no items are {wanted}"),
        None => writeln!(output, "the ledger has no items"),
    };
}

/// One item's line: its identifier, what it may be called now, its holder, and its title.
pub(super) fn Print_Listing(item: &LedgerItem, label: &str, output: &mut impl std::io::Write)
{
    let holder = item
        .claim
        .as_ref()
        .map_or_else(String::new, |claim| return format!("  [{}]", claim.holder));

    let _ = writeln!(output, "{:<13} {:<9}{holder}  {}", item.id, label, item.title);
}

/// The live claim, if there is one.
///
/// A lapsed claim is still shown, and still says it lapsed. It stops excluding without being
/// removed, so a reader who is not told would take it for a live one.
pub(super) fn Print_Claim(found: &LedgerItem, now: Timestamp, output: &mut impl std::io::Write)
{
    let Some(claim) = &found.claim
    else
    {
        return;
    };

    let lapsed = if claim.Has_Lapsed(now)
    {
        " (lapsed)"
    }
    else
    {
        ""
    };

    let _ = writeln!(
        output,
        "held by {} since unix {} until unix {}{lapsed}",
        claim.holder,
        claim.acquired_at.Unix_Seconds(),
        claim.lease_expires_at.Unix_Seconds()
    );
}

/// What has happened to the item: takeovers, abandonments, and the verification that ended
/// it.
///
/// Reported, not merely stored. `OD-LEDGER-006`'s rule and `OD-LEDGER-012`'s reason for
/// obeying it here: a record no surface reports is one only somebody willing to read the
/// JSON can find, which is most of the way back to not keeping it. Each displacement line
/// names the holder a takeover displaced and the window they held — who displaced them is
/// the next line's holder, or the live claim above.
pub(super) fn Print_History(found: &LedgerItem, output: &mut impl std::io::Write)
{
    Print_Displacements(found, output);
    Print_Abandonments(found, output);
    Print_Verification(found, output);
}

/// Every holder a takeover displaced, and the window they held.
fn Print_Displacements(found: &LedgerItem, output: &mut impl std::io::Write)
{
    for displaced in &found.displaced
    {
        let _ = writeln!(
            output,
            "taken over from {}, who held it from unix {} until unix {}",
            displaced.holder,
            displaced.acquired_at.Unix_Seconds(),
            displaced.lease_expires_at.Unix_Seconds()
        );
    }
}

/// Every claim that was given up without finishing, and the reason given.
fn Print_Abandonments(found: &LedgerItem, output: &mut impl std::io::Write)
{
    for abandonment in &found.abandoned
    {
        let _ = writeln!(
            output,
            "abandoned by {} at unix {}: {}",
            abandonment.holder,
            abandonment.abandoned_at.Unix_Seconds(),
            abandonment.reason
        );
    }
}

/// The predicate that ended the item, if one has.
fn Print_Verification(found: &LedgerItem, output: &mut impl std::io::Write)
{
    let Some(record) = &found.verified
    else
    {
        return;
    };

    let _ = writeln!(
        output,
        "verified by `{}` at unix {} with exit {}",
        record.argv.join(" "),
        record.verified_at.Unix_Seconds(),
        record.exit_code
    );
}

/// What the success line says about a declared amendment, and nothing when there is none.
///
/// Said on the way out because it is the one thing about the item that the board does not
/// keep. Territory is on the item and can be read back with `show`; the declaration decided
/// this add and is then gone, so an author who mis-declared has this line and no other
/// chance to notice.
pub(super) fn Amendment_Note(amending: &Territory) -> String
{
    if amending.paths.is_empty()
    {
        return String::new();
    }

    return format!(", amending {}", amending.paths.join(", "));
}

/// The exit code an `add` refusal reports.
///
/// Each arm keeps the code this command already gave it. Routing the write through the
/// store changed which type carries the refusal out and must not change what an agent
/// branching on the number concludes: a taken identifier is the caller's to resolve by
/// choosing another, an item that reserves nothing is the caller's to correct, and only a
/// ledger that cannot be read or written at all is the one an agent stops and fetches a
/// person for.
pub(super) const fn Code_For_Refusal(refusal: &AddRefusal) -> ExitCode
{
    return match refusal
    {
        AddRefusal::AlreadyPresent { .. } => ExitCode::Conflict,
        // The same code as a taken item identifier, and for the same reason: an identifier
        // somebody else holds, which the author resolves by choosing another.
        // `ExitCode::Usage` was the other candidate and is wrong — that is the parser's code
        // for a malformed invocation, and an agent that saw it would go and inspect its own
        // argument syntax, which is not the fix. The refusal text is what tells the two
        // record cases apart; the code tells an agent what kind of thing happened, and this
        // is the kind that already had one.
        AddRefusal::RecordPublished { .. } | AddRefusal::RecordReserved { .. } =>
        {
            ExitCode::Conflict
        }
        // A declared amendment of nothing joins the invalid item rather than the two record
        // conflicts above, and the difference is what an agent does next. Nothing is contended
        // here — the identifier is free — so `Conflict` would send it looking for a holder
        // that does not exist. What is wrong is the item's own declaration, which is the
        // caller's to correct, and that is already what this code means.
        AddRefusal::WouldBeInvalid { .. } | AddRefusal::AmendmentNotPublished { .. } =>
        {
            ExitCode::ValidationError
        }
        AddRefusal::LedgerUnusable { .. } => ExitCode::StoreError,
    };
}

pub(super) fn Report_Finish(
    result: Result<nomos_ledger::VerificationRecord, FinishRefusal>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    return match result
    {
        Ok(record) =>
        {
            let _ = writeln!(
                output,
                "verified by `{}` at unix {}",
                record.argv.join(" "),
                record.verified_at.Unix_Seconds()
            );
            ExitCode::Ok
        }
        Err(refusal) =>
        {
            let _ = writeln!(output, "not finished: {}", refusal.Describe());

            // A failing predicate is a validation error: the work was judged and found
            // incomplete. Everything else prevented the judgment, and reporting that as
            // the same thing would send an author to fix code that may be fine.
            if refusal.Judged_The_Work()
            {
                ExitCode::ValidationError
            }
            else
            {
                ExitCode::Conflict
            }
        }
    };
}

/// What stands between `item` and an agent that would take it, if anything.
///
/// The reason this is a function is the reason [`Claim_Refusal`] is one: two implementations
/// of a rule is how they come to disagree — `OD-LEDGER-005`. What it adds over
/// `Claim_Refusal` is one filter, and `audit` is what needs it.
///
/// An item that is not `Ready` returns `None` rather than its refusal. `Claim_Refusal`
/// would answer `NotClaimable` for every `Done` and `Declined` item on the board, which is
/// true and useless: nobody is queued behind finished work, and reporting it buries the
/// handful of refusals somebody could actually act on.
///
/// That filter is why [`Listing_Label`] reads `Claim_Refusal` directly and not this. A
/// listing has to name the state of every item including the ones nothing is queued behind,
/// and the answer it most needs — `lapsed` — is one this function is deliberately silent
/// about. Both still label from [`Refusal_Label`], so the two reports cannot disagree about
/// the word for a refusal they both see.
pub(super) fn Blocking_Refusal(
    document: &LedgerDocument,
    item: &LedgerItem,
    now: Timestamp,
) -> Option<ClaimRefusal>
{
    if !matches!(item.state, ItemState::Ready)
    {
        return None;
    }

    return Claim_Refusal(document, &item.id, now);
}

/// The word for a refusal.
///
/// Shared by the listing and the audit for the same reason the refusal itself is: one item
/// must not be `held` in one report and something else in the other.
const fn Refusal_Label(refusal: &ClaimRefusal) -> &'static str
{
    return match refusal
    {
        // Retryable and not the reader's problem to solve: something else has to finish or
        // lapse first. `waiting` rather than `blocked`, because `blocked` is already a state
        // an author sets by hand and conflating them would lose that distinction.
        ClaimRefusal::DependencyUnmet { .. } => "waiting",
        ClaimRefusal::HeldBy { .. } => "held",
        // The one word here that names an operation rather than a wait. An item whose holder
        // is gone is not queued behind anybody and is not a dead end either: it is takeable by
        // whoever says so, with `nomos work takeover`.
        ClaimRefusal::Lapsed { .. } => "lapsed",
        // Not retryable: somebody has to close a modelling gap. Reporting it as `ready`
        // would send an agent to discover that by being refused.
        _ => "snagged",
    };
}

/// What to call an item in a listing.
///
/// For everything except a `Ready` item this is just the state. `Ready` is the word that
/// was lying: it means "nobody has taken this", and a reader takes it to mean "I can take
/// this". Those came apart whenever a dependency was unfinished or somebody held
/// overlapping ground — on 2026-08-09 the column said `ready` for eight items that a single
/// held claim refused, three separate times.
///
/// `claimed` is the second word that lied, for the same reason `ready` was the first. An item
/// whose lease ran out four hours ago reads as work in progress, and it is work whose holder is
/// gone. It excludes nobody — `OD-LEDGER-009` — and since `OD-LEDGER-012` it is takeable, by
/// `nomos work takeover` rather than by `claim`. So `lapsed` now says an operation is available
/// rather than that an editor is.
///
/// The answer comes from [`Claim_Refusal`], the function `claim` itself refuses with. That is
/// the point rather than an implementation detail: a listing computing its own idea of
/// claimability would be a second guard for one rule, and the two would eventually disagree
/// about whether an agent may proceed.
///
/// This function *was* that second guard. It decided `lapsed` itself, in a branch above the
/// refusal it now reads, and after `OD-LEDGER-012` that branch was a second implementation of
/// the predicate deciding whether `takeover` succeeds — the exact arrangement the paragraph
/// above forbids, in the function whose doc comment forbids it. Every label is unchanged for
/// every input; what changed is that one function decides.
pub(super) fn Listing_Label(document: &LedgerDocument, item: &LedgerItem, now: Timestamp) -> &'static str
{
    return match Claim_Refusal(document, &item.id, now)
    {
        // Nothing refuses it, or nothing is meant to: the state word is the honest answer in
        // both cases, and for a `Ready` item that word is `ready`. `NotClaimable` is the arm
        // every `Blocked`, `Done` and `Declined` item arrives on, and its own word is better
        // than any refusal's — nobody is queued behind finished work.
        None | Some(ClaimRefusal::NotClaimable { .. }) => State_Label(&item.state),
        Some(refusal) => Refusal_Label(&refusal),
    };
}

fn State_Label(state: &ItemState) -> &'static str
{
    return match state
    {
        ItemState::Ready => "ready",
        ItemState::Claimed => "claimed",
        ItemState::Blocked => "blocked",
        ItemState::Done => "done",
        ItemState::Declined { .. } => "declined",
    };
}

pub(super) fn Report_Claim(
    result: Result<nomos_ledger::Reservation, ClaimRefusal>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    return match result
    {
        Ok(reservation) =>
        {
            let _ = writeln!(
                output,
                "{} held by {} until unix {}",
                reservation.item,
                reservation.holder,
                reservation.expires_at.Unix_Seconds()
            );
            ExitCode::Ok
        }
        Err(refusal) =>
        {
            let _ = writeln!(output, "refused: {}", refusal.Describe());
            Code_For(&refusal)
        }
    };
}

/// Reports a decline, naming the item on both paths.
///
/// The success line says `declined` and not `released`: an agent that reads `released` after
/// running `decline` has been told the item is back on the board, which is the opposite of
/// what happened and the exact confusion this verb exists to end.
///
/// The refusal names the item first and then prints [`ClaimRefusal::Describe`] beneath it,
/// which is the composition `OD-LEDGER-014` phrased those sentences for.
pub(super) fn Report_Decline(
    item: &ItemId,
    result: Result<(), ClaimRefusal>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    return match result
    {
        Ok(()) =>
        {
            let _ = writeln!(output, "{item} declined");
            ExitCode::Ok
        }
        Err(refusal) =>
        {
            let _ = writeln!(output, "refused: {}", refusal.Describe());
            Code_For(&refusal)
        }
    };
}

pub(super) fn Report_Release(result: Result<(), ClaimRefusal>, output: &mut impl std::io::Write)
-> ExitCode
{
    return match result
    {
        Ok(()) =>
        {
            let _ = writeln!(output, "released");
            ExitCode::Ok
        }
        Err(refusal) =>
        {
            let _ = writeln!(output, "refused: {}", refusal.Describe());
            Code_For(&refusal)
        }
    };
}

/// The exit code a refusal earns.
///
/// The distinction `3` versus `5` is the one the README says earns its own code: an agent
/// told the item is taken picks up something else, and an agent told the ledger is broken
/// stops and fetches a person. [`ClaimRefusal::LedgerUnusable`] is the second of those and
/// used to arrive as `4` — a conflict a human resolves — after arriving as "no such item",
/// which sent them to check a spelling. Three answers, one of them right.
const fn Code_For(refusal: &ClaimRefusal) -> ExitCode
{
    return match refusal
    {
        ClaimRefusal::LedgerUnusable { .. } => ExitCode::StoreError,
        other =>
        {
            if other.Is_Retryable()
            {
                ExitCode::ClaimUnavailable
            }
            else
            {
                ExitCode::Conflict
            }
        }
    };
}

/// Whether the ledger is valid, and whether the executable asking is current.
///
/// The second half is what makes this the one command an operator can answer "is the `nomos.exe`
/// I copied still current?" with. Sessions run a copy of the binary, because `finish` runs a
/// predicate that rebuilds the running executable, and a copy taken before a schema change was a
/// silent data-loss channel until `OD-LEDGER-008`. It is loud now — every verb exits 5 — and this
/// is where the two numbers can be read side by side without provoking a refusal first.
///
/// `Load` and [`Validate`] rather than `Validate_Current`, which discards the document and so
/// cannot report the file's own version. One read, not two, so both halves of the line describe
/// the same file.
pub(super) fn Report_Validation(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

    let violations = Validate(&document, ledger.Now());
    if !violations.is_empty()
    {
        return Report_Error(&LedgerError::Invalid { violations }, output);
    }

    let _ = writeln!(
        output,
        "ledger is valid (schema {}, and this build understands {})",
        document.schema_version, SCHEMA_VERSION
    );

    return ExitCode::Ok;
}

/// Every item somebody is waiting on, and what they are waiting for.
///
/// # Why this asks the listing's question and not its own
///
/// `audit` used to walk every item on the board and ask [`ExclusionLedger::Conflicts`] which
/// live claims overlapped its territory. That was a second implementation of the rule
/// `list` already went through, and the two disagreed the moment the board had history in
/// it. Measured on 2026-08-09: of fourteen lines, nine described `Done` items as blocked by
/// a claim they will never contend for, because a finished item still has territory and
/// `Conflicts` has no opinion about state. The board `P10-AUDIT-STATE` was written against
/// was worse — fifty-four lines, forty-four of them about work nobody can pick up.
///
/// The over-report is how it was noticed; the silence was the cost. `Conflicts` compares
/// territory and knows nothing about `depends_on`, so an item refused with
/// `ClaimRefusal::DependencyUnmet` printed nothing at all — `audit` could not say `waiting`
/// where `list` could, which is the one answer an agent looking for the next thing to do
/// most needs.
///
/// So the filter and the reason both come from [`Blocking_Refusal`], the function `list`
/// labels with and `claim` refuses with — `OD-LEDGER-005`. An item reported here is exactly
/// an item `list` calls `waiting`, `held` or `snagged`, with the same word, and there is no
/// way for the two to drift because they are one answer read twice.
pub(super) fn Print_Blocked(item: &LedgerItem, refusal: &ClaimRefusal, output: &mut impl std::io::Write)
{
    let _ = writeln!(
        output,
        "{:<13} {:<9} {}",
        item.id,
        Refusal_Label(refusal),
        // `Describe` directly, and no local rephrasing. Its held arm used to name the
        // blocker where the subject belongs, so this line phrased that one arm itself;
        // `OD-LEDGER-014` fixed the library and deleted the workaround in the same commit,
        // because the whole cost of the workaround was that two renderings of one refusal
        // outlived the reason for the second.
        refusal.Describe()
    );
}

pub(super) fn Report_Error(error: &LedgerError, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(output, "{error}");

    return match error
    {
        LedgerError::Invalid { .. } => ExitCode::ValidationError,
        // `Unrecognized` is a store error and not a conflict. The README's own criterion
        // decides it: an agent told the ledger cannot be used at all stops and fetches a
        // person, and a binary that cannot read the board is exactly that. No new code is
        // introduced, so the exit-code table does not move.
        LedgerError::Unreadable { .. }
        | LedgerError::Malformed { .. }
        | LedgerError::Unrecognized { .. }
        | LedgerError::Locked { .. } => ExitCode::StoreError,
    };
}
