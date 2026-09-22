//! This crate as the `ModelBackend` port a generic path names, and as the target a
//! composition root declares.

use nomos_agent_contracts::{
    DeclaredTarget, DispatchPort, DispatchRefusal, ModelAnswer, ModelBackend, TaskEnvelope,
};
use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_model_package::{ModelRoutePackage, ModelSelection, PackageVersion, ProtocolRange};
use nomos_platform::ProgramLauncher;

use crate::Execute_Task;

/// The family label a profile names this backend by, and the spelling `--model-backend`
/// accepts.
///
/// Published for the same reason its sibling's is: the string a person types, the string a
/// flag parser validates and the string a resolution matches have one origin.
pub const FAMILY: &str = "ollama";

/// The package version this declaration carries. One version, because this is an adapter
/// this workspace ships rather than an independently released package.
const DECLARED_AT: PackageVersion = PackageVersion::New(1, 0, 0);

/// The contract version this declaration states at both ends of its protocol range -- both
/// ends and the same version, because it describes what this build itself speaks.
const DECLARED_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// This crate's dispatch, bound to the launcher a composition root chose.
///
/// Structurally parallel to `nomos_agent_executor_claude_code::ClaudeCodeExecutor` and
/// implementing a different trait, which is the whole of the relationship: both are
/// subprocess dispatches bound to a launcher, and `OD-EXECUTOR-005` measured that the
/// resemblance is a property of `ProgramLauncher`-based dispatch rather than evidence the two
/// are the same kind of package.
pub struct OllamaModelBackend<'launcher, Launcher: ProgramLauncher>
{
    launcher: &'launcher Launcher,
}

impl<'launcher, Launcher: ProgramLauncher> OllamaModelBackend<'launcher, Launcher>
{
    /// This backend, dispatching through `launcher`.
    #[must_use]
    pub const fn Through(launcher: &'launcher Launcher) -> Self
    {
        return Self { launcher };
    }

    /// This backend as the target a composition root declares.
    ///
    /// `package_kind` is `ModelBackendPackage`, which is `OD-PACKAGE-013`'s own finding and
    /// the reason the CLI refuses `--model-backend claude-code`. `model_selection` is
    /// [`ModelSelection::Opaque`] because this adapter runs one turn against a locally
    /// hosted program that carries its own configured model and names none in this
    /// workspace, so the package genuinely does not expose which model answers.
    #[must_use]
    pub fn Declared_Target(&self) -> DeclaredTarget<'_>
    {
        return DeclaredTarget {
            family: FAMILY.to_owned(),
            package: ModelRoutePackage {
                package_id: PackageId::New(format!("nomos.model.{FAMILY}")),
                package_kind: PackageKind::ModelBackendPackage,
                package_version: DECLARED_AT,
                protocol_range: ProtocolRange::New(DECLARED_VERSION, DECLARED_VERSION),
                model_selection: ModelSelection::Opaque,
            },
            port: DispatchPort::Model(self),
        };
    }
}

/// The port, over this crate's own [`Execute_Task`].
///
/// Answers a [`ModelAnswer`], which carries a response and nothing else -- no spend, no
/// duration and no denial list, because this backend establishes none of them. That is the
/// honesty the two-port shape exists to keep: there is no conversion here that could supply
/// a zero for a field this mechanism never measured, because the type has no such field.
impl<Launcher: ProgramLauncher> ModelBackend for OllamaModelBackend<'_, Launcher>
{
    fn Answer(&self, task: &TaskEnvelope) -> Result<ModelAnswer, DispatchRefusal>
    {
        return match Execute_Task(task, self.launcher)
        {
            Ok(outcome) => Ok(ModelAnswer { response: outcome.response }),
            Err(error) => Err(DispatchRefusal::Of(error.to_string())),
        };
    }
}
