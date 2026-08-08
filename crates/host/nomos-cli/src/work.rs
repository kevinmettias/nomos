//! `nomos work` — the ledger, from a terminal.

use nomos_ledger::{
    ClaimRefusal, DEFAULT_LEASE, ExclusionLedger, FileLedger, ItemId, ItemState, LedgerError,
    ReleaseOutcome,
};
use nomos_platform_std::{FileLock, StdFileSystem, SystemClock};
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

    let Some(value_of) = Some(|name: &str| Named_Value(arguments, name))
    else
    {
        return Err(Usage_Text());
    };

    return match verb.as_str()
    {
        "list" => Ok(WorkCommand::List {
            state: value_of("--state"),
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

fn Named_Value(arguments: &[String], name: &str) -> Option<String>
{
    let position = arguments.iter().position(|argument| argument == name)?;
    return arguments.get(position.saturating_add(1)).cloned();
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
            \x20 claim    --item <id> --holder <name> [--lease 2h]\n\
            \x20 renew    --item <id> --holder <name> [--lease 2h]\n\
            \x20 abandon  --item <id> --holder <name> --reason <text>\n\
            \x20 validate\n\
            \x20 audit\n\
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
        let _ = writeln!(output, "{:<10} {:<9}{holder}  {}", item.id, label, item.title);
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
