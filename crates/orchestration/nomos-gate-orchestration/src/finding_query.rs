//! `nomos gate explain` -- what one named finding looks like right now, and whether it
//! would keep a real run from passing.

mod explanation;
mod gate_explain_result;

pub use explanation::Explanation;
pub use gate_explain_result::GateExplainResult;

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher, Timestamp};
use nomos_rules::SourceFile;

use crate::gate_environment::{GateEnvironment, JudgeContext, Judged_Sources};
use crate::policy::{GatePolicyFile, Resolve_Gate_Policy};
use crate::{AdoptionPolicy, BaselinePolicy, GateCommand, SuppressionPolicy};

/// Which finding to explain: the rule that produced it, and one of the locations it names --
/// the same human-visible `Finding::locations` a reader of `nomos gate run`'s own output
/// already sees, not a digest nothing prints today.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingQuery
{
    /// The rule the finding to explain was produced by.
    pub rule: RuleId,
    /// One location the finding names.
    pub location: String,
}

/// Judges `walked` exactly as [`crate::Run_Gate`] would, then answers `query` against what
/// was judged.
///
/// Deliberately independent of `command.scope` and `command.rules`: those narrow a real
/// run's *disposition* over many findings, and this answers a question about one named
/// finding as check would produce it right now -- not "what would a scope- or
/// rule-narrowed `run` currently see". Calibration, suppression and baseline are the three
/// it does consult, because whether any applies is part of the finding's own explanation,
/// not part of narrowing which findings a run counts.
///
/// Those three are taken from the *effective* policy -- the declared `nomos-gate.json` under
/// `command.root`, with the caller's own preferred over it -- and not from `command` alone.
/// Reading `command` alone was a defect rather than a narrower reading: no caller populates
/// those fields (the CLI hardcodes all three to their defaults and no flag authors them, and
/// `nomos_api`'s handler passes a default command narrowed only by `root`), so this answered
/// every query as though a repository tolerated nothing. Over a tree declaring one baseline
/// entry, `run` reported the finding baselined and exited clean while `explain` reported it
/// blocking and exited 1 -- one product contradicting itself about one tree, on the side a
/// repository had adopted on purpose.
///
/// `Judged` is passed an empty rule selection here, not `command.rules.include`, for
/// the same reason: since `OD-GATE-017`, a non-empty selection also narrows what
/// [`nomos_check_orchestration::Run`] computes at all, and a query about a rule
/// `command.rules` excludes must still be answerable.
#[must_use]
pub fn Explain_Gate<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    walked: Option<Vec<SourceFile>>,
    environment: GateEnvironment<'_, Launcher, Fs, Env>,
    command: &GateCommand,
    query: &FindingQuery,
) -> GateExplainResult
{
    let GateEnvironment { variant, launcher, filesystem, environment, now } = environment;
    let effective = Effective_Policies(command, filesystem);
    let check_outcome = Judged_Sources(walked, JudgeContext { launcher, filesystem, environment, variant, root: &command.root, selected: &[] });
    let explanation = Explained_Query(
        &check_outcome,
        query,
        DispositionPolicies { adoption: &effective.adoption, suppressions: &effective.suppressions, baseline: &effective.baseline, now },
    );

    return GateExplainResult { root: command.root.clone(), check_outcome, explanation };
}

/// The policy [`Explain_Gate`] answers with: the `nomos-gate.json` under `command.root`, merged
/// under the caller's own.
///
/// Resolved here rather than taken from `command` alone, because the three policies this answer
/// is about are the ones a real run would apply, and a run applies the declared file merged under
/// the caller's own. Read before `filesystem` is handed to the judging, which is the same
/// ordering `Run_Gate` keeps for the same reason.
///
/// An unreadable file falls back to the caller's policies standing alone, exactly as `Run_Gate`
/// does. That direction is the safe one here: a policy nobody could read tolerates nothing, so
/// the answer over-reports blocking rather than claiming a tolerance it could not verify.
/// `Run_Gate` additionally withholds its verdict in that case; this function has no verdict to
/// withhold, and reporting one finding as blocking is not a claim about the run.
fn Effective_Policies<Fs: FileSystem>(command: &GateCommand, filesystem: &Fs) -> GatePolicyFile
{
    return match Resolve_Gate_Policy(&command.root, filesystem)
    {
        Ok(Some(from_file)) => from_file.Resolved_Over(command),
        Ok(None) | Err(_) => GatePolicyFile::default().Resolved_Over(command),
    };
}

/// The three per-finding overrides [`Explained_Query`] and [`Disposed_Finding`] check,
/// grouped into one value so [`Explained_Query`] stays within this crate's own
/// parameter-count limit -- `adoption` checked first (a coarser, rule-wide override), then
/// `suppressions`, then `baseline`, the same order [`crate::Run_Gate`] reduces by.
#[derive(Clone, Copy)]
struct DispositionPolicies<'a>
{
    adoption: &'a AdoptionPolicy,
    suppressions: &'a SuppressionPolicy,
    baseline: &'a BaselinePolicy,
    now: Timestamp,
}

fn Explained_Query(outcome: &CheckOutcome, query: &FindingQuery, policies: DispositionPolicies<'_>) -> Explanation
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return Explanation::NotFound;
    };

    return Named_Finding(findings, query)
        .map_or(Explanation::NotFound, |finding| return Disposed_Finding(finding, policies));
}

/// The one finding `query` names among `findings`, if any.
fn Named_Finding<'a>(findings: &'a [Finding], query: &FindingQuery) -> Option<&'a Finding>
{
    return findings
        .iter()
        .find(|finding| return finding.rule == query.rule && finding.locations.iter().any(|location| return location == &query.location));
}

/// `finding`, reduced to what a real run would do with it -- blocked, calibrated,
/// suppressed, or baselined, checked in that order, the same order [`crate::Run_Gate`]
/// reduces by.
fn Disposed_Finding(finding: &Finding, policies: DispositionPolicies<'_>) -> Explanation
{
    let DispositionPolicies { adoption, suppressions, baseline, now } = policies;
    let calibrated_by = adoption.Calibrating(finding).cloned();
    let suppressed_by = calibrated_by.is_none().then(|| suppressions.Suppressing(finding, now).cloned()).flatten();
    let baselined_by = (calibrated_by.is_none() && suppressed_by.is_none())
        .then(|| baseline.Tolerating(finding).cloned())
        .flatten();
    let would_block = finding.Can_Fail_A_Build() && calibrated_by.is_none() && suppressed_by.is_none() && baselined_by.is_none();
    let contract = Contract_Of(&finding.rule);

    return Explanation::Found {
        finding: Box::new(finding.clone()),
        would_block,
        calibrated_by,
        suppressed_by,
        baselined_by,
        contract,
    };
}

/// `query.rule`'s contract citation, from the same registry `nomos gate plan` builds --
/// `None` only for a registry `Registered` itself refuses, or a rule that registry does
/// not hold at all.
fn Contract_Of(rule: &RuleId) -> Option<(String, u32)>
{
    use crate::Registered;

    let registry = Registered().ok()?;
    let offer = registry.Offered(rule)?;

    return Some((offer.contract_record.clone(), offer.contract_record_version));
}

#[cfg(test)]
mod tests
{
    use super::{Explain_Gate, FindingQuery};
    use crate::{BaselineAllowance, BaselineDebt, Explanation, GateCommand, GateEnvironment};
    use nomos_model::Subject_Of_Path;
    use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
    use nomos_rules::{SourceFile, COMPLETENESS_MIRROR, NO_SINGLE_LINE_FUNCTION_BODIES};
    use nomos_workspace::BuildVariant;
    use std::path::PathBuf;

    /// The distinguishing part of a scratch root's directory name.
    ///
    /// A type of its own rather than the `&str` it wraps, because it sits in the position next to
    /// the policy text it has nothing in common with: two adjacent `&str` parameters are a pair a
    /// caller can hand over in the wrong order and the compiler will accept both.
    struct ScratchName(&'static str);

    /// The path a fixture source is reported under, and the subject its findings are
    /// addressed to -- distinct from the text beside it, because a caller handing the text
    /// over where the path belongs compiles as readily as the other way round.
    struct SourcePath(&'static str);

    /// The text judged as the contents of one fixture source.
    struct SourceText(&'static str);

    fn Source_File(path: SourcePath, text: SourceText) -> SourceFile
    {
        return SourceFile::New(path.0, Subject_Of_Path(path.0), text.0);
    }

    /// A real query naming the one blocking finding a phantom mirror produces answers `Found`,
    /// with `would_block` true -- proof this crate's own `Explain_Gate` (not a rename of the
    /// higher-level `Run_Gate` fixture) actually judges the source and answers the query.
    #[test]
    fn Test_Explain_Gate_Should_Find_A_Real_Blocking_Finding()
    {
        let sources = vec![Source_File(
            SourcePath("a.rs"),
            SourceText("/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n"),
        )];
        let query = FindingQuery { rule: nomos_contracts::RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };
        let command = GateCommand { root: Repository_Root(), ..Default::default() };

        let result = Explain_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, &command, &query);

        let Explanation::Found { would_block, .. } = result.explanation
        else
        {
            // this fixture's own source matches the rule it names; a refusal here is a bug
            // in the fixture, not a caller-facing failure to route through Result.
            panic!("this fixture must produce the finding the query names");
        };
        assert!(would_block);
    }

    fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest.parent().and_then(std::path::Path::parent).and_then(std::path::Path::parent).map(PathBuf::from).expect("this crate sits three levels below the workspace root");
    }

    /// The declared policy decides this answer, because it decides the run's.
    ///
    /// Before this, `Explain_Gate` read the three policies off `command` and nothing else, and
    /// no caller fills them -- so a repository that had adopted a baseline was told by `explain`
    /// that the finding blocks, while `run` over the same tree tolerated it and exited clean.
    /// One product, one tree, two answers, and the wrong one contradicted a tolerance the
    /// repository had declared deliberately.
    ///
    /// Judged from a `walked` fixture over a scratch root holding nothing but the policy file:
    /// the tree is never walked, so this asserts the policy resolution alone and nothing about
    /// what a walk would find.
    #[test]
    fn Test_Explain_Gate_Should_Read_The_Baseline_The_Tree_Declares()
    {
        let explanation = Explained_Under_Declared_Policy(
            ScratchName("declares-a-baseline"),
            &format!(r#"{{ "baseline": [ {{ "rule": "{NO_SINGLE_LINE_FUNCTION_BODIES}", "path": "a.rs", "rationale": "pre-existing at adoption" }} ] }}"#),
        );
        let Explanation::Found { would_block, baselined_by, .. } = explanation
        else
        {
            panic!("this fixture must produce the finding the query names");
        };
        assert_eq!(
            baselined_by,
            Some(BaselineDebt {
                rule: nomos_contracts::RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES),
                subject: Subject_Of_Path("a.rs"),
                rationale: "pre-existing at adoption".to_owned(),
                allowance: BaselineAllowance::Unbounded,
                declared_path: Some("a.rs".to_owned()),
            }),
            "the declared entry is what a real run would apply, so it is what explains the finding"
        );
        assert!(!would_block, "a finding a real run tolerates must not be reported as one that blocks");
    }

    /// A declared file that cannot be parsed leaves the caller's own policies standing alone,
    /// the same fallback `Run_Gate` takes.
    ///
    /// Asserted in the blocking direction on purpose. Nothing could be read, so nothing is
    /// known to be tolerated, and the honest answer is the one that claims no tolerance rather
    /// than the one that assumes the entry a reader meant to write.
    #[test]
    fn Test_Explain_Gate_Should_Claim_No_Tolerance_It_Could_Not_Read()
    {
        let explanation = Explained_Under_Declared_Policy(ScratchName("declares-an-unreadable-baseline"), "{ not json");
        let Explanation::Found { would_block, baselined_by, .. } = explanation
        else
        {
            panic!("this fixture must produce the finding the query names");
        };
        assert_eq!(baselined_by, None, "an unreadable file grants no tolerance");
        assert!(would_block);
    }

    /// What `Explain_Gate` answers about the collapsed body under a scratch root declaring
    /// `policy`, with that root removed before this returns.
    ///
    /// The two tests above differ only in the policy text and in what they then assert about the
    /// answer, so the three steps that get from one to the other -- write the file, explain under
    /// it, clean the directory up -- are written once.
    fn Explained_Under_Declared_Policy(name: ScratchName, policy: &str) -> Explanation
    {
        let root = Scratch_Root_Declaring(name, policy);
        let command = GateCommand { root: root.clone(), ..Default::default() };
        let explanation = Explained_Collapsed_Body(&command);
        let _ignored = std::fs::remove_dir_all(&root);

        return explanation;
    }

    /// A directory holding one `nomos-gate.json` and nothing else.
    fn Scratch_Root_Declaring(name: ScratchName, policy: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(format!("nomos-explain-{}-{}", name.0, std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a scratch root");
        std::fs::write(root.join("nomos-gate.json"), policy).expect("the directory holding this file was created by the line above, so the write cannot fail for want of a parent");

        return root;
    }

    /// A collapsed function body, explained under `command`.
    ///
    /// This rule rather than the phantom mirror above, because a declared entry addresses its
    /// subject by *path*, and only a rule whose findings are subjected to the file can be named
    /// by one at all. Picking a rule a declared entry cannot reach would have tested the fixture
    /// rather than the resolution.
    ///
    /// The source is handed over as `walked`, so no tree is read and the only thing `root`
    /// decides is which policy file is resolved -- which is what these two tests are about.
    fn Explained_Collapsed_Body(command: &GateCommand) -> Explanation
    {
        let sources = vec![Source_File(SourcePath("a.rs"), SourceText("pub fn A_Thing() -> i32 { return 1; }
"))];
        let query = FindingQuery { rule: nomos_contracts::RuleId::New(NO_SINGLE_LINE_FUNCTION_BODIES), location: "a.rs:1".to_owned() };

        let result = Explain_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProgramLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment, now: nomos_platform::Timestamp::From_Unix_Seconds(0) }, command, &query);

        return result.explanation;
    }

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }
}
