//! The real platform a caller chose, grouped into one value a workflow run threads through.

use nomos_platform::{Environment, FileSystem, ProgramLauncher, Timestamp};

/// The real `ProgramLauncher` and `FileSystem` a caller chose, grouped into one value so
/// [`crate::Run`] and [`super::Dispatch_Body`] each take a platform as one parameter rather
/// than two -- this crate's own version of the identical grouping
/// `nomos_check_orchestration::MaterializationEnvironment` and
/// `nomos_gate_orchestration::GateEnvironment` already use for the same two values.
pub struct Platform<'a, Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>
{
    pub launcher: &'a Launcher,
    pub filesystem: &'a Fs,
    /// Where a provider reads `CARGO` and the working directory from, rather than from this
    /// process's own ambient state. `OD-HOST-001`: the composition root chooses it.
    pub environment: &'a Env,
    /// The moment a step is judged against.
    ///
    /// The same execution fact `GateEnvironment::now` carries, riding here for the same
    /// reason: [`super::Dispatched_Gate`] is already at this workspace's four-parameter
    /// limit, and a workflow's gate step must judge a waiver's expiry against the run's own
    /// moment rather than a clock read inside policy logic.
    pub now: Timestamp,
    /// The dispatch targets an agent step may reach, each carrying the port that answers it.
    ///
    /// Supplied by the composition root rather than read here, because `OD-PACKAGE-016`
    /// decision 2 decided the resolver resolves against a caller-supplied sequence and
    /// discovers nothing, and because `OD-ROADMAP-005` decision 2 made the thing that answers
    /// part of what is supplied. A host offers each adapter's own declaration; a host holding
    /// a real manifest would supply what it read, and nothing in this crate would change.
    pub declared: &'a [nomos_agent_contracts::DeclaredTarget<'a>],
}
