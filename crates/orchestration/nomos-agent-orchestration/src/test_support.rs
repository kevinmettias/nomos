//! Ports and declarations this crate's own tests dispatch through.
//!
//! Every one of them is a fake, and that is the point of the port. Before `OD-ROADMAP-005`
//! decision 2 this crate named both backend crates and its tests reached the real adapters
//! through a scripted `nomos_platform::ProgramLauncher`, because a match arm calling
//! `nomos_agent_executor_claude_code::Execute_Task` had no other seam to substitute at. There
//! is a seam now, so the resolution and the dispatch are exercised against ports that answer
//! from fixed data and this crate depends on no adapter at all.
//!
//! What that gives up is the real adapters' own coverage, and it is not lost: each adapter
//! crate's own tests drive its port implementation through a scripted launcher, and
//! `nomos-cli`'s `agent_executor_claude_code_seam` and `model_backend_ollama_seam` drive each
//! real `Execute_Task` directly. The families named here are deliberately not `claude-code` or
//! `ollama`, so a test that only passes because it happened to name a shipped family is not a
//! test that could pass here.

use nomos_agent_contracts::{
    AgentExecution, AgentExecutor, DeclaredTarget, DispatchPort, DispatchRefusal, ModelAnswer,
    ModelBackend, PortionSubstantiation, Substantiation, TaskEnvelope, UnsubstantiatedReason,
    WorkResult,
};
use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_model_package::{ModelRoutePackage, ModelSelection, PackageVersion, ProtocolRange};
use std::path::Path;

/// An executor port that answers from fixed data.
pub(crate) struct ScriptedExecutor
{
    pub(crate) assumption: String,
}

impl AgentExecutor for ScriptedExecutor
{
    fn Execute(&self, _task: &TaskEnvelope, _root: &Path) -> Result<AgentExecution, DispatchRefusal>
    {
        return Ok(AgentExecution {
            result: WorkResult {
                plan: None,
                claims: Vec::new(),
                tests: Vec::new(),
                requested_verification: None,
                assumptions: vec![self.assumption.clone()],
                unresolved_questions: Vec::new(),
                substantiation: Substantiation {
                    plan: PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround),
                    claims: PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround),
                    tests: PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround),
                    requested_verification: PortionSubstantiation::Unsubstantiated(
                        UnsubstantiatedReason::ProducerCannotGround,
                    ),
                    assumptions: PortionSubstantiation::Substantiated,
                    unresolved_questions: PortionSubstantiation::Substantiated,
                },
            },
            denied_tool_uses: Vec::new(),
            is_error: false,
            spend: nomos_agent_contracts::MicroDollars::From_Micros(10_000),
            duration_ms: 500,
        });
    }
}

/// A model backend port that answers from fixed data.
pub(crate) struct ScriptedModel
{
    pub(crate) response: String,
}

impl ModelBackend for ScriptedModel
{
    fn Answer(&self, _task: &TaskEnvelope) -> Result<ModelAnswer, DispatchRefusal>
    {
        return Ok(ModelAnswer { response: self.response.clone() });
    }
}

/// An executor port that produces no execution at all.
pub(crate) struct RefusingExecutor;

impl AgentExecutor for RefusingExecutor
{
    fn Execute(&self, _task: &TaskEnvelope, _root: &Path) -> Result<AgentExecution, DispatchRefusal>
    {
        return Err(DispatchRefusal::Of("no such program"));
    }
}

/// A model backend port that produces no answer at all.
pub(crate) struct RefusingModel;

impl ModelBackend for RefusingModel
{
    fn Answer(&self, _task: &TaskEnvelope) -> Result<ModelAnswer, DispatchRefusal>
    {
        return Err(DispatchRefusal::Of("no such program"));
    }
}

/// `executor` as a declared target of `family`.
pub(crate) fn Declared_Executor<'port>(
    family: &str, executor: &'port dyn AgentExecutor,
) -> DeclaredTarget<'port>
{
    return DeclaredTarget {
        family: family.to_owned(),
        package: Package(family, PackageKind::AgentExecutorPackage),
        port: DispatchPort::Executor(executor),
    };
}

/// `model` as a declared target of `family`.
pub(crate) fn Declared_Model<'port>(family: &str, model: &'port dyn ModelBackend) -> DeclaredTarget<'port>
{
    return DeclaredTarget {
        family: family.to_owned(),
        package: Package(family, PackageKind::ModelBackendPackage),
        port: DispatchPort::Model(model),
    };
}

/// The package a declared fixture target carries -- one of the two kinds that declare a model
/// selection, which is what `ModelRoutePackage`'s own reader restricts the field to.
fn Package(family: &str, kind: PackageKind) -> ModelRoutePackage
{
    return ModelRoutePackage {
        package_id: PackageId::New(format!("fixture.{family}")),
        package_kind: kind,
        package_version: PackageVersion::New(1, 0, 0),
        protocol_range: ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0)),
        model_selection: ModelSelection::Opaque,
    };
}
