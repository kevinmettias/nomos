//! The two ports this binary reaches the machine through, as one value.

use nomos_platform::{Environment, ProgramLauncher};

/// A process launcher for running `git`, and the environment `--root` defaults from.
///
/// One value rather than two adjacent parameters because they are one thing — what this
/// process is allowed to reach outside itself — and because they always travel together:
/// every function below [`crate::main`] that needs either needs both, or needs neither.
/// `nomos_check_orchestration::RunContext` bundles its own root, launcher, filesystem and
/// store for the same reason.
///
/// It was the `parameter-count` rule that forced the question, when adding the environment
/// took [`crate::Run_From_String_Arguments`] to five parameters against a cap of four. The
/// answer that rule pushed toward is the better one: a third port added later joins this
/// struct instead of widening every signature between here and the composition root again.
pub(crate) struct Machine<'a, Launcher: ProgramLauncher, Surroundings: Environment>
{
    pub(crate) launcher: &'a Launcher,
    pub(crate) environment: &'a Surroundings,
}
