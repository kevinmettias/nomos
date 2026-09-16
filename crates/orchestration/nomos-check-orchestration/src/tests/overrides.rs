//! The policies a repository declares for itself: each of the five `standards.json` sections
//! `Run` materializes, proven to reach the rule that reads it.

use nomos_analysis::MemoryFactStore;
use nomos_contracts::{Finding, RuleId};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use std::path::Path;

use crate::{CheckOutcome, Run, RunContext};

use super::{Scratch_Directory, Source_File, SourceText, Test_Variant};

/// One declared goal nothing serves and one declared part serving no goal: the two findings
/// the goal audit returns, one per direction it checks.
const GOAL_AUDIT_DIRECTIONS: usize = 2;

/// A repository declaration a fixture writes: the scratch-root name it is written under, and
/// the `standards.json` body itself. One value, so the two cannot drift apart at a call site.
struct Standards<'a>
{
    name: &'a str,
    declaration: &'a str,
}

/// The two halves of a repository-declared policy: the findings the same sources answer under
/// a scratch root declaring nothing, and under one declaring the test's own `standards.json`.
struct PolicyAnswers
{
    unconfigured: Vec<Finding>,
    declared: Vec<Finding>,
}

/// What a policy test expects of the comparison above.
struct ExpectedAnswers<'a>
{
    /// The claim under test, quoted into every failure message.
    context: &'a str,
    /// How many findings the shipped default must answer.
    unconfigured: usize,
    /// How many findings the repository's own declaration must leave.
    declared: usize,
    /// A substring each declared finding's summary must carry, when the declaration makes a
    /// rule speak about something specific rather than merely more.
    declared_summaries: &'a [&'a str],
}

/// Judges `sources` under `selected` twice: once over a scratch root declaring nothing, once
/// over one declaring `standards`. The first is the shipped default a repository has to opt
/// out of; the second is the answer its own declaration produced.
fn Answers_With_And_Without(sources: &[SourceFile], selected: &[RuleId], standards: Standards<'_>) -> PolicyAnswers
{
    let unconfigured_root = Scratch_Directory(&format!("{}-unconfigured", standards.name));
    let declared_root = Scratch_Directory(standards.name);
    std::fs::write(declared_root.join("standards.json"), standards.declaration).expect("a scratch standards.json");

    return PolicyAnswers {
        unconfigured: Findings_Under(sources, &unconfigured_root, selected),
        declared: Findings_Under(sources, &declared_root, selected),
    };
}

/// Asserts both halves of the comparison, so a test cannot prove the declaration changed the
/// answer by asserting only one of them.
fn Assert_Answers(answers: &PolicyAnswers, expected: ExpectedAnswers<'_>)
{
    assert_eq!(
        answers.unconfigured.len(),
        expected.unconfigured,
        "{}: the shipped default must answer {} finding(s): {:?}",
        expected.context,
        expected.unconfigured,
        answers.unconfigured
    );
    assert_eq!(
        answers.declared.len(),
        expected.declared,
        "{}: the repository's own declaration must answer {} finding(s): {:?}",
        expected.context,
        expected.declared,
        answers.declared
    );
    Assert_Summaries_Carried(&answers.declared, expected.declared_summaries, expected.context);
}

/// Asserts every named substring is carried by some declared finding's summary.
fn Assert_Summaries_Carried(findings: &[Finding], summaries: &[&str], context: &str)
{
    for summary in summaries
    {
        assert!(
            findings.iter().any(|finding| return finding.summary.contains(*summary)),
            "{context}: no declared finding carries `{summary}`: {findings:?}"
        );
    }
}

/// One `Run` over `root` with `sources`, reduced to the findings it answered. Panics rather
/// than returning a non-judged outcome: every fixture here reads a real, valid source, so the
/// run reaches a judgment whatever that root's own `standards.json` declares.
fn Findings_Under(sources: &[SourceFile], root: &Path, selected: &[RuleId]) -> Vec<Finding>
{
    let outcome = Run(
        sources,
        RunContext { variant: Test_Variant(), root, launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() },
        selected,
    );

    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a tree with a readable source must be judged, whatever its standards.json declares");
    };

    return findings;
}

/// `RunContext`'s own `filesystem` reaching a real repository-declared policy fact: without a
/// real `standards.json` under `root`, `NAMING_CONVENTION` judged every function name against
/// its own hardcoded `UpperSnake` default, so an all-lowercase name is one finding; a root
/// declaring `lower-snake` is the only thing that makes it none.
#[test]
fn Test_Run_Should_Honor_A_Real_Standards_Json_Naming_Override()
{
    let answers = Answers_With_And_Without(
        &[Source_File("a.rs", SourceText("pub fn lower_snake_name() {}\n"))],
        &[RuleId::New(nomos_rules::NAMING_CONVENTION)],
        Standards { name: "naming-overridden", declaration: r#"{"naming":{"function":"lower-snake"}}"# },
    );

    Assert_Answers(&answers, ExpectedAnswers {
        context: "naming.function = \"lower-snake\" must accept a lower_snake function name",
        unconfigured: 1,
        declared: 0,
        declared_summaries: &[],
    });
}

/// The limits-policy materialization reaching `nomos_rules`' own `Resolve_Limit`, proven the
/// only way it can be proven on this repository: by declaring a threshold this workspace does
/// not.
///
/// `standards.json` here declares a hard limit of three lines, so the same five-line source
/// is far under the shipped 1500-line default and far over the declared one -- which is the
/// smallest thing that tells "the materialization ran" from "the fallback answered".
#[test]
fn Test_Run_Should_Honor_A_Real_Standards_Json_File_Size_Limit()
{
    let answers = Answers_With_And_Without(
        &[Source_File("a.rs", SourceText("fn one() {}\nfn two() {}\nfn three() {}\nfn four() {}\nfn five() {}\n"))],
        &[RuleId::New(nomos_rules::FILE_SIZE_JUSTIFICATION_TRIGGER)],
        Standards { name: "limits-overridden", declaration: r#"{"limits":{"file-size-hard-lines":3}}"# },
    );

    Assert_Answers(&answers, ExpectedAnswers {
        context: "limits.file-size-hard-lines = 3 must judge a five-line file against 3, not 1500",
        unconfigured: 0,
        declared: 1,
        declared_summaries: &[],
    });
}

/// The scripting-policy materialization reaching
/// `nomos_rules::Check_Declared_Tooling_Language_For_Scripts`.
///
/// This is the one policy capability whose absence is not a fallback: the rule resolves an
/// unreadable policy to no findings at all, because it never had a prior default to keep. So
/// the undeclared half of this comparison is not a control against a hardcoded value the way
/// the two above are -- it is the exact state every real check ran in before this wiring
/// existed, with the rule composed, selected, and structurally unable to fire.
///
/// A Rust file rides along because no syntax provider recognizes a `.sh` path, and a run whose
/// every source produced no fact reports `NoFacts` rather than `Judged` -- it would never
/// reach the rule at all. The script is still what is being judged; `a.rs` is only what makes
/// the run a judgment.
#[test]
fn Test_Run_Should_Honor_A_Real_Standards_Json_Forbidden_Script_Extension()
{
    let answers = Answers_With_And_Without(
        &[
            Source_File("a.rs", SourceText("pub fn Anything() {}\n")),
            Source_File("deploy.sh", SourceText("#!/usr/bin/env bash\necho deploying\n")),
        ],
        &[RuleId::New(nomos_rules::DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS)],
        Standards { name: "scripting-overridden", declaration: r#"{"scripting":{"tooling_language":"rust","forbidden_extensions":[".sh"]}}"# },
    );

    Assert_Answers(&answers, ExpectedAnswers {
        context: "a repository declaring rust tooling and .sh forbidden must report deploy.sh",
        unconfigured: 0,
        declared: 1,
        declared_summaries: &[],
    });
}

/// The goals-policy materialization reaching `Check_Goals_And_Parts_Line_Up`, the one composed
/// rule whose subject is not source at all.
///
/// Both halves of this comparison are load-bearing in a way the limits and scripting pairs are
/// not. The undeclared half is what this repository itself looks like -- `standards.json`
/// declares no goals -- so it is the state the rule runs in on every real check here, and
/// silence is the correct answer rather than a fallback. The declared half is the only place
/// anything proves the rule can speak at all through `Run`, and it declares the smallest
/// thing that exercises both directions of the audit: one goal nothing serves, and one part
/// serving no goal.
#[test]
fn Test_Run_Should_Honor_A_Real_Standards_Json_Goal_Declaration()
{
    let answers = Answers_With_And_Without(
        &[Source_File("a.rs", SourceText("pub fn Anything() {}\n"))],
        &[RuleId::New(nomos_rules::GOALS_AND_PARTS_LINE_UP)],
        Standards { name: "goals-declared", declaration: r#"{"goals":["render"],"subsystems":[{"subsystem":"utils","paths":["src/utils"]}]}"# },
    );

    Assert_Answers(&answers, ExpectedAnswers {
        context: "a declared goal nothing serves and a part serving no goal are two findings",
        unconfigured: 0,
        declared: GOAL_AUDIT_DIRECTIONS,
        declared_summaries: &["nothing was built for", "serves no declared goal"],
    });
}

/// The words-policy materialization reaching `Check_Abbreviations`, and the only one of the
/// five whose declaration makes a rule report *less* rather than more.
///
/// The other four policy capabilities either replace a threshold or let a silent rule speak.
/// This one extends a vocabulary the rule already ships, so the proof runs the other way
/// round: the same name is judged twice, and the finding disappears once the repository says
/// the word is one it uses on purpose.
///
/// `ctx` rather than one of this repository's own real additions, deliberately -- a fixture
/// asserting `std` would pass the moment `standards.json` declares it and stop proving the
/// materialization ran at all.
#[test]
fn Test_Run_Should_Honor_A_Real_Standards_Json_Approved_Abbreviation()
{
    let answers = Answers_With_And_Without(
        &[Source_File("a.rs", SourceText("pub fn Read_Ctx() {}\n"))],
        &[RuleId::New(nomos_rules::ABBREVIATIONS)],
        Standards { name: "words-approved", declaration: r#"{"words":{"approved_abbreviations":["ctx"]}}"# },
    );

    Assert_Answers(&answers, ExpectedAnswers {
        context: "ctx has no vowel, so only declaring it approved may silence the shipped vocabulary",
        unconfigured: 1,
        declared: 0,
        declared_summaries: &[],
    });
}
