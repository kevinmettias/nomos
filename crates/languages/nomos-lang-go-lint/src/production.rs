//! Linting a set of Go sources: each module that holds one vetted once, and one fact per module.

use crate::modules::{Group_By_Module, Module_Path};
use crate::vet::Vet_Module;
use crate::{Declared_Guarantee, FactContext, LintAnswer, LintPorts, ModuleFact, Unlinted, VetFailure, PROVIDER};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_lint::{Capability, DiagnosticsPayload, Encode_Payload, LintDiagnostic, Payload_Schema, CONTRACT_VERSION};
use nomos_contracts::{EvidenceClass, ProviderId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use std::path::{Path, PathBuf};

/// Lints every Go module holding one of `files` -- paths relative to `root` with forward slashes --
/// with one `go vet` per module, and answers one fact per module vetted.
///
/// Nothing is launched when `files` is empty, so a repository with no Go source costs nothing and
/// reports nothing. A module `go vet` gives no answer for is [`Unlinted::Module`] with the failure
/// sorted, never a fact with no diagnostic in it: a tool that failed is not a tool that found
/// nothing. Sources no module holds are [`Unlinted::Outside`], once for all of them.
///
/// Deliberately not a store writer, the division every provider here draws: the caller files each
/// fact into the store it owns.
#[must_use]
pub fn Materialize_Modules<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    root: &Path,
    files: &[&str],
    context: FactContext,
    ports: &LintPorts<'_, Launcher, Fs, Env>,
) -> LintAnswer
{
    let mut answer = LintAnswer::default();
    if files.is_empty()
    {
        return answer;
    }

    let modules = Group_By_Module(root, files, ports.filesystem);
    if !modules.outside.is_empty()
    {
        answer.unlinted.push(Unlinted::Outside { files: modules.outside });
    }
    if modules.holding.is_empty()
    {
        return answer;
    }

    let absolute = match Absolute_Root(root, ports.environment)
    {
        Ok(absolute) => absolute,
        Err(reason) =>
        {
            answer.unlinted.extend(modules.holding.into_keys().map(|path| return Unlinted::Module { path, failure: VetFailure::Unreadable, reason: reason.clone() }));
            return answer;
        }
    };
    for directory in modules.holding.into_keys()
    {
        match Vet_Module(&absolute, &absolute.join(&directory), ports.launcher, ports.environment)
        {
            Ok(diagnostics) =>
            {
                let package = Module_Path(root, &directory, ports.filesystem).unwrap_or_else(|| return directory.clone());
                answer.facts.push(Module_Fact(directory, package, diagnostics, context));
            }
            Err(error) => answer.unlinted.push(Unlinted::Module { path: directory, failure: error.failure, reason: error.reason }),
        }
    }

    return answer;
}

fn Module_Fact(path: String, package: String, diagnostics: Vec<LintDiagnostic>, context: FactContext) -> ModuleFact
{
    let subject = nomos_model::Subject_Of_Path(&path);
    let guarantee = Declared_Guarantee();
    let payload = Encode_Payload(&DiagnosticsPayload { package, diagnostics });
    let key = FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        // Empty, for the reason every subprocess-backed provider here gives: the real input is
        // `go vet`'s own analysis of a module, which no reader holds to rebuild a key from.
        semantic_inputs: InputDigest::Of(&[]),
        provider: ProviderId::New(PROVIDER),
        provider_version: CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(Payload_Schema(), payload),
    };

    return ModuleFact { path, subject, fact };
}

/// `root` as an absolute path, resolved against the injected working directory when relative --
/// `go vet` reports absolute positions, and they are made relative to this.
fn Absolute_Root<Env: Environment>(root: &Path, environment: &Env) -> Result<PathBuf, String>
{
    if root.is_absolute()
    {
        return Ok(root.to_path_buf());
    }

    return environment
        .Working_Directory()
        .map(|working| return working.join(root))
        .map_err(|error| return format!("the working directory a relative root is resolved against could not be read: {error}"));
}
