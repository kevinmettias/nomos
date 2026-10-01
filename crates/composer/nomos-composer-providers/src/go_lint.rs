//! The `go vet` row of `nomos.cap.lint.diagnostics`: which walked sources are Go, handed to the
//! provider, and every module it could not answer for turned into what a finding needs.

use nomos_check_orchestration::{SubjectFact, Unanswered, WalkFacts, WalkReading};
use nomos_contracts::{Applicability, GateCategory};
use nomos_lang_go_lint::{LintPorts, Unlinted};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};

/// How many stray sources a finding names before it only counts the rest.
const NAMED_STRAYS: usize = 5;

/// Every walked Go source linted, module by module, by `go vet`. A walk with no Go source asks the
/// provider about nothing, and it launches nothing.
pub(crate) fn Go_Vet_Facts<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(reading: &WalkReading<'_, Launcher, Fs, Env>) -> WalkFacts
{
    let files: Vec<&str> = reading.sources.iter().filter(|source| return super::Go_Recognizes(source.path)).map(|source| return source.path).collect();
    let ports = LintPorts { launcher: reading.launcher, filesystem: reading.filesystem, environment: reading.environment };
    let answer = nomos_lang_go_lint::Materialize_Modules(reading.root, &files, super::Go_Lint_Production(reading.context), &ports);

    return WalkFacts {
        facts: answer.facts.into_iter().map(|module| return SubjectFact { path: module.path, subject: module.subject, fact: module.fact }).collect(),
        unanswered: answer.unlinted.into_iter().map(Unanswered_Of).collect(),
    };
}

fn Unanswered_Of(unlinted: Unlinted) -> Unanswered
{
    return match unlinted
    {
        Unlinted::Module { path, failure, reason } =>
        {
            let manifest = if path.is_empty() { "go.mod".to_owned() } else { format!("{path}/go.mod") };
            Unanswered {
                reason: format!("go vet gave no answer for the Go module at {manifest}, so its lint diagnostics were not judged: {reason}"),
                subject_name: manifest,
                path,
                applicability: failure.Applicability(),
                gate: GateCategory::Advisory,
            }
        }
        Unlinted::Outside { files } =>
        {
            let first = files.first().cloned().unwrap_or_default();
            let named: Vec<&str> = files.iter().take(NAMED_STRAYS).map(String::as_str).collect();
            let more = files.len().saturating_sub(named.len());
            let rest = if more == 0 { String::new() } else { format!(" and {more} more") };
            Unanswered {
                reason: format!(
                    "{} Go source(s) sit under no go.mod -- {}{rest} -- so no module holds them and go vet has no package to load them in",
                    files.len(),
                    named.join(", ")
                ),
                subject_name: first.clone(),
                path: first,
                applicability: Applicability::NotApplicable,
                gate: GateCategory::Advisory,
            }
        }
    };
}
