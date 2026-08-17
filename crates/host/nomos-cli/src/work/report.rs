//! Saying what the ledger answered, and which exit code that is.

use nomos_ledger::{
    AddRefusal, Claim_Refusal, ClaimRefusal, FinishRefusal, ItemId, ItemState, LedgerDocument,
    LedgerError, LedgerItem, RefusalLayer, SCHEMA_VERSION, Territory,
};
use nomos_platform::Timestamp;

use super::ExitCode;

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
///
/// Matched on [`ClaimRefusal::Layer`] first and the specific variant second, rather than on
/// the variant alone. `OD-LEDGER-022` is why: the four specific words below already told a
/// plan fact (`waiting`, `stranded`) apart from a coordination fact (`held`, `lapsed`), and
/// matching flat let that distinction exist only here, in this function's arm order, where a
/// caller other than this CLI could not read it.
///
/// The two catch-alls below both say `snagged` today, which reads as one arm to
/// `clippy::match_same_arms` and is merged into one for that reason — but it is not a claim
/// that a plan fact nothing else names and a coordination fact nothing else names are the same
/// kind of thing. A caller that needs them apart has [`ClaimRefusal::Layer`] itself, which is
/// the point of routing through it here rather than matching the variant alone.
pub(super) fn Refusal_Label(refusal: &ClaimRefusal) -> &'static str
{
    return match (refusal.Layer(), refusal)
    {
        // Retryable and not the reader's problem to solve: something else has to finish or
        // lapse first. `waiting` rather than `blocked`, because `blocked` is already a state
        // an author sets by hand and conflating them would lose that distinction.
        (RefusalLayer::Readiness, ClaimRefusal::DependencyUnmet { .. }) => "waiting",
        // Not retryable, and not `waiting`: nothing finishing resolves this, because the
        // dependency it names already answered `Declined` and stays there. `OD-LEDGER-020`.
        (RefusalLayer::Readiness, ClaimRefusal::DependencyDeclined { .. }) => "stranded",
        (RefusalLayer::Dispatch, ClaimRefusal::HeldBy { .. }) => "held",
        // The one word here that names an operation rather than a wait. An item whose holder
        // is gone is not queued behind anybody and is not a dead end either: it is takeable by
        // whoever says so, with `nomos work takeover`.
        (RefusalLayer::Dispatch, ClaimRefusal::Lapsed { .. }) => "lapsed",
        // Everything left in either layer: a plan fact nothing else names (the item is not
        // claimable, or is not on the board at all) or a coordination fact nothing else names
        // (an unprovable overlap, a lease request coordination refuses, a live claim on the
        // same item, a store coordination cannot use). Neither is retryable, and the caller
        // who needs the two apart asks `Layer`, not this word.
        (_, _) => "snagged",
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
/// `result` already carries a board known to satisfy its own invariants —
/// `nomos_work_orchestration::Run` folded the violation check into the same read that used to
/// happen here, so this function only says what the two numbers mean.
pub(super) fn Report_Validation(
    result: Result<LedgerDocument, LedgerError>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let document = match result
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

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
