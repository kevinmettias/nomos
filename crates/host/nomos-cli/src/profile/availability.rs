//! What this host establishes about answering each declared capability, short of running
//! anything.
//!
//! The registry settles two things outright, and nothing here re-decides them: which
//! capabilities `nomos_check_orchestration::Registered` declares, and which providers offer
//! each. What it settles nothing about is the machine — an offer is a statement about a
//! fact's strength, not about whether the program that produces it is installed here. So
//! this module asks one further question per offer, [`super::Need_Of`]'s, and then looks
//! for whatever program that answer names.
//!
//! # Where the line between determined and undetermined is drawn
//!
//! Looking along the search path is a filesystem read, not an execution, so "no directory
//! on `PATH` holds `cargo`" is established here and the report says so with the tool named.
//! The converse is not established: `cargo` being on `PATH` says nothing about whether
//! `cargo clippy` or `cargo deny` is installed, whether the toolchain carries the
//! component, or whether the subcommand would refuse — and the only thing that settles any
//! of those is running them. This verb does not, so it reports undetermined and says why.
//!
//! That asymmetry is the whole point of the module. A verb that ran the tools to find out
//! would be a slow first-run command whose answer was a side effect of a build; a verb that
//! assumed from a program's presence would be handing out the silent incompleteness a
//! person is here to avoid.

use super::{CapabilityStanding, Need_Of, OfferStanding, ProgramSearch, ProviderNeed, ProviderStanding};
use nomos_capability::{CapabilityContract, Registry};
use nomos_platform::{Environment, FileSystem};
use std::path::Path;

/// The variable naming the directories a program is looked for in. The same name on every
/// platform this workspace runs on, which is why it is one literal rather than a choice.
const SEARCH_PATH_VARIABLE: &str = "PATH";

/// The variable naming the suffixes an executable may carry on a host that uses them.
///
/// Read rather than branched on the compilation target: a host that spells its executables
/// with a suffix sets this, and one that does not leaves it unset, so consulting it gives
/// the right answer on both without this module deciding which host it is on.
const EXECUTABLE_SUFFIX_VARIABLE: &str = "PATHEXT";

/// How the suffix variable separates its entries.
const SUFFIX_SEPARATOR: char = ';';

/// Every declared capability, with what this host establishes about answering it, ordered
/// by capability identity so two runs over one host render the same report.
pub(crate) fn Capability_Standings<Env: Environment, Fs: FileSystem>(
    registry: &Registry,
    environment: &Env,
    filesystem: &Fs,
) -> Vec<CapabilityStanding>
{
    let mut standings: Vec<CapabilityStanding> = registry
        .Declared()
        .map(|contract| return Standing_Of_Capability(registry, contract, environment, filesystem))
        .collect();

    standings.sort_by(|left, right| return left.capability.cmp(&right.capability));
    return standings;
}

/// One capability, with a row per offer standing against it.
fn Standing_Of_Capability<Env: Environment, Fs: FileSystem>(
    registry: &Registry,
    contract: &CapabilityContract,
    environment: &Env,
    filesystem: &Fs,
) -> CapabilityStanding
{
    let offers = registry
        .Offers(&contract.id)
        .iter()
        .map(|offer| {
            return OfferStanding {
                provider: offer.provider.As_Str().to_owned(),
                standing: Standing_Of_Provider(offer.provider.As_Str(), environment, filesystem),
            };
        })
        .collect();

    return CapabilityStanding { capability: contract.id.As_Str().to_owned(), offers };
}

/// What this host establishes about one provider.
///
/// Reachable from this module's siblings so the conservative default -- a provider nothing
/// declares a need for reads as undetermined rather than as available -- can be driven
/// directly by a test with a name nothing else offers, rather than inferred from a capability
/// row that would need a provider to be added to the real registry first.
pub(super) fn Standing_Of_Provider<Env: Environment, Fs: FileSystem>(
    provider: &str,
    environment: &Env,
    filesystem: &Fs,
) -> ProviderStanding
{
    return match Need_Of(provider)
    {
        Some(ProviderNeed::InThisBinary) => ProviderStanding::InThisBinary,
        Some(ProviderNeed::HostProgram { variable, fallback }) =>
        {
            Standing_Of_Program(&Program_Named(variable, fallback, environment), environment, filesystem)
        }
        None => ProviderStanding::Undetermined {
            because: format!("this build carries no declaration of what `{provider}` needs from a host"),
        },
    };
}

/// What this host establishes about a provider that runs `program`.
fn Standing_Of_Program<Env: Environment, Fs: FileSystem>(program: &str, environment: &Env, filesystem: &Fs) -> ProviderStanding
{
    return match Search_For(program, environment, filesystem)
    {
        ProgramSearch::Found => ProviderStanding::Undetermined {
            because: format!(
                "`{program}` is on this host's search path, and whether it answers for this \
                 capability was not established, because establishing it means running it"
            ),
        },
        ProgramSearch::NotOnPath => ProviderStanding::ToolMissing { tool: program.to_owned() },
        ProgramSearch::PathUnreadable => ProviderStanding::Undetermined {
            because: format!("this process has no search path, so `{program}` could not be looked for"),
        },
    };
}

/// The program a provider would actually launch: what its variable says, and otherwise the
/// name the provider itself falls back to.
fn Program_Named<Env: Environment>(variable: Option<&str>, fallback: &str, environment: &Env) -> String
{
    return variable
        .and_then(|name| return environment.Variable(name))
        .and_then(|value| return value.into_string().ok())
        .unwrap_or_else(|| return fallback.to_owned());
}

/// Whether any directory on the search path holds `program`.
fn Search_For<Env: Environment, Fs: FileSystem>(program: &str, environment: &Env, filesystem: &Fs) -> ProgramSearch
{
    let Some(search_path) = environment.Variable(SEARCH_PATH_VARIABLE)
    else
    {
        return ProgramSearch::PathUnreadable;
    };

    let suffixes = Executable_Suffixes(environment);

    for directory in std::env::split_paths(&search_path)
    {
        if Holds_Program(&directory, program, &suffixes, filesystem)
        {
            return ProgramSearch::Found;
        }
    }

    return ProgramSearch::NotOnPath;
}

/// Whether `directory` holds `program`, under its bare name or under any suffix this host
/// spells an executable with.
fn Holds_Program<Fs: FileSystem>(directory: &Path, program: &str, suffixes: &[String], filesystem: &Fs) -> bool
{
    if filesystem.Exists(&directory.join(program))
    {
        return true;
    }

    return suffixes
        .iter()
        .any(|suffix| return filesystem.Exists(&directory.join(format!("{program}{suffix}"))));
}

/// Every suffix this host spells an executable with, or none when it spells them bare.
fn Executable_Suffixes<Env: Environment>(environment: &Env) -> Vec<String>
{
    return environment
        .Variable(EXECUTABLE_SUFFIX_VARIABLE)
        .and_then(|value| return value.into_string().ok())
        .map(|value| return Split_Suffixes(&value))
        .unwrap_or_default();
}

/// The non-empty entries of a suffix list.
fn Split_Suffixes(value: &str) -> Vec<String>
{
    return value
        .split(SUFFIX_SEPARATOR)
        .filter(|suffix| return !suffix.is_empty())
        .map(str::to_owned)
        .collect();
}
