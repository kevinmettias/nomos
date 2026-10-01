//! This crate as the `AgentExecutor` port a generic path names, and as the target a
//! composition root declares.

use std::path::Path;

use nomos_agent_contracts::{
    AgentExecution, AgentExecutor, DeclaredTarget, DispatchPort, DispatchRefusal, TaskEnvelope,
};
use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_model_package::{ModelRoutePackage, ModelSelection, PackageVersion, ProtocolRange};
use nomos_platform::ProgramLauncher;

use crate::Execute_Task;

/// The family label a profile names this executor by, and the spelling `--executor` accepts.
///
/// Published rather than restated at every site that needs it, so the string a person types,
/// the string a flag parser validates and the string a resolution matches have one origin.
/// `nomos-cli`'s own parser and `nomos_agent_contracts::DeclaredTarget::family` both read
/// this.
pub const FAMILY: &str = "claude-code";

/// The package version this declaration carries.
///
/// One version, because this is not an independently released package: it is an adapter this
/// workspace ships in the same binary as the composition root that declares it. A real
/// per-package version belongs to a package a repository installs, which is what
/// `nomos_model_package::Read_Manifest` is for and this deliberately is not.
const DECLARED_AT: PackageVersion = PackageVersion::New(1, 0, 0);

/// The contract version this declaration states at both ends of its protocol range.
///
/// Both ends, and the same version, because the declaration describes what this build itself
/// speaks. A wider range would claim a compatibility nobody measured.
const DECLARED_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// This crate's dispatch, bound to the launcher a composition root chose.
///
/// The port is not generic over a launcher and this type is: binding the platform is the
/// composition root's act (`OD-HOST-002`), so the generic path receives something already
/// bound and never names a launcher to dispatch an agent task. That is what retired
/// `nomos_agent_orchestration::AgentEnvironment`, whose one field was a launcher the seam
/// accepted but did not choose.
pub struct ClaudeCodeExecutor<'launcher, Launcher: ProgramLauncher>
{
    launcher: &'launcher Launcher,
}

impl<'launcher, Launcher: ProgramLauncher> ClaudeCodeExecutor<'launcher, Launcher>
{
    /// This executor, dispatching through `launcher`.
    #[must_use]
    pub const fn Through(launcher: &'launcher Launcher) -> Self
    {
        return Self { launcher };
    }

    /// This executor as the target a composition root declares.
    ///
    /// The declaration is this crate's own rather than a table in the composition root,
    /// which is the shape `nomos_capability::Registry` already uses -- a root calls
    /// `registry.Offer(nomos_lang_rust::Provider_Offer())` rather than restating what the
    /// provider knows about itself. So two hosts declaring this target are two calls, not
    /// two copies of a package declaration that could drift apart.
    ///
    /// `package_kind` is measured rather than defaulted: `OD-PACKAGE-013` found this crate
    /// is an `AgentExecutorPackage` and Ollama's a `ModelBackendPackage`, which is why the
    /// CLI refuses `--executor ollama`. `model_selection` is
    /// [`ModelSelection::ExecutorControlled`] because Claude Code chooses which model
    /// answers a request, in as many words; a `Catalog` would be a claim about the outside
    /// world that nothing here measured.
    #[must_use]
    pub fn Declared_Target(&self) -> DeclaredTarget<'_>
    {
        return DeclaredTarget {
            family: FAMILY.to_owned(),
            package: ModelRoutePackage {
                package_id: PackageId::New(format!("nomos.model.{FAMILY}")),
                package_kind: PackageKind::AgentExecutorPackage,
                package_version: DECLARED_AT,
                protocol_range: ProtocolRange::New(DECLARED_VERSION, DECLARED_VERSION),
                model_selection: ModelSelection::ExecutorControlled,
            },
            port: DispatchPort::Executor(self),
        };
    }
}

/// The port, over this crate's own [`Execute_Task`].
///
/// Folds this crate's own [`crate::AgentExecutionError`] down to the text its `Display`
/// already produces, which is the fold the generic path already made before a port existed.
/// The structured error stays on this crate's public surface for a caller that wants to
/// branch on which side of the process boundary a failure was on.
impl<Launcher: ProgramLauncher> AgentExecutor for ClaudeCodeExecutor<'_, Launcher>
{
    fn Execute(&self, task: &TaskEnvelope, root: &Path) -> Result<AgentExecution, DispatchRefusal>
    {
        return match Execute_Task(task, self.launcher, root)
        {
            Ok(outcome) => Ok(AgentExecution {
                result: outcome.result,
                denied_tool_uses: outcome.denied_tool_uses,
                is_error: outcome.is_error,
                spend: outcome.cost,
                duration_ms: outcome.duration_ms,
            }),
            Err(error) => Err(DispatchRefusal::Of(error.to_string())),
        };
    }
}
