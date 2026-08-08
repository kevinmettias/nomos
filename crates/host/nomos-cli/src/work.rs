//! `nomos work` — the ledger, from a terminal.

use nomos_ledger::{
    ClaimRefusal, DEFAULT_LEASE, ExclusionLedger, FileLedger, Finish, FinishRefusal, ItemId,
    ItemState, LedgerError, LedgerItem, ReleaseOutcome, Territory, VerificationPredicate,
};
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

    return match verb.as_str()
    {
        "list" => Ok(WorkCommand::List {
            state: value_of("--state"),
        }),
        "add" => Parse_Add(named, predicate_argv),
        "finish" => Ok(WorkCommand::Finish {
            item: ItemId::New(Required(value_of("--item").as_ref(), "--item")?),
            holder: Required(value_of("--holder").as_ref(), "--holder")?,
        }),
        "claim" | "renew" =>
        {
            let item = Required(value_of("--item").as_ref(), "--item")?;
            let holder = Required(value_of("--holder").as_ref(), "--holder")?;
            let lease = value_of("--lease")
                .map_or(Ok(DEFAULT_LEASE), |text| Parse_Duration(&text))?;

            if verb == "claim"
            {
                Ok(WorkCommand::Claim {
                    item: ItemId::New(item),
                    holder,
                    lease,
                })
            }
            else
            {
                Ok(WorkCommand::Renew {
                    item: ItemId::New(item),
                    holder,
                    lease,
                })
            }
        }
        "abandon" => Ok(WorkCommand::Abandon {
            item: ItemId::New(Required(value_of("--item").as_ref(), "--item")?),
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
    let patterns = Named_Values(named, "--territory-pattern");
    if paths.is_empty() && patterns.is_empty()
    {
        return Err(format!(
            "--territory is required: an item that reserves nothing excludes nobody.\n\n{}",
            Usage_Text()
        ));
    }

    let mut territory = Territory::Of_Files(paths);
    for pattern in patterns
    {
        territory = territory.With_Pattern(pattern);
    }

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
            id: ItemId::New(Required(value_of("--item").as_ref(), "--item")?),
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
        }),
    });
}

fn Named_Value(arguments: &[String], name: &str) -> Option<String>
{
    let position = arguments.iter().position(|argument| argument == name)?;
    return arguments.get(position.saturating_add(1)).cloned();
}

/// Every value given for a repeatable flag.
///
/// Repeating rather than comma-splitting, because a path may contain a comma and a
/// separator character invents a quoting problem the argument vector already solved.
fn Named_Values(arguments: &[String], name: &str) -> Vec<String>
{
    let mut values = Vec::new();
    let mut index = 0_usize;

    while let Some(argument) = arguments.get(index)
    {
        if argument == name
            && let Some(value) = arguments.get(index.saturating_add(1))
        {
            values.push(value.clone());
            index = index.saturating_add(2);
            continue;
        }
        index = index.saturating_add(1);
    }

    return values;
}

fn Required(value: Option<&String>, name: &str) -> Result<String, String>
{
    return value
        .cloned()
        .ok_or_else(|| format!("{name} is required.\n\n{}", Usage_Text()));
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
            \x20 list     [--state ready|claimed|blocked|done|declined]\n\
            \x20 add      --item <id> --title <text> --why <text> --done-when <text>\n\
            \x20          --territory <path> [--territory <path> …]\n\
            \x20          [--territory-pattern <glob> …] [--depends-on <id> …]\n\
            \x20          [-- <program> <args…>]\n\
            \x20 claim    --item <id> --holder <name> [--lease 2h]\n\
            \x20 renew    --item <id> --holder <name> [--lease 2h]\n\
            \x20 finish   --item <id> --holder <name>\n\
            \x20 abandon  --item <id> --holder <name> --reason <text>\n\
            \x20 validate\n\
            \x20 audit\n\
            \n\
            everything after `--` is the verification predicate, run directly with no \
            shell. `finish` runs it and records the item done only if it exits zero.\n\
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
        WorkCommand::Add { item } => Add(&ledger, item, output),
        WorkCommand::Finish { item, holder } => Report_Finish(
            // No working directory: the predicate runs where the user invoked `nomos`,
            // which for a repository tool run inside a repository is the repository. A
            // predicate silently relocated into `work/` would fail in ways that look
            // like the work being wrong.
            Finish(&mut ledger, &StdProcessLauncher, item, holder, None),
            output,
        ),
        WorkCommand::Claim {
            item,
            holder,
            lease,
        } => Report_Claim(ledger.Claim(item, holder, *lease), output),
        WorkCommand::Renew {
            item,
            holder,
            lease,
        } => Report_Claim(ledger.Renew(item, holder, *lease), output),
        WorkCommand::Abandon {
            item,
            holder,
            reason,
        } => Report_Release(
            ledger.Release(
                item,
                holder,
                ReleaseOutcome::Abandoned {
                    reason: reason.clone(),
                },
            ),
            output,
        ),
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

    let mut shown = 0_u32;
    for item in &document.items
    {
        let label = State_Label(&item.state);
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

/// Records a new item, refusing one whose territory is already spoken for.
///
/// The whole document is validated before the write, so an item that would break an
/// invariant never lands. The alternative — write now, notice later — leaves every agent
/// reading a ledger the system itself says is wrong.
fn Add(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    item: &LedgerItem,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let mut document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

    if document
        .items
        .iter()
        .any(|existing| existing.id == item.id)
    {
        let _ = writeln!(output, "{} is already on the ledger", item.id);
        return ExitCode::Conflict;
    }

    document.items.push(item.clone());

    return match ledger.Save(&document)
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
        Err(error) => Report_Error(&error, output),
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
            if refusal.Is_Retryable()
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
            ExitCode::Conflict
        }
    };
}

fn Report_Validation(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    return match ledger.Validate_Current()
    {
        Ok(()) =>
        {
            let _ = writeln!(output, "ledger is valid");
            ExitCode::Ok
        }
        Err(error) => Report_Error(&error, output),
    };
}

fn Audit(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

    for item in &document.items
    {
        for refusal in ledger.Conflicts(&item.territory)
        {
            if let ClaimRefusal::HeldBy { item: held, .. } = &refusal
                && held == &item.id
            {
                continue;
            }
            let _ = writeln!(output, "{}: {}", item.id, refusal.Describe());
        }
    }

    return ExitCode::Ok;
}

fn Report_Error(error: &LedgerError, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(output, "{error}");

    return match error
    {
        LedgerError::Invalid { .. } => ExitCode::ValidationError,
        LedgerError::Unreadable { .. }
        | LedgerError::Malformed { .. }
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
