//! What the `--` separator carries on an `add` line: the predicate behind it, the wall
//! bound `--timeout` puts on that predicate, and the flags the split keeps out of the
//! parser.
//!
//! These five are about one mechanism rather than about any one verb's own words, which
//! is why they are read here rather than in `parsing.rs` beside the arm-by-arm cases.

use super::Added;
use super::Arguments;
use super::super::Work_Command_From_String_Arguments;

/// The predicate is everything after `--`, so a command carrying its own flags needs
/// no quoting and no escaping — and the named arguments before the separator are
/// still parsed normally.
#[test]
fn Test_Add_Should_Take_The_Predicate_After_The_Separator()
{
    let item = Added(
        concat!(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed ",
            "--territory src/a.rs ",
            "-- cargo test -p nomos-spec-model --lib"
        ),
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

/// Forty minutes, as the fixture below spells it with `--timeout 40m`; `Parse_Duration`
/// is what turns the flag into this many seconds.
const TIMEOUT_FORTY_MINUTES_SECONDS: u64 = 2_400;

/// Ten minutes: the bound `VerificationPredicate::From_String_Arguments` applies when an
/// `add` line omits `--timeout`. Restated here because the assertion is about the value a
/// caller sees, and the constructor that applies it keeps its own copy private.
const DEFAULT_PREDICATE_TIMEOUT_SECONDS: u64 = 600;

/// `--timeout` overrides the predicate's default ten-minute bound.
#[test]
fn Test_Timeout_Should_Bound_The_Predicate()
{
    let item = Added(
        concat!(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed ",
            "--territory src/a.rs --timeout 40m ",
            "-- cargo test -p nomos-workspace"
        ),
    );

    assert_eq!(
        item.verification.map(|predicate| predicate.timeout_seconds),
        Some(TIMEOUT_FORTY_MINUTES_SECONDS)
    );
}

/// Left unset, the predicate keeps the ten-minute default `VerificationPredicate::From_String_Arguments`
/// gives it — `--timeout` overrides, it does not replace, the construction.
#[test]
fn Test_Timeout_Should_Default_When_Omitted()
{
    let item = Added(
        concat!(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed ",
            "--territory src/a.rs ",
            "-- cargo test -p nomos-workspace"
        ),
    );

    assert_eq!(
        item.verification.map(|predicate| predicate.timeout_seconds),
        Some(DEFAULT_PREDICATE_TIMEOUT_SECONDS)
    );
}

/// `add` lines that name `--timeout` with no predicate after `--` to bound.
fn Timeouts_With_No_Predicate_To_Bound() -> Vec<&'static str>
{
    return vec![
        concat!(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed ",
            "--territory src/a.rs --timeout 40m"
        ),
        concat!(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed ",
            "--territory src/a.rs --timeout 2h"
        ),
        concat!(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed ",
            "--territory src/a.rs --timeout 10s"
        ),
    ];
}

/// A bound with nothing to bound is a mistake, not a no-op: an author who typed
/// `--timeout` meant to give the predicate a longer wall bound, and silently discarding it
/// because there is no predicate would leave that author believing the item is more
/// patient than it is.
#[test]
fn Test_Timeout_Without_A_Predicate_Should_Be_Refused()
{
    for arguments in Timeouts_With_No_Predicate_To_Bound()
    {
        let error = Work_Command_From_String_Arguments(&Arguments(arguments)).unwrap_err();

        assert!(
            error.contains("--timeout"),
            "the refusal must name the flag that has nothing to bound: {error}"
        );
    }
}

/// A flag that looks like a named argument but sits after the separator belongs to
/// the predicate. Without the split, `--lib` above would be read as an option to
/// `nomos work`.
#[test]
fn Test_Arguments_After_The_Separator_Should_Not_Be_Read_As_Options()
{
    let item = Added(
        concat!(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed ",
            "--territory src/a.rs ",
            "-- prog --title stolen"
        ),
    );

    assert_eq!(item.title, "t");
}
