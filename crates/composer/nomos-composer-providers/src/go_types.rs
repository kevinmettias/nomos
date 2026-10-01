//! The row of `nomos.cap.go.discarded_values`: which walked sources are Go, handed to the provider,
//! and everything it could not type turned into what a finding needs.

use nomos_check_orchestration::{SubjectFact, Unanswered, WalkFacts, WalkReading};
use nomos_contracts::{Applicability, GateCategory};
use nomos_lang_go_types::{TypesPorts, Untyped};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};

/// How many sources a finding names before it only counts the rest.
const NAMED_SOURCES: usize = 5;

/// Every walked Go source typed, module by module, by the Go types helper. A walk with no Go source
/// asks the provider about nothing, and it writes and launches nothing.
pub(crate) fn Go_Types_Facts<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(reading: &WalkReading<'_, Launcher, Fs, Env>) -> WalkFacts
{
    let files: Vec<&str> = reading.sources.iter().filter(|source| return super::Go_Recognizes(source.path)).map(|source| return source.path).collect();
    let ports = TypesPorts { launcher: reading.launcher, filesystem: reading.filesystem, environment: reading.environment };
    let answer = nomos_lang_go_types::Materialize_Files(reading.root, &files, super::Go_Types_Production(reading.context), &ports);

    return WalkFacts {
        facts: answer.facts.into_iter().map(|file| return SubjectFact { path: file.path, subject: file.subject, fact: file.fact }).collect(),
        unanswered: answer.untyped.into_iter().map(Unanswered_Of).collect(),
    };
}

fn Unanswered_Of(untyped: Untyped) -> Unanswered
{
    return match untyped
    {
        Untyped::Module { path, failure, reason } =>
        {
            let manifest = Manifest(&path);
            Unanswered {
                reason: format!("the Go type checker gave no answer for the module at {manifest}, so the errors its files discard were not judged: {reason}"),
                subject_name: manifest,
                path,
                applicability: failure.Applicability(),
                gate: GateCategory::Advisory,
            }
        }
        Untyped::Package { package, files, reason } =>
        {
            let first = files.first().cloned().unwrap_or_default();
            Unanswered {
                reason: format!("package {package} does not type-check, so the errors its files discard were not judged -- {}: {reason}", Named(&files)),
                subject_name: first.clone(),
                path: first,
                applicability: Applicability::AnalysisFailed,
                gate: GateCategory::Advisory,
            }
        }
        Untyped::Unchecked { module, files } =>
        {
            let first = files.first().cloned().unwrap_or_default();
            Unanswered {
                reason: format!(
                    "{} Go source(s) in the module at {} are compiled by no package on this host -- a build constraint excludes them, go does not look in their directory, or they are cgo sources -- so the errors they discard were not judged: {}",
                    files.len(),
                    Manifest(&module),
                    Named(&files)
                ),
                subject_name: first.clone(),
                path: first,
                applicability: Applicability::NotApplicable,
                gate: GateCategory::Advisory,
            }
        }
        Untyped::Outside { files } =>
        {
            let first = files.first().cloned().unwrap_or_default();
            Unanswered {
                reason: format!("{} Go source(s) sit under no go.mod -- {} -- so no module holds them and go has no package to type them in", files.len(), Named(&files)),
                subject_name: first.clone(),
                path: first,
                applicability: Applicability::NotApplicable,
                gate: GateCategory::Advisory,
            }
        }
    };
}

/// The `go.mod` of the module at `directory`, relative to the root.
fn Manifest(directory: &str) -> String
{
    return if directory.is_empty() { "go.mod".to_owned() } else { format!("{directory}/go.mod") };
}

/// The first few of `files`, and a count of the rest.
fn Named(files: &[String]) -> String
{
    let named: Vec<&str> = files.iter().take(NAMED_SOURCES).map(String::as_str).collect();
    let more = files.len().saturating_sub(named.len());
    let rest = if more == 0 { String::new() } else { format!(" and {more} more") };
    return format!("{}{rest}", named.join(", "));
}
