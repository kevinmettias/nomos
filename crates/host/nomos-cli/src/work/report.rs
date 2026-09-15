//! Saying what the ledger answered, and which exit code that is.

// file-size: allow this file pairs its production code with its own inline #[cfg(test)]
// module; check-test-coverage keys a test's companion unit off the exact file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their attribution to every function this file declares.
// responsibility: allow same reason -- the coupling that keeps this file whole is
// check-test-coverage's stem-based companion attribution, not a design choice.

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
        // Beside its two neighbours above rather than with the record conflicts, and for the
        // same reason they are: nothing is contended. The author is entitled to amend the
        // record they named — they spelled its filename wrongly — so `Conflict` would send an
        // agent looking for a holder that does not exist. The declaration is the caller's own
        // to correct, and the refusal already carries the spelling to correct it with.
        AddRefusal::AmendmentMisspelled { .. } => ExitCode::ValidationError,
        AddRefusal::LedgerUnusable { .. } => ExitCode::StoreError,
    };
}

pub(super) fn Report_Finish(
    ended: &Ended<'_>,
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
            Report_Fanout(ended, output);
            ExitCode::Ok
        }
        Err(refusal) =>
        {
            let _ = writeln!(output, "not finished: {}", refusal.Describe());

            // A failing predicate is a validation error: the work was judged and found
            // incomplete. Everything else prevented the judgment, and reporting that as
            // the same thing would send an author to fix code that may be fine.
            if refusal.Has_Judged_The_Work()
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

/// An item that was just ended, and the board it was ended on.
///
/// The two travel together because neither answers the question alone: the id says which
/// item's dependents to look for, and the board says what those dependents are now. Grouped
/// rather than passed as two parameters because both reporters need both, and `finish`'s own
/// signature is already at this crate's `parameter-count` limit.
pub(super) struct Ended<'a>
{
    pub item: &'a ItemId,
    /// `None` when the transition refused, or when the board could not be re-read after it
    /// succeeded — see `nomos_work_orchestration::WorkOutcome::Finish`'s own doc.
    pub board: Option<&'a LedgerDocument>,
    /// Read by the composition root rather than here, the same division `OD-HOST-002` draws
    /// everywhere else: a lease is `held` or `lapsed` depending on it, so a report that
    /// picked its own clock could label an item differently from the `list` beside it.
    pub now: Timestamp,
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
    ended: &Ended<'_>,
    result: Result<(), ClaimRefusal>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    return match result
    {
        Ok(()) =>
        {
            let _ = writeln!(output, "{} declined", ended.item);
            Report_Fanout(ended, output);
            ExitCode::Ok
        }
        Err(refusal) =>
        {
            let _ = writeln!(output, "refused: {}", refusal.Describe());
            Code_For(&refusal)
        }
    };
}

/// Reports a widening, saying what it actually added rather than what was asked for.
///
/// The two differ whenever a path was already reserved, and printing the request back would
/// report a widening that did not happen. Every added path is named rather than counted: a
/// count is what a reader would have to go and check against the board anyway, and this line
/// is the only place the escape is visible at the moment it is made.
///
/// Adding nothing is reported as its own sentence and not as a success with an empty list. It
/// is not a refusal -- the territory is exactly what the caller asked for, which is the
/// outcome they wanted -- but a caller told only `widened` would believe a path landed that
/// was already there.
pub(super) fn Report_Widen(
    item: &nomos_ledger::ItemId,
    result: Result<Vec<String>, ClaimRefusal>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    return match result
    {
        Ok(added) if added.is_empty() =>
        {
            let _ = writeln!(output, "{item} already reserved every path given; nothing added");
            ExitCode::Ok
        }
        Ok(added) =>
        {
            let _ = writeln!(output, "{item} widened by {}: {}", added.len(), added.join(" "));
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
/// One blocked item's line: its identifier, the refusal label, its kind and origin, and
/// the refusal's own description.
///
/// `kind` and `origin` sit between the label and the description, the same two columns
/// [`super::listing::Print_Listing`] added for the same reason -- `OD-LEDGER-024`'s
/// follow-on, closed here.
pub(super) fn Print_Blocked(item: &LedgerItem, refusal: &ClaimRefusal, output: &mut impl std::io::Write)
{
    let _ = writeln!(
        output,
        "{:<13} {:<9} {:<11?} {:<9?} {}",
        item.id,
        Refusal_Label(refusal),
        item.kind,
        item.origin,
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

/// Names every item whose `depends_on` names the one just ended, and what each is now.
///
/// # Why this is printed at the moment of ending
///
/// `OD-LEDGER-038` decision 1: ending an item legitimately changes what other items can be
/// claimed, and until now nothing said so at the moment it happened. A `finish` frees its
/// dependents; a `decline` strands them permanently, and the measured case was seven items
/// made unreachable in one write with nothing printed. Both are correct operations whose
/// consequences were invisible.
///
/// # Why it labels through `Listing_Label`
///
/// So that an item cannot read `stranded` here and something else in `nomos work list`. The
/// reachability rule is `Claim_Refusal`'s, the word for it is `Refusal_Label`'s, and this
/// function implements neither — it selects which items to ask about and prints the answer.
///
/// Silent when nothing depends on the ended item, rather than a header over an empty list:
/// most endings free nobody, and a line saying so every time is noise that would teach a
/// reader to skip the line that matters.
fn Report_Fanout(ended: &Ended<'_>, output: &mut impl std::io::Write)
{
    let lines = Dependent_Lines(ended);

    if lines.is_empty()
    {
        return;
    }
    let _ = writeln!(output, "depending on {}:", ended.item);
    for line in lines
    {
        let _ = writeln!(output, "{line}");
    }
}

/// One line per item whose `depends_on` names the ended one, in board order — and empty when
/// nothing depends on it, which is the common case and the one that prints nothing at all.
///
/// The label comes from `listing`'s own reader so that an item cannot read `stranded` here
/// and something else in `nomos work list`; this function only selects whom to ask about.
fn Dependent_Lines(ended: &Ended<'_>) -> Vec<String>
{
    let Some(document) = ended.board
    else
    {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for item in &document.items
    {
        if item.depends_on.contains(ended.item)
        {
            let label = super::listing::Listing_Label(document, item, ended.now);
            let line = format!("  {label} {}", item.id);
            lines.push(line);
        }
    }

    return lines;
}

#[cfg(test)]
mod tests;
