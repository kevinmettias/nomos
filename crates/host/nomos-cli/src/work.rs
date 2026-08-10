//! `nomos work` — the ledger, from a terminal.

use crate::arguments::{Named_Value, Named_Values};
use nomos_ledger::{
    AddRefusal, ClaimRefusal, Claim_Refusal, DEFAULT_LEASE, ExclusionLedger, FileLedger, Finish,
    FinishRefusal, ItemId, ItemState, LedgerDocument, LedgerError, LedgerItem, ReleaseOutcome,
    SCHEMA_VERSION, Territory, Validate, VerificationPredicate,
};
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher, SystemClock};
use std::path::Path;
use std::time::Duration;

/// What the process exits with.
///
/// A contract, not an implementation detail. Agents branch on these rather than parsing
/// output, so they are documented here and covered by tests. The distinction that earns
/// its own code is [`ExitCode::ClaimUnavailable`]: an agent that is told the item is
/// taken should try another one, and an agent that is told the ledger is broken should
/// stop and get a human — collapsing those into "non-zero" makes the first case
/// indistinguishable from the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// The operation succeeded.
    Ok = 0,
    /// The ledger is invalid, or an operation was refused on its merits.
    ValidationError = 1,
    /// The command line was wrong.
    Usage = 2,
    /// Somebody else holds it. Retryable.
    ClaimUnavailable = 3,
    /// A conflict a human has to resolve.
    Conflict = 4,
    /// The ledger or its lock could not be used at all.
    StoreError = 5,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// What to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkCommand
{
    /// Show items, optionally filtered by state.
    List
    {
        /// Only items in this state.
        state: Option<String>,
    },
    /// Report one item, including what has happened to it.
    Show
    {
        /// Which item.
        item: ItemId,
    },
    /// Put a new item on the ledger.
    Add
    {
        /// The item to record.
        item: Box<LedgerItem>,
    },
    /// Run an item's verification predicate and record it done if it passes.
    Finish
    {
        /// Which item.
        item: ItemId,
        /// Who holds it.
        holder: String,
    },
    /// Take an item.
    Claim
    {
        /// Which item.
        item: ItemId,
        /// Who is taking it.
        holder: String,
        /// How long to hold it.
        lease: Duration,
    },
    /// Extend a held claim.
    Renew
    {
        /// Which item.
        item: ItemId,
        /// Who holds it.
        holder: String,
        /// How much longer.
        lease: Duration,
    },
    /// Take over an item whose holder's lease ran out, keeping the claim it displaces.
    ///
    /// Separate from [`WorkCommand::Claim`] because taking another agent's abandoned work is
    /// a decision, and a decision belongs in a verb somebody typed — `OD-LEDGER-012`.
    TakeOver
    {
        /// Which item.
        item: ItemId,
        /// Who is taking it over.
        holder: String,
        /// How long to hold it.
        lease: Duration,
    },
    /// Give up a claim without finishing.
    Abandon
    {
        /// Which item.
        item: ItemId,
        /// Who holds it.
        holder: String,
        /// Why.
        reason: String,
    },
    /// End an item that turned out not to be work.
    ///
    /// Separate from [`WorkCommand::Abandon`] because they are about different subjects —
    /// abandoning ends a claim and puts the item back on the board, declining ends the item —
    /// and because the items this exists for are unclaimed, which `abandon` cannot reach.
    /// `OD-LEDGER-019`.
    Decline
    {
        /// Which item.
        item: ItemId,
        /// Who is ending it.
        holder: String,
        /// Why it is not work.
        reason: String,
    },
    /// Check the ledger's invariants.
    Validate,
    /// Show what would block a claim.
    Audit,
}

/// Parses `nomos work` arguments.
///
/// # Errors
///
/// Returns a message naming what was wrong and what was expected.
pub fn Parse(arguments: &[String]) -> Result<WorkCommand, String>
{
    let Some(verb) = arguments.first()
    else
    {
        return Err(Usage_Text());
    };

    // Everything after a bare `--` is the verification argv, so a predicate carrying its
    // own flags needs no quoting and no escaping. The named arguments are parsed from
    // the part before it, which is why the split happens here rather than per-command.
    let separator = arguments.iter().position(|argument| argument == "--");
    let (named, predicate_argv) = match separator
    {
        Some(index) => (
            arguments.get(..index).unwrap_or_default(),
            arguments.get(index.saturating_add(1)..).unwrap_or_default(),
        ),
        None => (arguments, &[] as &[String]),
    };

    let value_of = |name: &str| Named_Value(named, name);
    let item_of = || -> Result<ItemId, String> {
        let text = Required(value_of("--item").as_ref(), "--item")?;

        return Ok(ItemId::New(text));
    };

    return match verb.as_str()
    {
        "list" => Ok(WorkCommand::List {
            state: value_of("--state"),
        }),
        "show" => Ok(WorkCommand::Show { item: item_of()? }),
        "add" => Parse_Add(named, predicate_argv),
        "finish" => Ok(WorkCommand::Finish {
            item: item_of()?,
            holder: Required(value_of("--holder").as_ref(), "--holder")?,
        }),
        // One parse for three verbs. `claim`, `renew` and `takeover` take exactly the same
        // three arguments and default the lease the same way, so sharing this is what stops
        // them drifting apart in what they accept — which they would be free to do while
        // being documented as identical.
        "claim" | "renew" | "takeover" =>
        {
            let item = item_of()?;
            let holder = Required(value_of("--holder").as_ref(), "--holder")?;
            let lease = value_of("--lease")
                .map_or(Ok(DEFAULT_LEASE), |text| Parse_Duration(&text))?;

            // The two verbs that do something unusual are named and `claim` is the
            // fallthrough, deliberately. A fourth verb added to the pattern above and
            // forgotten here becomes a plain claim, which refuses anything that is not
            // `Ready`; had `takeover` been the fallthrough it would displace a holder
            // instead.
            Ok(match verb.as_str()
            {
                "takeover" => WorkCommand::TakeOver {
                    item,
                    holder,
                    lease,
                },
                "renew" => WorkCommand::Renew {
                    item,
                    holder,
                    lease,
                },
                _ => WorkCommand::Claim {
                    item,
                    holder,
                    lease,
                },
            })
        }
        // Two verbs, one shape of argument list, and *not* one shared arm. `claim`, `renew`
        // and `takeover` share theirs above because they do the same thing to the same
        // subject with a different policy; these two do different things to different
        // subjects and merely take the same three words, so a shared arm would be a
        // similarity of spelling standing in for a similarity of meaning — which is the
        // conflation `OD-LEDGER-019` refuses at the verb level.
        "abandon" => Ok(WorkCommand::Abandon {
            item: item_of()?,
            holder: Required(value_of("--holder").as_ref(), "--holder")?,
            reason: Required(value_of("--reason").as_ref(), "--reason")?,
        }),
        "decline" => Ok(WorkCommand::Decline {
            item: item_of()?,
            holder: Required(value_of("--holder").as_ref(), "--holder")?,
            reason: Required(value_of("--reason").as_ref(), "--reason")?,
        }),
        "validate" => Ok(WorkCommand::Validate),
        "audit" => Ok(WorkCommand::Audit),
        other => Err(format!("unknown command `{other}`.\n\n{}", Usage_Text())),
    };
}

/// Builds an item from `add`'s arguments.
///
/// Territory is required and has no default. An item that reserves nothing excludes
/// nobody, so letting `--territory` be omitted would mean the easiest item to write is
/// the one that silently opts out of the exclusion the ledger exists to provide.
fn Parse_Add(named: &[String], predicate_argv: &[String]) -> Result<WorkCommand, String>
{
    let value_of = |name: &str| Named_Value(named, name);

    let paths = Named_Values(named, "--territory");

    // Refused before `--territory` is even checked, because it is the more specific answer:
    // somebody who passed only a pattern needs to be told the pattern is the problem, not
    // that they reserved nothing. See `OD-LEDGER-013`.
    //
    // A pattern is recorded unexpanded, and `Territory::Intersect` answers `Unknown` for
    // every comparison involving one. That is the correct answer to a question the
    // comparison cannot decide, and it is not correct as the *outcome of a flag*: the item
    // becomes unclaimable by anyone including its own author, every other claim on the board
    // is refused against it, and the refusal is non-retryable, which by the exit-code
    // contract tells an agent to stop and fetch a person. So the flag is withdrawn rather
    // than the refusal weakened.
    if let Some(pattern) = Named_Values(named, "--territory-pattern").first()
    {
        return Err(format!(
            "--territory-pattern is not supported: a pattern is never expanded, so every \
             comparison against `{pattern}` answers that independence cannot be established \
             — which makes the item unclaimable and refuses every other claim on the board.\n\
             \n\
             Reserve a directory instead. Territory is compared by containment, so \
             `--territory crates/spec` already reserves everything beneath it, and it is \
             decided from the text with no filesystem access.\n\n{}",
            Usage_Text()
        ));
    }

    if paths.is_empty()
    {
        return Err(format!(
            "--territory is required: an item that reserves nothing excludes nobody.\n\n{}",
            Usage_Text()
        ));
    }

    let territory = Territory::Of_Files(paths);
    let identifier = Required(value_of("--item").as_ref(), "--item")?;
    let item = ItemId::New(identifier);

    let verification = if predicate_argv.is_empty()
    {
        None
    }
    else
    {
        Some(VerificationPredicate::New(predicate_argv.to_vec()))
    };

    return Ok(WorkCommand::Add {
        item: Box::new(LedgerItem {
            id: item,
            title: Required(value_of("--title").as_ref(), "--title")?,
            why: Required(value_of("--why").as_ref(), "--why")?,
            done_when: Required(value_of("--done-when").as_ref(), "--done-when")?,
            territory,
            state: ItemState::Ready,
            depends_on: Named_Values(named, "--depends-on")
                .into_iter()
                .map(ItemId::New)
                .collect(),
            blocked: None,
            claim: None,
            verification,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            declined: None,
        }),
    });
}

fn Required(value: Option<&String>, name: &str) -> Result<String, String>
{
    return crate::arguments::Required(value, name, &Usage_Text());
}

/// Parses a lease such as `2h`, `30m` or `45s`.
///
/// # Errors
///
/// Returns a message when the text is not a recognized duration.
fn Parse_Duration(text: &str) -> Result<Duration, String>
{
    let (number, unit) = text.split_at(text.len().saturating_sub(1));
    let amount: u64 = number
        .parse()
        .map_err(|_| format!("`{text}` is not a duration; try 2h, 30m or 45s"))?;

    return match unit
    {
        "h" => Ok(Duration::from_secs(amount.saturating_mul(3_600))),
        "m" => Ok(Duration::from_secs(amount.saturating_mul(60))),
        "s" => Ok(Duration::from_secs(amount)),
        _ => Err(format!("`{text}` has no unit; try 2h, 30m or 45s")),
    };
}

fn Usage_Text() -> String
{
    return "usage: nomos work <command>\n\
            \n\
            \x20 list     [--state ready|waiting|held|snagged|claimed|blocked|done|declined]\n\
            \x20          `ready` means claimable now. An item nothing can claim is \
            reported as `waiting` (a dependency is unfinished), `held` (somebody holds \
            overlapping territory) or `snagged` (independence cannot be established), \
            from the same refusal `claim` would give.\n\
            \x20 show     --item <id>\n\
            \x20          one item in full: its claim, every claim given up on it with the \
            reason given, and its verification. `list` is a column per item and cannot \
            carry prose.\n\
            \x20 add      --item <id> --title <text> --why <text> --done-when <text>\n\
            \x20          --territory <path> [--territory <path> …]\n\
            \x20          [--depends-on <id> …]\n\
            \x20          [-- <program> <args…>]\n\
            \x20 claim    --item <id> --holder <name> [--lease 2h]\n\
            \x20 renew    --item <id> --holder <name> [--lease 2h]\n\
            \x20 takeover --item <id> --holder <name> [--lease 2h]\n\
            \x20          takes over an item listed `lapsed` — one whose holder's lease ran \
            out. `claim` never takes over a lapsed item; `takeover` does, and records the \
            claim it displaced, which `show` then reports.\n\
            \x20 finish   --item <id> --holder <name>\n\
            \x20 abandon  --item <id> --holder <name> --reason <text>\n\
            \x20 decline  --item <id> --holder <name> --reason <text>\n\
            \x20          ends an item that turned out not to be work — superseded by another \
            item, or refused by a record since it was written. `abandon` ends a *claim* and \
            puts the item back on the board for somebody else; `decline` ends the *item*, and \
            takes no claim, because an item nobody intends to do should not have to be claimed \
            first. It refuses an item somebody is holding, and one already done or already \
            declined.\n\
            \x20 validate\n\
            \x20 audit\n\
            \n\
            everything after `--` is the verification predicate, run directly with no \
            shell. `finish` runs the gate's own lint step first, derived from \
            .github/workflows/gate.yml rather than written here, then the item's \
            predicate, and records the item done only if both exit zero. An item whose \
            predicate passes while the gate is red is not finished.\n\
            \n\
            exit codes: 0 ok, 1 validation error, 2 usage, 3 claim unavailable \
            (retryable), 4 conflict, 5 store error"
        .to_owned();
}

/// Runs a command against the ledger at `directory`, writing to `output`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub fn Run(
    command: &WorkCommand,
    directory: &Path,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let mut ledger = FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );

    return match command
    {
        WorkCommand::List { state } => List(&ledger, state.as_deref(), output),
        WorkCommand::Show { item } => Show(&ledger, item, output),
        WorkCommand::Add { item } => Add(&mut ledger, item, &Published_Records(directory), output),
        WorkCommand::Finish { item, holder } =>
        {
            // No working directory: the predicate runs where the user invoked `nomos`,
            // which for a repository tool run inside a repository is the repository. A
            // predicate silently relocated into `work/` would fail in ways that look
            // like the work being wrong.
            let outcome = Finish(&mut ledger, &StdProcessLauncher, item, holder, None);

            Report_Finish(outcome, output)
        }
        WorkCommand::Claim {
            item,
            holder,
            lease,
        } =>
        {
            let outcome = ledger.Claim(item, holder, *lease);

            Report_Claim(outcome, output)
        }
        WorkCommand::Renew {
            item,
            holder,
            lease,
        } =>
        {
            let outcome = ledger.Renew(item, holder, *lease);

            Report_Claim(outcome, output)
        }
        // `Report_Claim` and `Code_For` unchanged, which is the point: one mapping from a
        // refusal to an exit code, so `claim` and `takeover` cannot come to disagree about
        // what a refusal means. No new code is introduced and the README's table does not move.
        WorkCommand::TakeOver {
            item,
            holder,
            lease,
        } =>
        {
            let outcome = ledger.Take_Over(item, holder, *lease);

            Report_Claim(outcome, output)
        }
        WorkCommand::Abandon {
            item,
            holder,
            reason,
        } =>
        {
            let abandoned = ReleaseOutcome::Abandoned {
                reason: reason.clone(),
            };
            let outcome = ledger.Release(item, holder, abandoned);

            Report_Release(outcome, output)
        }
        WorkCommand::Decline {
            item,
            holder,
            reason,
        } =>
        {
            let outcome = ledger.Decline(item, holder, reason);

            Report_Decline(item, outcome, output)
        }
        WorkCommand::Validate => Report_Validation(&ledger, output),
        WorkCommand::Audit => Audit(&ledger, output),
    };
}

fn List(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    state: Option<&str>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

    let now = SystemClock.Now();

    let mut shown = 0_u32;
    for item in &document.items
    {
        let label = Listing_Label(&document, item, now);
        if state.is_some_and(|wanted| !label.eq_ignore_ascii_case(wanted))
        {
            continue;
        }

        let holder = item
            .claim
            .as_ref()
            .map_or_else(String::new, |claim| format!("  [{}]", claim.holder));
        let _ = writeln!(output, "{:<13} {:<9}{holder}  {}", item.id, label, item.title);
        shown = shown.saturating_add(1);
    }

    if shown == 0
    {
        // An empty result and a filter that matched nothing look identical otherwise,
        // and the user's next action differs.
        let _ = match state
        {
            Some(wanted) => writeln!(output, "no items are {wanted}"),
            None => writeln!(output, "the ledger has no items"),
        };
    }

    return ExitCode::Ok;
}

/// Reports one item, including what has happened to it.
///
/// `list` is one line per item and cannot carry prose. That is why an abandonment had
/// nowhere to be read even once the ledger began keeping one: a record no surface reports
/// is a record only somebody willing to read the JSON can find, which is most of the way
/// back to not keeping it.
fn Show(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    item: &ItemId,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

    let Some(found) = document
        .items
        .iter()
        .find(|candidate| return &candidate.id == item)
    else
    {
        let _ = writeln!(output, "no item named {item}");
        return ExitCode::Conflict;
    };

    let now = SystemClock.Now();

    let _ = writeln!(output, "{} {}", found.id, found.title);
    let _ = writeln!(output, "state: {}", Listing_Label(&document, found, now));

    if let Some(claim) = &found.claim
    {
        // A lapsed claim is still shown, and still says it lapsed. It stops excluding
        // without being removed, so a reader who is not told would take it for a live one.
        let lapsed = if claim.Has_Lapsed(now) { " (lapsed)" } else { "" };
        let _ = writeln!(
            output,
            "held by {} since unix {} until unix {}{lapsed}",
            claim.holder,
            claim.acquired_at.Unix_Seconds(),
            claim.lease_expires_at.Unix_Seconds()
        );
    }

    // Reported, not merely stored. `OD-LEDGER-006`'s rule and `OD-LEDGER-012`'s reason for
    // obeying it here: a record no surface reports is one only somebody willing to read the
    // JSON can find, which is most of the way back to not keeping it. Each line names the
    // holder a takeover displaced and the window they held — who displaced them is the next
    // line's holder, or the live claim above.
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

    if let Some(record) = &found.verified
    {
        let _ = writeln!(
            output,
            "verified by `{}` at unix {} with exit {}",
            record.argv.join(" "),
            record.verified_at.Unix_Seconds(),
            record.exit_code
        );
    }

    return ExitCode::Ok;
}

/// Records a new item, refusing a duplicate identifier and a document that would not
/// validate. It does **not** refuse an item whose territory somebody already holds.
///
/// The whole document is validated before the write, so an item that would break an
/// invariant never lands. The alternative — write now, notice later — leaves every agent
/// reading a ledger the system itself says is wrong.
///
/// # The control this used to claim
///
/// This comment said "refusing one whose territory is already spoken for" from the commit
/// that wrote the command until `OD-LEDGER-010`, and nothing here ever did that.
/// [`nomos_ledger::Validate`] compares territories only between items holding an *active
/// claim*; an item arriving here holds none, so the comparison has nothing to say about it
/// and never fires.
///
/// The behaviour is the one to keep and the sentence is what moved. Opening an item on
/// ground somebody holds is how this board is used — `work/ledger.json` is in nobody's
/// territory precisely so that `add` stays available when every item on the board is held,
/// and a refusal here would shut the one door that is open when the board is fully
/// claimed. Exclusion belongs to `claim`, which is where an agent is about to edit files.
/// `add` writes a sentence about work that may not start for days.
///
/// What is guaranteed is therefore smaller than the old sentence promised, and it is
/// stated exactly because the promise is what the next reader will act on: the identifier
/// is unused, and the document that results still satisfies its own invariants. Nothing
/// here says two agents may edit one file, and nothing here is what stops them.
///
/// # The exclusion this *does* need, and did not have
///
/// Both of those guarantees were until `OD-LEDGER-021` written outside the lock. This
/// function read the whole document, decided against it and wrote the whole document back
/// with nothing held in between — the shape `OD-LEDGER-015` removed from `Claim`, `Renew`
/// and `Release`, left behind here because those three were fixed by name. An add that read
/// the board before a concurrent verb's write put its own snapshot back over it, and the
/// caller was told exit 0 either way.
///
/// So the read, the duplicate check and the write are now one [`FileLedger::Add`], and what
/// is left here is reporting. The check in particular had to travel with them: outside the
/// lock, two sessions adding one identifier both read a board without it, and the second
/// write produced a document `Validate` calls invalid — a board that then refuses to load,
/// for two callers who were each told they had succeeded.
/// Every record this repository has already published, as repository-relative paths.
///
/// Read here rather than in the store, and that division is the point rather than a
/// convenience. `OD-LEDGER-021` put the *decision* behind the ledger's lock and left the
/// command layer its input and its reporting; enumerating a repository is input. A general
/// exclusion ledger that learned to walk a source tree would be answering a question about
/// this repository's conventions, and `nomos-ledger` already stretches as far as it should
/// by knowing what a record filename folds to.
///
/// An unreadable or absent directory yields nothing rather than refusing. That is the one
/// judgement here worth stating, because this repository's usual rule is the opposite: a
/// check that cannot find its subject must fail loudly. It does not apply, because this is
/// not the check — the open-item comparison still runs, and it is the half that races. A
/// tree with no `docs/records` is a ledger being used somewhere that has no records, and
/// refusing every `add` in it would be this repository's convention refusing everybody
/// else's.
fn Published_Records(directory: &Path) -> Territory
{
    // The ledger lives in `work/`, so the repository is its parent. A `work/` at the root of
    // nothing has no records, which the walk below reports as none.
    let Some(root) = directory.parent()
    else
    {
        return Territory::Empty();
    };

    let Ok(entries) = std::fs::read_dir(root.join(RECORD_DIRECTORY))
    else
    {
        return Territory::Empty();
    };

    let mut published = Vec::new();
    for entry in entries.flatten()
    {
        if let Some(name) = entry.file_name().to_str()
        {
            // Repository-relative and forward-slashed, which is the spelling a territory is
            // authored in. `Normalize_Path` would accept either, and handing it the shape it
            // documents keeps the refusal's text readable by whoever has to act on it.
            published.push(format!("{RECORD_DIRECTORY}/{name}"));
        }
    }

    // Sorted so that an item colliding with two records is refused against the same one
    // every run. A refusal that names a different file each time reads as two defects.
    published.sort();

    return Territory::Of_Files(published);
}

/// Where this repository authors its decision records, relative to the repository root.
const RECORD_DIRECTORY: &str = "docs/records";

fn Add(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    item: &LedgerItem,
    published: &Territory,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    // The lock's holder name is a courtesy for a stale-takeover report and never an
    // identity that is checked, which is why `add` can name itself here while every other
    // verb passes the agent that asked. `add` takes no `--holder` because it takes no
    // claim: the item it writes is `Ready` and belongs to nobody yet.
    return match ledger.Add(item, "nomos work add", published)
    {
        Ok(()) =>
        {
            let _ = writeln!(
                output,
                "added {} reserving {} path(s)",
                item.id,
                item.territory.paths.len()
            );
            ExitCode::Ok
        }
        // Each arm keeps the exit code this command already gave it. Routing the write
        // through the store changed which type carries the refusal out and must not change
        // what an agent branching on the number concludes: a taken identifier is the
        // caller's to resolve by choosing another, an item that reserves nothing is the
        // caller's to correct, and only a ledger that cannot be read or written at all is
        // the one an agent stops and fetches a person for.
        Err(refusal) =>
        {
            let _ = writeln!(output, "{}", refusal.Describe());

            match refusal
            {
                AddRefusal::AlreadyPresent { .. } => ExitCode::Conflict,
                // The same code as a taken item identifier, and for the same reason: an
                // identifier somebody else holds, which the author resolves by choosing
                // another. `ExitCode::Usage` was the other candidate and is wrong — that is
                // the parser's code for a malformed invocation, and an agent that saw it
                // would go and inspect its own argument syntax, which is not the fix. The
                // refusal text is what tells the two record cases apart; the code tells an
                // agent what kind of thing happened, and this is the kind that already had
                // one.
                AddRefusal::RecordPublished { .. } | AddRefusal::RecordReserved { .. } =>
                {
                    ExitCode::Conflict
                }
                AddRefusal::WouldBeInvalid { .. } => ExitCode::ValidationError,
                AddRefusal::LedgerUnusable { .. } => ExitCode::StoreError,
            }
        }
    };
}

fn Report_Finish(
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
fn Blocking_Refusal(
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
fn Listing_Label(document: &LedgerDocument, item: &LedgerItem, now: Timestamp) -> &'static str
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

fn Report_Claim(
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
fn Report_Decline(
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

fn Report_Release(result: Result<(), ClaimRefusal>, output: &mut impl std::io::Write)
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
fn Report_Validation(
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
fn Audit(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    // Once, for the whole report. `Conflicts` loads and parses the ledger internally, so
    // asking it per item read the file once per item — sixty opens to answer one question
    // about sixty items, and sixty chances for the answer to be about a different board
    // than the line above it.
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

    let now = SystemClock.Now();

    let mut reported = 0_u32;
    for item in &document.items
    {
        let Some(refusal) = Blocking_Refusal(&document, item, now)
        else
        {
            continue;
        };

        let _ = writeln!(
            output,
            "{:<13} {:<9} {}",
            item.id,
            Refusal_Label(&refusal),
            // `Describe` directly, and no local rephrasing. Its held arm used to name the
            // blocker where the subject belongs, so this line phrased that one arm itself;
            // `OD-LEDGER-014` fixed the library and deleted the workaround in the same
            // commit, because the whole cost of the workaround was that two renderings of
            // one refusal outlived the reason for the second.
            refusal.Describe()
        );
        reported = reported.saturating_add(1);
    }

    if reported == 0
    {
        // Empty output used to mean either "nothing is blocked" or "the report cannot
        // express what is blocking this", and only one of those is good news.
        let _ = writeln!(output, "nothing claimable is blocked");
    }

    return ExitCode::Ok;
}

fn Report_Error(error: &LedgerError, output: &mut impl std::io::Write) -> ExitCode
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

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arguments(text: &str) -> Vec<String>
    {
        return text.split_whitespace().map(str::to_owned).collect();
    }

    #[test]
    fn Test_Claim_Should_Parse_With_A_Default_Lease()
    {
        let parsed = Parse(&Arguments("claim --item T-1 --holder agent-a")).unwrap();

        assert_eq!(
            parsed,
            WorkCommand::Claim {
                item: ItemId::New("T-1"),
                holder: "agent-a".to_owned(),
                lease: DEFAULT_LEASE,
            }
        );
    }

    /// `takeover` shares `claim`'s parse, so it must share `claim`'s defaulting too. A verb
    /// that silently required `--lease` where its two siblings default it is a verb agents
    /// learn by being refused.
    #[test]
    fn Test_Takeover_Should_Parse_With_A_Default_Lease()
    {
        let parsed = Parse(&Arguments("takeover --item T-1 --holder agent-b")).unwrap();

        assert_eq!(
            parsed,
            WorkCommand::TakeOver {
                item: ItemId::New("T-1"),
                holder: "agent-b".to_owned(),
                lease: DEFAULT_LEASE,
            }
        );
    }

    /// One word, no hyphen, matching the other nine verbs — and not a spelling of `claim`.
    ///
    /// The second assertion is the one worth making: the shared parse arm hands three verbs
    /// the same arguments, so the only thing keeping them distinct is which variant it
    /// returns. A `takeover` that parsed as a `Claim` would refuse every lapsed item while
    /// appearing to be the remedy for one.
    #[test]
    fn Test_Takeover_Should_Not_Parse_As_A_Claim()
    {
        let parsed = Parse(&Arguments("takeover --item T-1 --holder agent-b --lease 30m")).unwrap();

        assert!(matches!(parsed, WorkCommand::TakeOver { .. }), "{parsed:?}");
        assert!(Parse(&Arguments("takeover --holder agent-b")).is_err());
        assert!(Parse(&Arguments("takeover --item T-1")).is_err());
    }

    #[test]
    fn Test_Lease_Units_Should_Parse()
    {
        assert_eq!(Parse_Duration("2h").unwrap(), Duration::from_secs(7_200));
        assert_eq!(Parse_Duration("30m").unwrap(), Duration::from_secs(1_800));
        assert_eq!(Parse_Duration("45s").unwrap(), Duration::from_secs(45));
    }

    /// A lease with no unit is ambiguous. Guessing seconds would silently give an agent
    /// a two-second lease when it asked for two hours.
    #[test]
    fn Test_A_Unitless_Lease_Should_Be_Refused()
    {
        assert!(Parse_Duration("2").is_err());
        assert!(Parse_Duration("").is_err());
        assert!(Parse_Duration("2d").is_err());
    }

    /// A missing required argument must name what is missing, not just print usage.
    #[test]
    fn Test_A_Missing_Argument_Should_Name_Itself()
    {
        let error = Parse(&Arguments("claim --holder agent-a")).unwrap_err();

        assert!(error.contains("--item"));
    }

    #[test]
    fn Test_An_Unknown_Command_Should_Be_A_Usage_Error()
    {
        let error = Parse(&Arguments("frobnicate")).unwrap_err();

        assert!(error.contains("frobnicate"));
        assert!(error.contains("usage"));
    }

    /// One word, no hyphen, matching the other ten verbs — and not a spelling of `abandon`.
    ///
    /// The second half is the assertion worth making. These two take the same three
    /// arguments and do opposite things to the item: a `decline` that parsed as an `Abandon`
    /// would put the item back on the board while reporting that it had been ended, which is
    /// exactly the state `OD-LEDGER-019` was written to stop an item being left in.
    #[test]
    fn Test_Decline_Should_Not_Parse_As_An_Abandon()
    {
        let parsed = Parse(&Arguments("decline --item T-1 --holder agent-a --reason done"))
            .unwrap();

        assert_eq!(
            parsed,
            WorkCommand::Decline {
                item: ItemId::New("T-1"),
                holder: "agent-a".to_owned(),
                reason: "done".to_owned(),
            }
        );
    }

    /// The state carries its reason so that an item cannot be declined reasonlessly, and the
    /// flag is required so that the guarantee reaches somebody typing rather than stopping at
    /// the type.
    #[test]
    fn Test_Decline_Should_Require_A_Reason()
    {
        let error =
            Parse(&Arguments("decline --item T-1 --holder agent-a")).unwrap_err();

        assert!(error.contains("--reason"), "{error}");
        assert!(Parse(&Arguments("decline --holder agent-a --reason r")).is_err());
        assert!(Parse(&Arguments("decline --item T-1 --reason r")).is_err());
    }

    /// The usage text is what an agent reads at exit 2, so a verb missing from it is a verb
    /// that does not exist as far as the next session is concerned.
    #[test]
    fn Test_The_Usage_Text_Should_Name_Every_Verb_It_Accepts()
    {
        let usage = Usage_Text();

        for verb in [
            "list", "show", "add", "claim", "renew", "takeover", "finish", "abandon",
            "decline", "validate", "audit",
        ]
        {
            assert!(usage.contains(verb), "the usage text does not name `{verb}`");
            assert!(
                Parse(&Arguments(verb)).is_ok() || !Parse(&Arguments(verb)).unwrap_err().contains("unknown command"),
                "the usage text names `{verb}` and the parser does not accept it"
            );
        }
    }

    /// A board holding one declined item and nothing else.
    fn Board_With_A_Declined_Item() -> LedgerDocument
    {
        let mut item = match Parse(&Arguments(
            "add --item T-1 --title t --why w --done-when d --territory src/a.rs",
        ))
        .unwrap()
        {
            WorkCommand::Add { item } => *item,
            other => panic!("expected an add, got {other:?}"),
        };

        item.Decline("superseded by T-2", "agent-a", Timestamp::From_Unix_Seconds(1));

        return LedgerDocument {
            schema_version: nomos_ledger::SCHEMA_VERSION,
            items: vec![item],
        };
    }

    /// The column an agent reads before claiming has to say the item is over.
    ///
    /// This is the whole of what the state buys at the surface. `P10-REQUIRABLE-DECLARED` was
    /// superseded twice and read `ready` both times, with the reason behind `work show` where
    /// nobody looks first — so the second session claimed it and spent its run establishing
    /// that the first one was right.
    #[test]
    fn Test_A_Declined_Item_Should_Be_Listed_As_Declined()
    {
        let document = Board_With_A_Declined_Item();
        let item = document.items.first().expect("the fixture has an item");

        assert_eq!(
            Listing_Label(&document, item, Timestamp::From_Unix_Seconds(2)),
            "declined"
        );
    }

    /// `work audit` answers for items somebody could act on, and nobody can act on this one.
    ///
    /// `P10-AUDIT-STATE` settled that once: an audit that reported blockers for finished work
    /// made forty-four lines nobody could do anything about. A newly reachable terminal state
    /// is the obvious way to reopen it.
    #[test]
    fn Test_Audit_Should_Not_Answer_For_A_Declined_Item()
    {
        let document = Board_With_A_Declined_Item();
        let item = document.items.first().expect("the fixture has an item");

        assert!(
            Blocking_Refusal(&document, item, Timestamp::From_Unix_Seconds(2)).is_none(),
            "audit answered for an item nobody can act on"
        );
    }

    fn Added(text: &str) -> LedgerItem
    {
        return match Parse(&Arguments(text)).unwrap()
        {
            WorkCommand::Add { item } => *item,
            other => panic!("expected an add, got {other:?}"),
        };
    }

    #[test]
    fn Test_Add_Should_Collect_Repeated_Territory_Flags()
    {
        let item = Added(
            "add --item T-1 --title t --why w --done-when d \
             --territory src/a.rs --territory src/b.rs",
        );

        assert_eq!(item.territory.paths, vec!["src/a.rs", "src/b.rs"]);
        assert_eq!(item.state, ItemState::Ready);
    }

    /// The predicate is everything after `--`, so a command carrying its own flags needs
    /// no quoting and no escaping — and the named arguments before the separator are
    /// still parsed normally.
    #[test]
    fn Test_Add_Should_Take_The_Predicate_After_The_Separator()
    {
        let item = Added(
            "add --item T-1 --title t --why w --done-when d --territory src/a.rs \
             -- cargo test -p nomos-spec-model --lib",
        );

        assert_eq!(
            item.verification.map(|predicate| predicate.argv),
            Some(vec![
                "cargo".to_owned(),
                "test".to_owned(),
                "-p".to_owned(),
                "nomos-spec-model".to_owned(),
                "--lib".to_owned(),
            ])
        );
    }

    /// A flag that looks like a named argument but sits after the separator belongs to
    /// the predicate. Without the split, `--lib` above would be read as an option to
    /// `nomos work`.
    #[test]
    fn Test_Arguments_After_The_Separator_Should_Not_Be_Read_As_Options()
    {
        let item = Added(
            "add --item T-1 --title t --why w --done-when d --territory src/a.rs \
             -- prog --title stolen",
        );

        assert_eq!(item.title, "t");
    }

    /// An item that reserves nothing excludes nobody, so the ledger would hand two
    /// agents the same files and call it disjoint.
    #[test]
    fn Test_Add_Should_Refuse_An_Item_With_No_Territory()
    {
        let error =
            Parse(&Arguments("add --item T-1 --title t --why w --done-when d")).unwrap_err();

        assert!(error.contains("--territory"));
    }

    /// The flag `OD-LEDGER-013` withdrew.
    ///
    /// A pattern reaches `Territory::Intersect`, which short-circuits to `Unknown` before
    /// comparing a single path, and `Unknown` refuses non-retryably. So this one flag made an
    /// item nobody could claim *and* refused every other claim on the board, and told each
    /// refused agent to stop and fetch a person rather than try another item.
    ///
    /// Pinned as a **usage** error rather than a ledger one: the mistake is in what was
    /// typed, and an exit code of 1 or 5 here would read as the board being broken, which is
    /// the confusion this item exists to remove.
    #[test]
    fn Test_Add_Should_Refuse_A_Territory_Pattern()
    {
        let error = Parse(&Arguments(
            "add --item T-1 --title t --why w --done-when d \
             --territory src/a.rs --territory-pattern crates/spec/**",
        ))
        .unwrap_err();

        assert!(
            error.contains("--territory-pattern is not supported"),
            "the refusal must name the flag that is unsupported: {error}"
        );
        assert!(
            error.contains("crates/spec/**"),
            "and quote the pattern it refused, or an author cannot tell which one: {error}"
        );
    }

    /// The pattern is refused even when it is the only territory given.
    ///
    /// The ordering control. `--territory` was checked first before this item, so a pattern
    /// on its own reported "an item that reserves nothing excludes nobody" — a true sentence
    /// about the wrong problem, which sends the author to add a path rather than to drop the
    /// flag. Swapping the two checks back turns this red while the test above stays green.
    #[test]
    fn Test_A_Pattern_Alone_Should_Be_Refused_As_A_Pattern()
    {
        let error = Parse(&Arguments(
            "add --item T-1 --title t --why w --done-when d --territory-pattern crates/**",
        ))
        .unwrap_err();

        assert!(
            error.contains("--territory-pattern is not supported"),
            "a pattern alone must be refused for being a pattern: {error}"
        );
    }

    /// The flag is gone from the usage text as well as from the parser.
    ///
    /// `done_when` requires the flag to leave the advertised surface, and the usage string is
    /// the copy an agent actually reads — it is printed on every refusal above.
    #[test]
    fn Test_The_Usage_Text_Should_Not_Advertise_A_Territory_Pattern()
    {
        assert!(
            !Usage_Text().contains("--territory-pattern"),
            "the usage text still advertises a flag that is refused: {}",
            Usage_Text()
        );
    }

    #[test]
    fn Test_Finish_Should_Parse()
    {
        assert_eq!(
            Parse(&Arguments("finish --item T-1 --holder agent-a")).unwrap(),
            WorkCommand::Finish {
                item: ItemId::New("T-1"),
                holder: "agent-a".to_owned(),
            }
        );
    }

    /// The exit codes are a contract agents branch on, so their values are pinned.
    #[test]
    fn Test_Exit_Codes_Should_Be_Stable()
    {
        assert_eq!(ExitCode::Ok.Value(), 0);
        assert_eq!(ExitCode::ValidationError.Value(), 1);
        assert_eq!(ExitCode::Usage.Value(), 2);
        assert_eq!(ExitCode::ClaimUnavailable.Value(), 3);
        assert_eq!(ExitCode::Conflict.Value(), 4);
        assert_eq!(ExitCode::StoreError.Value(), 5);
    }
}
