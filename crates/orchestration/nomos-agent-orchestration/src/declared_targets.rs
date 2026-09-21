//! The dispatch targets this build declares, as the values a resolution reads.
//!
//! `OD-PACKAGE-016` decision 2 decided the resolver reads a caller-supplied declared package
//! set rather than discovering one, because no manifest file exists anywhere in this tree and
//! reading files is a composition root's concern. It also said writing that file and pointing
//! a host at it is a later item. This is what a host supplies until that item exists: not a
//! manifest, and not a discovery, but a declaration of what this binary ships.
//!
//! The shape is `nomos_check_orchestration::Declared_Rules`'s, deliberately. That function
//! declares this workspace's own rules as packages, in production, with one version for the
//! whole set and one contract version at both ends of every protocol range, and its own doc
//! states the authority: these are not independently released packages, they are this
//! workspace's own, declared in one place and shipped in one binary. The two backends here
//! are in exactly that position, so they are declared the same way rather than a second way.

use nomos_contracts::{ContractVersion, PackageId, PackageKind};
use nomos_model_package::{ModelRoutePackage, ModelSelection, PackageVersion, ProtocolRange};

use crate::{Backend, DeclaredTarget};

/// The package version every declaration below carries.
///
/// One version for the whole set, for the reason `Declared_Rules` gives for its own: these
/// are not independently released packages. They are this workspace's two shipped adapters,
/// declared here and built into the same binary as the resolver that reads them. A real
/// per-package version belongs to a package a repository installs, which is what
/// `nomos_model_package::Read_Manifest` is for and this function deliberately is not.
const DECLARED_AT: PackageVersion = PackageVersion::New(1, 0, 0);

/// The contract version every declaration below states at both ends of its protocol range.
///
/// Both ends, and the same version, because these declarations describe what this build
/// itself speaks. A range wider than one version would claim a compatibility nobody measured.
const DECLARED_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// Every dispatch target this build declares, in [`Backend`]'s own order.
///
/// Derived from [`Backend`] rather than listed beside it, so a third backend cannot be added
/// without appearing here: `Backend::ALL` is the one place the set is written, and a variant
/// missing a label is already a compile error in `Backend::Label`.
#[must_use]
pub fn Declared_Targets() -> Vec<DeclaredTarget>
{
    return Backend::ALL.iter().map(|backend| return Declared_Target(*backend)).collect();
}

/// One backend as the declaration a resolution reads.
///
/// `package_kind` is measured rather than defaulted, and the two differ: `OD-PACKAGE-013`
/// found the two backends are different kinds of thing, which is why the CLI refuses
/// `--executor ollama` and `--model-backend claude-code`. Claude Code is an agent executor;
/// Ollama is a model inference backend.
///
/// `model_selection` is the field that would be a fabrication if it were guessed, and neither
/// value is guessed. Claude Code chooses which model answers a request, which is
/// [`ModelSelection::ExecutorControlled`] in as many words. The Ollama adapter runs one turn
/// against a locally hosted program that carries its own configured model and names none in
/// this workspace, so the package genuinely does not expose which model answers, which is
/// [`ModelSelection::Opaque`]. A `Catalog` for either would be a claim about the outside
/// world that nothing here measured.
fn Declared_Target(backend: Backend) -> DeclaredTarget
{
    return DeclaredTarget {
        backend,
        package: ModelRoutePackage {
            package_id: PackageId::New(format!("nomos.model.{}", backend.Label())),
            package_kind: Kind_Of(backend),
            package_version: DECLARED_AT,
            protocol_range: ProtocolRange::New(DECLARED_VERSION, DECLARED_VERSION),
            model_selection: Selection_Of(backend),
        },
    };
}

/// Which kind of package each backend is.
const fn Kind_Of(backend: Backend) -> PackageKind
{
    return match backend
    {
        Backend::ClaudeCode => PackageKind::AgentExecutorPackage,
        Backend::Ollama => PackageKind::ModelBackendPackage,
    };
}

/// What each backend exposes about the model that answers.
const fn Selection_Of(backend: Backend) -> ModelSelection
{
    return match backend
    {
        Backend::ClaudeCode => ModelSelection::ExecutorControlled,
        Backend::Ollama => ModelSelection::Opaque,
    };
}
