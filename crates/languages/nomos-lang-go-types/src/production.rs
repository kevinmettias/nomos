//! Typing a set of Go sources: the helper run once in each module that holds one, and one fact per
//! source it checked.

use crate::helper::Install_Helper;
use crate::modules::Group_By_Module;
use crate::typecheck::{ModuleAnswer, TypesError, Type_Module};
use crate::{Declared_Guarantee, FactContext, FileFact, TypesAnswer, TypesFailure, TypesPorts, Untyped, PROVIDER};
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_cap_go_types::{Capability, DiscardedValue, DiscardedValuesPayload, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_contracts::{EvidenceClass, ProviderId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Types every Go module holding one of `files` -- paths relative to `root` with forward slashes --
/// with one run of the helper per module, and answers one fact per source it checked.
///
/// Nothing is written or launched when `files` is empty, so a repository with no Go source costs
/// nothing and reports nothing. A module the helper gives no answer for is [`Untyped::Module`] with
/// the failure sorted, never a set of empty facts: a tool that failed is not a file that discards
/// nothing. Within a module that answered, a source in a package that did not type-check is
/// [`Untyped::Package`], and one no compiled package holds is [`Untyped::Unchecked`]. Sources no
/// module holds are [`Untyped::Outside`], once for all of them.
///
/// Deliberately not a store writer, the division every provider here draws: the caller files each
/// fact into the store it owns.
#[must_use]
pub fn Materialize_Files<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    root: &Path,
    files: &[&str],
    context: FactContext,
    ports: &TypesPorts<'_, Launcher, Fs, Env>,
) -> TypesAnswer
{
    let mut answer = TypesAnswer::default();
    if files.is_empty()
    {
        return answer;
    }

    let modules = Group_By_Module(root, files, ports.filesystem);
    if !modules.outside.is_empty()
    {
        answer.untyped.push(Untyped::Outside { files: modules.outside });
    }
    if modules.holding.is_empty()
    {
        return answer;
    }

    let prepared = Absolute_Root(root, ports.environment).and_then(|absolute| return Install_Helper(ports.filesystem, ports.environment).map(|helper| return (absolute, helper)));
    let (absolute, helper) = match prepared
    {
        Ok(prepared) => prepared,
        Err(error) =>
        {
            answer.untyped.extend(modules.holding.into_keys().map(|path| return Untyped::Module { path, failure: error.failure, reason: error.reason.clone() }));
            return answer;
        }
    };
    for (directory, asked) in modules.holding
    {
        match Type_Module(&helper, &absolute, &absolute.join(&directory), ports)
        {
            Ok(module) =>
            {
                let module_answer = File_Module(directory, &asked, &module, context);
                answer.facts.extend(module_answer.facts);
                answer.untyped.extend(module_answer.untyped);
            }
            Err(error) => answer.untyped.push(Untyped::Module { path: directory, failure: error.failure, reason: error.reason }),
        }
    }

    return answer;
}

/// One module's answer filed: a fact for each source asked about that the helper checked, and each
/// other source reported under the package that failed on it or as unchecked.
fn File_Module(module: String, asked: &[String], typed: &ModuleAnswer, context: FactContext) -> TypesAnswer
{
    let mut answer = TypesAnswer::default();
    let mut unchecked = Vec::new();
    let mut failed: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for file in asked
    {
        if let Some(values) = typed.checked.get(file)
        {
            answer.facts.push(File_Fact(file.clone(), values.clone(), context));
            continue;
        }
        match typed.failed.iter().position(|package| return package.files.contains(file))
        {
            Some(index) => failed.entry(index).or_default().push(file.clone()),
            None => unchecked.push(file.clone()),
        }
    }

    for (index, files) in failed
    {
        if let Some(package) = typed.failed.get(index)
        {
            answer.untyped.push(Untyped::Package { package: package.package.clone(), files, reason: package.reason.clone() });
        }
    }
    if !unchecked.is_empty()
    {
        answer.untyped.push(Untyped::Unchecked { module, files: unchecked });
    }

    return answer;
}

fn File_Fact(path: String, values: Vec<DiscardedValue>, context: FactContext) -> FileFact
{
    let subject = nomos_model::Subject_Of_Path(&path);
    let guarantee = Declared_Guarantee();
    let payload = Encode_Payload(&DiscardedValuesPayload { values });
    let key = FactKey {
        contract: Capability(),
        contract_version: CONTRACT_VERSION,
        subject,
        // Empty, for the reason every subprocess-backed provider here gives: the real input is the
        // type checker's reading of the whole package, which no reader holds to rebuild a key from.
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

    return FileFact { path, subject, fact };
}

/// `root` as an absolute path, resolved against the injected working directory when relative -- the
/// helper reports absolute paths, and they are made relative to this.
fn Absolute_Root<Env: Environment>(root: &Path, environment: &Env) -> Result<PathBuf, TypesError>
{
    if root.is_absolute()
    {
        return Ok(root.to_path_buf());
    }

    return environment.Working_Directory().map(|working| return working.join(root)).map_err(|error| {
        return TypesError::New(TypesFailure::Unreadable, &format!("the working directory a relative root is resolved against could not be read: {error}"));
    });
}
