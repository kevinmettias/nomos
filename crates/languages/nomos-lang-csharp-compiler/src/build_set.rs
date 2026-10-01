//! Answering for a set of builds at once: every build asked, then every file judged under the
//! builds that compile it -- or no file judged, when any build could not be asked.

use crate::{Evaluate_Build, EvaluationError, EvaluationRequest, FactContext, Materialization, Materialize_Conditional_Fact, ParseFailure};
use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;
use nomos_platform::{Environment, ProgramLauncher};

/// One file a caller wants judged: its path relative to the repository root with forward
/// slashes, its own subject, and its text.
#[derive(Clone, Copy, Debug)]
pub struct FileToJudge<'text>
{
    pub path: &'text str,
    pub subject: SubjectId,
    pub text: &'text str,
}

/// One file's fact under one build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgedFile
{
    /// The file's path, as the caller gave it.
    pub path: String,
    /// The fact, filed under the subject of that file under that build.
    pub fact: MaterializedFact,
}

/// Something in a build set that was not judged, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unjudged
{
    /// A build could not be evaluated -- and because of it, no file was judged under any build.
    Build { request: EvaluationRequest, error: EvaluationError },
    /// A file's directives could not be read under a build that compiles it, so it has no fact
    /// under any build.
    File { path: String, failure: ParseFailure },
}

/// What answering a set of builds produced.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildSetAnswer
{
    /// One fact per file per build that compiles it.
    pub facts: Vec<JudgedFile>,
    /// Everything not judged.
    pub unjudged: Vec<Unjudged>,
}

/// The ports a build set is asked through: a launcher for `dotnet`, and the environment it is
/// found and resolved in.
pub struct BuildPorts<'ports, Launcher: ProgramLauncher, Env: Environment>
{
    pub launcher: &'ports Launcher,
    pub environment: &'ports Env,
}

/// Every build in `requests` asked of `MSBuild`, then each of `files` judged under every build
/// that compiles it.
///
/// # All builds or none
///
/// If any build cannot be evaluated, no file is judged under any build, and each failure is
/// reported. A question asked of a set of builds -- is this branch compiled by any of them? --
/// needs every build's answer: the build that could not be asked may be the one that compiles
/// the branch the others skip, and a caller holding the others' facts would read that branch as
/// compiled by nothing. The same holds per file: a file refused under one build gets no fact under
/// any, so no file is ever answered for part of the builds that compile it.
///
/// A file no build compiles gets no fact. It is judged under the builds that compile it, which
/// are none; `MSBuild`'s `Compile` items decide that, never the directory the file sits in.
#[must_use]
pub fn Materialize_Build_Set<Launcher: ProgramLauncher, Env: Environment>(
    requests: &[EvaluationRequest],
    files: &[FileToJudge<'_>],
    context: FactContext,
    ports: &BuildPorts<'_, Launcher, Env>,
) -> BuildSetAnswer
{
    let mut evaluations = Vec::new();
    let mut unjudged = Vec::new();
    for request in requests
    {
        match Evaluate_Build(request, ports.launcher, ports.environment)
        {
            Ok(evaluation) => evaluations.push(evaluation),
            Err(error) => unjudged.push(Unjudged::Build { request: request.clone(), error }),
        }
    }
    if !unjudged.is_empty()
    {
        return BuildSetAnswer { facts: Vec::new(), unjudged };
    }

    let mut facts = Vec::new();
    for file in files
    {
        let compiling = evaluations.iter().filter(|evaluation| return evaluation.Compiles(file.path));
        let judged: Result<Vec<JudgedFile>, ParseFailure> = compiling.map(|evaluation| return Judged(file, &evaluation.definitions, context)).collect();
        match judged
        {
            Ok(judged) => facts.extend(judged),
            Err(failure) => unjudged.push(Unjudged::File { path: file.path.to_owned(), failure }),
        }
    }

    return BuildSetAnswer { facts, unjudged };
}

fn Judged(file: &FileToJudge<'_>, definitions: &crate::DefinitionSet, context: FactContext) -> Result<JudgedFile, ParseFailure>
{
    return match Materialize_Conditional_Fact(file.subject, file.text, definitions, context)
    {
        Materialization::Materialized(fact) => Ok(JudgedFile { path: file.path.to_owned(), fact: *fact }),
        Materialization::Refused(failure) => Err(failure),
    };
}

#[cfg(test)]
mod tests;
