//! Reading a `nomos work` command line, and the usage text that says what one may be.

use std::time::Duration;

use nomos_ledger::{
    DEFAULT_LEASE, ItemId, ItemKind, ItemOrigin, ItemState, LedgerItem, Territory,
    VerificationPredicate,
};

use crate::arguments::{Named_Value_From_String_Arguments, Named_Values_From_String_Arguments};

use super::{ClaimRequest, EndingRequest, ListingScope, WorkCommand};

mod usage;

/// Parses `nomos work` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub fn Work_Command_From_String_Arguments(arguments: &[String]) -> Result<WorkCommand, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };
    let Split { named, predicate_argv } = Split_From_String_Arguments(arguments);

    return match verb.as_str()
    {
        "list" => Ok(Parse_List(named)),
        "show" => Parse_Show(named),
        "add" => Parse_Add(named, predicate_argv),
        "finish" => Parse_Finish(named),
        "claim" | "renew" | "takeover" => Parse_Reservation(verb, named),
        "abandon" => Parse_Abandon(named),
        "decline" => Parse_Decline(named),
        "widen" => Parse_Widen(named),
        "validate" => Ok(WorkCommand::Validate),
        "audit" => Ok(WorkCommand::Audit),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

/// An argument list cut in two at a bare `--`.
///
/// Named rather than a pair. Both halves are `&[String]` and the compiler cannot tell
/// them apart, so a caller that took them in the wrong order would hand a verb its
/// predicate's flags and still build.
struct Split<'a>
{
    named: &'a [String],
    predicate_argv: &'a [String],
}

/// The named arguments and the verification argv, split at a bare `--`.
///
/// Everything after the separator is the predicate's own argv, so a predicate carrying its
/// own flags needs no quoting and no escaping. The split happens once here rather than
/// per-command.
fn Split_From_String_Arguments(arguments: &[String]) -> Split<'_>
{
    let Some(index) = arguments.iter().position(|argument| return argument == "--")
    else
    {
        return Split {
            named: arguments,
            predicate_argv: &[],
        };
    };

    return Split {
        named: arguments.get(..index).unwrap_or_default(),
        predicate_argv: arguments.get(index.saturating_add(1)..).unwrap_or_default(),
    };
}

/// Listing takes no argument that can be wrong, so it does not return a `Result`.
///
/// Every other verb here can refuse its arguments and this one cannot, and a `Result` that
/// is never `Err` invites a caller to write a handler for a case that does not exist.
fn Parse_List(named: &[String]) -> WorkCommand
{
    return WorkCommand::List {
        state: Named_Value_From_String_Arguments(named, "--state"),
        scope: Listing_Scope(named),
    };
}

/// How much of the board `list` was asked for: the live board, unless [`ALL`] says otherwise.
///
/// `OD-LEDGER-041` decided both halves. The whole board stays in the file, because the claim
/// check reads a dependency's terminal state in order to answer its dependent; the default
/// *listing* answers with what a session can still act on, because 98.7 per cent of the rows
/// it printed were items nobody could take and the operating contract sends every session to
/// it before anything else.
///
/// A bare flag rather than another `--state` value. `--state` names one of the words a row can
/// carry, and "all" is not one of them: admitting it would make `--state all` a sibling of
/// `--state done` and leave the usage text promising a state no item is ever in.
///
/// Read here rather than through [`crate::arguments`]. Every helper there answers about a flag
/// that takes a value, and this is the only flag in the group that takes none -- a shared
/// predicate with one caller is a second place for the same answer to live before it is a
/// saving.
fn Listing_Scope(named: &[String]) -> ListingScope
{
    if named.iter().any(|argument| return argument == ALL)
    {
        return ListingScope::Whole;
    }

    return ListingScope::Live;
}

/// The flag that asks `list` for every row, the ones that have ended included.
const ALL: &str = "--all";

fn Parse_Show(named: &[String]) -> Result<WorkCommand, String>
{
    return Ok(WorkCommand::Show {
        item: Item_Of(named)?,
    });
}

/// Builds an item from `add`'s arguments.
///
/// Territory is required and has no default. An item that reserves nothing excludes
/// nobody, so letting `--territory` be omitted would mean the easiest item to write is
/// the one that silently opts out of the exclusion the ledger exists to provide.
fn Parse_Add(named: &[String], predicate_argv: &[String]) -> Result<WorkCommand, String>
{
    let amended = Named_Values_From_String_Arguments(named, "--amends");
    let amending = Territory::Of_Files(amended);
    let territory = Parse_Territory(named, &amending)?;
    let verification = Timed_Predicate(Parse_Predicate(predicate_argv), named)?;
    let item = New_Item(named, territory, verification)?;

    return Ok(WorkCommand::Add {
        item: Box::new(item),
        amending,
    });
}

/// What the item reserves, or the message saying why what was given cannot reserve.
///
/// `--amends` reserves as well as declares, and is folded in here rather than being a second
/// thing an author has to remember to also pass to `--territory`. An amendment edits the
/// record it names, so an item that declared one without reserving it would be editing a file
/// nothing keeps a second writer off — and requiring both spellings would make that omission
/// the easy mistake instead of an impossible one.
fn Parse_Territory(named: &[String], amending: &Territory) -> Result<Territory, String>
{
    if let Some(pattern) = Named_Values_From_String_Arguments(named, "--territory-pattern").first()
    {
        return Err(Refuse_A_Pattern(pattern));
    }

    let mut paths = Named_Values_From_String_Arguments(named, "--territory");
    paths.extend(amending.paths.iter().cloned());

    if paths.is_empty()
    {
        return Err(format!(
            "--territory is required: an item that reserves nothing excludes nobody. \
             `--amends <record>` reserves too, and says the item edits that record rather \
             than allocating it.\n\n{}",
            Usage_Text()
        ));
    }

    return Ok(Territory::Of_Files(paths));
}

/// Applies `--timeout` to a parsed predicate, or refuses it when there is no predicate to
/// bound.
///
/// `VerificationPredicate::From_String_Arguments` always starts a predicate at its own ten-minute default,
/// which is too short for a territory that legitimately runs quiet for a while — a
/// corpus-backed crate's determinism test, for one, which can stay silent for several
/// minutes before printing anything and trips `nomos-ledger`'s idle bound (half the wall
/// bound) long before the wall bound itself would. Without this flag no author could ever
/// declare a predicate patient enough for that territory, on any item, ever.
fn Timed_Predicate(predicate: Option<VerificationPredicate>, named: &[String]) -> Result<Option<VerificationPredicate>, String>
{
    let Some(text) = Named_Value_From_String_Arguments(named, "--timeout")
    else
    {
        return Ok(predicate);
    };

    let Some(mut predicate) = predicate
    else
    {
        return Err(format!(
            "--timeout {text:?} was given but there is no predicate to bound: everything \
             after `--` is the predicate, and nothing followed it here.\n\n{}",
            Usage_Text()
        ));
    };

    predicate.timeout_seconds = Parse_Duration(&text)?.as_secs();

    return Ok(Some(predicate));
}

/// The predicate an item is verified by, when one was given after `--`.
fn Parse_Predicate(predicate_argv: &[String]) -> Option<VerificationPredicate>
{
    if predicate_argv.is_empty()
    {
        return None;
    }

    return Some(VerificationPredicate::From_String_Arguments(predicate_argv.to_vec()));
}

/// The item itself, from the arguments describing it.
fn New_Item(
    named: &[String],
    territory: Territory,
    verification: Option<VerificationPredicate>,
) -> Result<LedgerItem, String>
{
    let value_of = |name: &str| Named_Value_From_String_Arguments(named, name);
    let depends_on = Named_Values_From_String_Arguments(named, "--depends-on").into_iter().map(ItemId::New);

    return Ok(LedgerItem {
        id: Item_Of(named)?,
        title: Required_Value(value_of("--title").as_ref(), "--title")?,
        why: Required_Value(value_of("--why").as_ref(), "--why")?,
        done_when: Required_Value(value_of("--done-when").as_ref(), "--done-when")?,
        kind: Kind_Of(named)?,
        origin: Origin_Of(named)?,
        territory,
        state: ItemState::Ready,
        depends_on: depends_on.collect(),
        blocked: None,
        claim: None,
        verification,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
    });
}

/// The kind an argument list names. `OD-LEDGER-024`.
///
/// Required, like `--title` and `--territory`: an item that does not say what kind of
/// work it is is exactly the row that record exists to stop being written.
fn Kind_Of(named: &[String]) -> Result<ItemKind, String>
{
    let text = Required_Value(Named_Value_From_String_Arguments(named, "--kind").as_ref(), "--kind")?;

    return match text.as_str()
    {
        "capability" => Ok(ItemKind::Capability),
        "decision" => Ok(ItemKind::Decision),
        "validation" => Ok(ItemKind::Validation),
        "correction" => Ok(ItemKind::Correction),
        "cleanup" => Ok(ItemKind::Cleanup),
        other => Err(format!(
            "--kind {other:?} is not one of capability, decision, validation, correction, \
             cleanup.\n\n{}",
            Usage_Text()
        )),
    };
}

/// The origin an argument list names. `OD-LEDGER-024`.
///
/// Required, for the reason [`Kind_Of`] is.
fn Origin_Of(named: &[String]) -> Result<ItemOrigin, String>
{
    let text = Required_Value(Named_Value_From_String_Arguments(named, "--origin").as_ref(), "--origin")?;

    return match text.as_str()
    {
        "required" => Ok(ItemOrigin::Required),
        "proposed" => Ok(ItemOrigin::Proposed),
        other => Err(format!(
            "--origin {other:?} is not one of required, proposed.\n\n{}",
            Usage_Text()
        )),
    };
}

fn Parse_Finish(named: &[String]) -> Result<WorkCommand, String>
{
    return Ok(WorkCommand::Finish {
        item: Item_Of(named)?,
        holder: Holder_Of(named)?,
    });
}

/// One parse for three verbs.
///
/// `claim`, `renew` and `takeover` take exactly the same three arguments and default the
/// lease the same way, so sharing this is what stops them drifting apart in what they
/// accept — which they would be free to do while being documented as identical.
///
/// The two that do something unusual are named and `claim` is the fallthrough,
/// deliberately. A fourth verb added to the caller's pattern and forgotten here becomes a
/// plain claim, which refuses anything that is not `Ready`; had `takeover` been the
/// fallthrough it would displace a holder instead.
fn Parse_Reservation(verb: &str, named: &[String]) -> Result<WorkCommand, String>
{
    let lease = Named_Value_From_String_Arguments(named, "--lease")
        .map_or(Ok(DEFAULT_LEASE), |text| return Parse_Duration(&text))?;
    let request = ClaimRequest {
        item: Item_Of(named)?,
        holder: Holder_Of(named)?,
        lease,
    };

    return Ok(match verb
    {
        "takeover" => WorkCommand::TakeOver(request),
        "renew" => WorkCommand::Renew(request),
        _ => WorkCommand::Claim(request),
    });
}

/// `abandon` and `decline` take one shape of argument list, and are deliberately not one
/// parse.
///
/// `claim`, `renew` and `takeover` share theirs because they do the same thing to the same
/// subject with a different policy; these two do different things to different subjects and
/// merely take the same three words, so a shared parse would be a similarity of spelling
/// standing in for a similarity of meaning — which is the conflation `OD-LEDGER-019`
/// refuses at the verb level.
fn Parse_Abandon(named: &[String]) -> Result<WorkCommand, String>
{
    return Ok(WorkCommand::Abandon(Ending_Request(named)?));
}

/// The other half of the pair [`Parse_Abandon`] documents.
/// `decline --item <id> --holder <name> --reason <text>`.
fn Parse_Decline(named: &[String]) -> Result<WorkCommand, String>
{
    return Ok(WorkCommand::Decline(Ending_Request(named)?));
}

/// `widen --item <id> --holder <name> --territory <path> [--territory <path> …]`.
///
/// `--territory` and not a flag of its own, because it is the same argument `add` takes and
/// naming it twice would let an author learn one spelling and meet the other. At least one is
/// required: a widening that adds nothing is not a thing to ask for, and accepting it would
/// write a lock acquisition and a refusal path for a request that cannot change anything.
///
/// `--territory-pattern` is refused here exactly as `add` refuses it, through the same
/// function, so the withdrawn flag does not come back through the new verb.
fn Parse_Widen(named: &[String]) -> Result<WorkCommand, String>
{
    let item = Item_Of(named)?;
    let holder = Holder_Of(named)?;
    let adding = Paths_To_Reserve(named)?;

    return Ok(WorkCommand::Widen {
        item,
        holder,
        adding,
    });
}

/// The paths a `widen` reserves, refusing the two ways `--territory` can be given wrongly.
fn Paths_To_Reserve(named: &[String]) -> Result<Vec<String>, String>
{
    if let Some(pattern) = Named_Values_From_String_Arguments(named, "--territory-pattern").first()
    {
        return Err(Refuse_A_Pattern(pattern));
    }

    let adding = Named_Values_From_String_Arguments(named, "--territory");
    if adding.is_empty()
    {
        return Err(
            "--territory is required: a widening that adds no path changes nothing. \
             `widen` only ever adds -- to reserve fewer paths, decline the item and re-author \
             it, because dropping a path drops the `done_when` clause it carried."
                .to_owned(),
        );
    }

    return Ok(adding);
}

/// Parses a lease such as `2h`, `30m` or `45s`.
///
/// # Errors
///
/// Returns a message when the text is not a recognized duration.
const SECONDS_PER_MINUTE: u64 = 60;

const MINUTES_PER_HOUR: u64 = 60;

const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;

pub(super) fn Parse_Duration(text: &str) -> Result<Duration, String>
{
    let (number, unit) = text.split_at(text.len().saturating_sub(1));
    let amount: u64 = number
        .parse()
        .map_err(|cause| format!("`{text}` is not a duration; try 2h, 30m or 45s: {cause}"))?;

    return match unit
    {
        "h" => Ok(Duration::from_secs(amount.saturating_mul(SECONDS_PER_HOUR))),
        "m" => Ok(Duration::from_secs(amount.saturating_mul(SECONDS_PER_MINUTE))),
        "s" => Ok(Duration::from_secs(amount)),
        _ => Err(format!("`{text}` has no unit; try 2h, 30m or 45s")),
    };
}

pub(super) fn Usage_Text() -> String
{
    return format!("usage: nomos work <command>\n\n{}\n{}", usage::VERBS, usage::NOTES);
}

/// Why `--territory-pattern` is withdrawn rather than supported.
///
/// Refused before `--territory` is even checked, because it is the more specific answer:
/// somebody who passed only a pattern needs to be told the pattern is the problem, not that
/// they reserved nothing. See `OD-LEDGER-013`.
///
/// A pattern is recorded unexpanded, and `Territory::Intersect` answers `Unknown` for every
/// comparison involving one. That is the correct answer to a question the comparison cannot
/// decide, and it is not correct as the *outcome of a flag*: the item becomes unclaimable by
/// anyone including its own author, every other claim on the board is refused against it,
/// and the refusal is non-retryable, which by the exit-code contract tells an agent to stop
/// and fetch a person. So the flag is withdrawn rather than the refusal weakened.
fn Refuse_A_Pattern(pattern: &str) -> String
{
    return format!(
        "--territory-pattern is not supported: a pattern is never expanded, so every \
         comparison against `{pattern}` answers that independence cannot be established — \
         which makes the item unclaimable and refuses every other claim on the board.\n\
         \n\
         Reserve a directory instead. Territory is compared by containment, so \
         `--territory crates/spec` already reserves everything beneath it, and it is \
         decided from the text with no filesystem access.\n\n{}",
        Usage_Text()
    );
}

/// The item an argument list names.
fn Item_Of(named: &[String]) -> Result<ItemId, String>
{
    let text = Required_Value(Named_Value_From_String_Arguments(named, "--item").as_ref(), "--item")?;

    return Ok(ItemId::New(text));
}

/// The holder an argument list names.
fn Holder_Of(named: &[String]) -> Result<String, String>
{
    return Required_Value(Named_Value_From_String_Arguments(named, "--holder").as_ref(), "--holder");
}

/// The reason an argument list gives.
fn Reason_Of(named: &[String]) -> Result<String, String>
{
    return Required_Value(Named_Value_From_String_Arguments(named, "--reason").as_ref(), "--reason");
}

/// The three words `abandon` and `decline` both take.
fn Ending_Request(named: &[String]) -> Result<EndingRequest, String>
{
    return Ok(EndingRequest {
        item: Item_Of(named)?,
        holder: Holder_Of(named)?,
        reason: Reason_Of(named)?,
    });
}

fn Required_Value(value: Option<&String>, name: &str) -> Result<String, String>
{
    return crate::arguments::Required_Value(value, crate::arguments::Name(name), crate::arguments::Usage(&Usage_Text()));
}

#[cfg(test)]
mod tests;
