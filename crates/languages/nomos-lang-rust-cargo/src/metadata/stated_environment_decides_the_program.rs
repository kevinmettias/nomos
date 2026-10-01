//! What the environment port decides, exercised -- the program a launcher is handed comes
//! from the `Environment` a caller supplied, not from this process's own ambient state.

use super::Discover_Workspace;
use nomos_platform::{Command, DeterminismStrength, Environment, EnvironmentError, ProgramLauncher, ReproducibilityScope, Strategy, TraceEquivalence};
use std::cell::RefCell;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The program name a test states, rather than the one this process happens to be
/// standing in.
struct Stated
{
    cargo: Option<&'static str>,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Stated
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for Stated
{
    fn Variable(&self, name: &str) -> Option<OsString>
    {
        if name != "CARGO"
        {
            return None;
        }

        return self.cargo.map(OsString::from);
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        return Ok(PathBuf::from("."));
    }
}

/// A launcher that records the command it was handed and never runs anything.
struct Recording
{
    seen: RefCell<Vec<Command>>,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Recording
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for Recording
{
    fn Run(&self, command: &Command) -> Result<nomos_platform::ProgramOutput, String>
    {
        self.seen.borrow_mut().push(command.clone());

        return Err("this launcher only records".to_owned());
    }
}

/// The defect `P87` closed, stated as the property it restores: a test can now say which
/// `cargo` a dispatch would run.
///
/// Before the port, this function read `CARGO` from `std::env` while building a command
/// for an *injected* launcher, so the fake launcher below received the program already
/// chosen and nothing could state it. Watched failing against the `std::env` read first —
/// under it the command names whatever this process was launched by, never `stated-cargo`.
#[test]
fn Test_A_Stated_Cargo_Should_Be_The_Program_The_Launcher_Is_Handed()
{
    let launcher = Recording { seen: RefCell::new(Vec::new()) };
    let environment = Stated { cargo: Some("stated-cargo") };

    let _refused = Discover_Workspace(Path::new("."), &launcher, &environment);

    let seen = launcher.seen.borrow();
    let command = seen.first().expect("the launcher was handed a command before it refused");
    assert_eq!(command.argv.first().map(String::as_str), Some("stated-cargo"));
}

/// And an environment naming no `CARGO` falls back to the plain program, rather than to
/// whatever this process inherited.
#[test]
fn Test_An_Unset_Cargo_Should_Fall_Back_To_The_Plain_Program_Name()
{
    let launcher = Recording { seen: RefCell::new(Vec::new()) };
    let environment = Stated { cargo: None };

    let _refused = Discover_Workspace(Path::new("."), &launcher, &environment);

    let seen = launcher.seen.borrow();
    let command = seen.first().expect("the launcher was handed a command before it refused");
    assert_eq!(command.argv.first().map(String::as_str), Some("cargo"));
}