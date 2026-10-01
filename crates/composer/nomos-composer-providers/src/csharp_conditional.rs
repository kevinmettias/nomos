//! The `nomos.cap.csharp.conditional_compilation` row: which builds the repository declares, which
//! walked files are C#, and every part of that population not judged, with why.
//!
//! The two questions `nomos-lang-csharp-compiler` refuses to answer for itself are answered here,
//! because they are the composition's: which builds a repository cares about is read from its own
//! declaration through `nomos-repo-policy`, never guessed; and which walked files are C# is
//! `nomos-lang-csharp`'s recognition, the same answer the syntax row gives. Which of those files
//! each build compiles is `MSBuild`'s, asked by the provider.

use nomos_check_orchestration::{WalkFacts, WalkReading, SubjectFact, Unanswered, WalkedSource};
use nomos_contracts::{Applicability, GateCategory};
use nomos_lang_csharp_compiler::{BuildPorts, EvaluationRequest, FileToJudge, Unjudged};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use nomos_repo_policy::csharp_builds::{CSHARP_BUILDS_JSON, DeclaredBuild, Read_Declared_Builds};

/// Every walked C# file judged under every declared build that compiles it, and every part of
/// that not judged.
///
/// A repository with no C# source launches nothing and reports nothing: there is nothing for a
/// build to decide. One with C# and no declared build reports that once, as not applicable -- a
/// repository that has not said which builds it ships has not failed at anything, and no build is
/// guessed for it. A declaration that cannot be read is `Blocking`: it is the repository's own
/// file saying something that will not be applied.
pub(crate) fn Csharp_Conditional_Facts<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(reading: &WalkReading<'_, Launcher, Fs, Env>) -> WalkFacts
{
    let files: Vec<FileToJudge<'_>> = reading.sources.iter().filter(|source| return super::Csharp_Recognizes(source.path)).map(File_To_Judge).collect();
    if files.is_empty()
    {
        return WalkFacts::default();
    }

    let builds = match Read_Declared_Builds(reading.root, reading.filesystem)
    {
        Ok(builds) if builds.is_empty() => return Only(Undeclared(files.len())),
        Ok(builds) => builds,
        Err(error) => return Only(Unanswered { subject_name: CSHARP_BUILDS_JSON.to_owned(), path: CSHARP_BUILDS_JSON.to_owned(), applicability: Applicability::Unparseable, gate: GateCategory::Blocking, reason: format!("{error}; no C# conditional branch is judged until it is corrected") }),
    };

    let requests: Vec<EvaluationRequest> = builds.iter().map(|build| return Request(reading, build)).collect();
    let ports = BuildPorts { launcher: reading.launcher, environment: reading.environment };
    let answer = nomos_lang_csharp_compiler::Materialize_Build_Set(&requests, &files, super::Conditional_Production(reading.context), &ports);

    return WalkFacts {
        facts: answer.facts.into_iter().map(|judged| return SubjectFact { path: judged.path, subject: judged.fact.Key().subject, fact: judged.fact }).collect(),
        unanswered: answer.unjudged.into_iter().map(Unanswered_Of).collect(),
    };
}

fn File_To_Judge<'text>(source: &WalkedSource<'text>) -> FileToJudge<'text>
{
    return FileToJudge { path: source.path, subject: source.subject, text: source.text };
}

fn Request<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(reading: &WalkReading<'_, Launcher, Fs, Env>, build: &DeclaredBuild) -> EvaluationRequest
{
    return EvaluationRequest {
        root: reading.root.to_path_buf(),
        project: build.project.clone(),
        configuration: build.configuration.clone(),
        target_framework: build.target_framework.clone(),
    };
}

fn Only(unanswered: Unanswered) -> WalkFacts
{
    return WalkFacts { facts: Vec::new(), unanswered: vec![unanswered] };
}

fn Undeclared(csharp_sources: usize) -> Unanswered
{
    return Unanswered {
        subject_name: CSHARP_BUILDS_JSON.to_owned(),
        path: CSHARP_BUILDS_JSON.to_owned(),
        applicability: Applicability::NotApplicable,
        gate: GateCategory::Advisory,
        reason: format!(
            "this repository has {csharp_sources} C# source(s) and declares no build in {CSHARP_BUILDS_JSON}, so none of their conditional branches was judged: \
             which branch compiles turns on a build's symbols, and no build is guessed"
        ),
    };
}

fn Unanswered_Of(unjudged: Unjudged) -> Unanswered
{
    return match unjudged
    {
        Unjudged::Build { request, error } =>
        {
            let framework = request.target_framework.unwrap_or_else(|| return "its one framework".to_owned());
            Unanswered {
                applicability: error.failure.Applicability(),
                gate: GateCategory::Advisory,
                reason: format!(
                    "the declared build {} {} {framework} could not be evaluated, so no C# conditional branch was judged against any declared build -- \
                     a branch only this build compiles would otherwise read as compiled by nothing: {error}",
                    request.project, request.configuration
                ),
                path: request.project.clone(),
                subject_name: request.project,
            }
        }
        Unjudged::File { path, failure } => Unanswered {
            subject_name: path.clone(),
            path,
            applicability: Applicability::Unparseable,
            gate: GateCategory::Advisory,
            reason: format!("this file's preprocessor directives could not be read, so its conditional branches were not judged against any declared build: {failure}"),
        },
    };
}
