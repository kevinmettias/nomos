//! The declared source a real run resolves its four policies from.
//!
//! Every policy type in this module's siblings documented that nothing constructed a
//! non-empty one, and each was reachable only from a unit test. This is the authoring
//! surface that closes that: one JSON file at the run's own root, read through the
//! [`FileSystem`] port [`crate::Run_Gate`] already carries.
//!
//! # Why resolution happens inside `Run_Gate` rather than in each composition root
//!
//! `Run_Gate` already takes both the root and the filesystem port, so resolving here serves
//! the CLI, `nomos-api`, the transport and MCP at once. Resolving in each composition root
//! instead would put the same file-format knowledge in four places, which is the duplication
//! `OD-GATE-011` names as a defect class, and would leave any surface that forgot it silently
//! running on defaults.
//!
//! # Why entries name a path and not a subject
//!
//! [`crate::Suppression`] and [`crate::BaselineDebt`] both match on `SubjectId`, which is a
//! digest no person can author by hand. `nomos_model::Subject_Of_Path` is the kernel's
//! canonical path-to-subject mapping -- `OD-MODEL-001`, and the one every real walker already
//! uses to build `SourceFile::subject` -- so a file names the path a reader can see and this
//! module computes the identity a run matches on, by calling the same function the walk does
//! rather than reimplementing it.
//!
//! # What a path-authored entry does not reach
//!
//! A finding is not always addressed by the subject of the file it was found in. Measured
//! directly against a real run rather than assumed: `no-single-line-function-bodies` and
//! `todo-format-is-todo-name-description-ticket` address the file's own subject, so a path
//! names them; `completeness-mirror` addresses the mirrored item and `single-letter-names`
//! addresses the name, so a path does not. `crate::tests`'
//! `Test_A_Suppressed_Finding_Should_Not_Block` already documented this for the first of
//! those, which is why it reads its subject off a real run instead of computing one.
//!
//! So this file suppresses and baselines findings a rule addresses by file, and silently
//! matches nothing for the rest. That is a real limit and not a safe default -- an author who
//! writes an entry for `completeness-mirror` gets no error and no effect. It is left here
//! rather than papered over with a digest field, because a file naming a raw `SubjectId` is
//! the unauthorable identity this module exists to avoid, and the honest fix is for
//! sub-item findings to carry an addressable name that a person can write. That is a
//! question about `Finding` rather than about this file, and it belongs to whichever item
//! takes it up.
//!
//! # Why the file loses to an explicitly constructed policy
//!
//! A caller that built a policy in code has said something more specific than a file sitting
//! in a tree, and a test that pins a policy must not have it silently replaced by whatever
//! the working directory happens to contain. [`GatePolicyFile::Resolved_Over`] therefore
//! fills in only the fields a command left at its default. `Default` already means "unset"
//! for all four -- `CoveragePolicy::Unset` says so in its own name -- so this reads the same
//! way `ScopeSelector`'s empty-means-everything already does.

use nomos_contracts::RuleId;
use nomos_model::Subject_Of_Path;
use nomos_platform::{FileSystem, FileSystemError};
use serde::Deserialize;
use std::path::Path;

use super::{
    AdoptionPolicy, BaselineDebt, BaselinePolicy, CoveragePolicy, RuleCalibration, Suppression, SuppressionDisposition, SuppressionPolicy,
};
use crate::GateCommand;

/// The file a run resolves its policies from, relative to the run's own root.
///
/// JSON rather than TOML because this workspace already reads JSON for `standards.json` and
/// `work/ledger.json` and parses none of the tool-configuration TOML it sits beside, so JSON
/// is the format a reader here has already met.
pub(crate) const GATE_POLICY_FILE: &str = "nomos-gate.json";

/// What [`GATE_POLICY_FILE`] said, resolved into the policy types a run judges against.
///
/// Holds the domain types rather than the wire shapes: the deserialized forms below exist
/// only long enough to be converted, so nothing downstream of this type has to know a file
/// was involved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GatePolicyFile
{
    /// Dispositions the file declared.
    pub(crate) suppressions: SuppressionPolicy,
    /// Existing debt the file declared.
    pub(crate) baseline: BaselinePolicy,
    /// Rule calibrations the file declared.
    pub(crate) adoption: AdoptionPolicy,
    /// The coverage floor the file declared.
    pub(crate) coverage: CoveragePolicy,
}

impl GatePolicyFile
{
    /// This file's policies, with any that `command` states explicitly winning.
    ///
    /// A field left at its default in `command` takes the file's value; a field the caller
    /// built takes the caller's. See this module's own doc for why that precedence and not
    /// the reverse.
    #[must_use]
    pub(crate) fn Resolved_Over(&self, command: &GateCommand) -> Self
    {
        return Self {
            suppressions: Preferred(&command.suppressions, &self.suppressions),
            baseline: Preferred(&command.baseline, &self.baseline),
            adoption: Preferred(&command.adoption, &self.adoption),
            coverage: if command.coverage == CoveragePolicy::default() { self.coverage } else { command.coverage },
        };
    }
}

/// `stated` when a caller built one, the file's otherwise.
///
/// Generic over the three list-shaped policies rather than written three times, since
/// "default means the caller said nothing" is one rule and not three.
fn Preferred<Policy: Clone + Default + PartialEq>(stated: &Policy, from_file: &Policy) -> Policy
{
    return if *stated == Policy::default() { from_file.clone() } else { stated.clone() };
}

/// Why a policy file that exists could not become a policy.
///
/// Absence is not represented here: a tree with no policy file is the ordinary case, not a
/// failure, and [`Resolve_Gate_Policy`] reports it as `Ok(None)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GatePolicyError
{
    /// The file is there and could not be read.
    Unreadable(String),
    /// The file was read and is not the shape this module expects.
    Malformed(String),
}

/// The policy [`GATE_POLICY_FILE`] under `root` declares, or `Ok(None)` when there is none.
///
/// # Errors
///
/// Returns [`GatePolicyError`] when a file is present but cannot be turned into a policy.
/// A present-but-broken file is deliberately not treated as an absent one: a repository that
/// meant to suppress a finding and misspelled the file would otherwise get a build that
/// passes for a reason nobody chose, which is the failure this whole item exists to stop.
pub(crate) fn Resolve_Gate_Policy<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<Option<GatePolicyFile>, GatePolicyError>
{
    let text = match filesystem.Read_To_String(&root.join(GATE_POLICY_FILE))
    {
        Ok(text) => text,
        Err(FileSystemError::NotFound { .. }) => return Ok(None),
        Err(error) => return Err(GatePolicyError::Unreadable(format!("{}: {error:?}", error.Path()))),
    };

    let declared: DeclaredPolicy = serde_json::from_str(&text).map_err(|error| return GatePolicyError::Malformed(error.to_string()))?;

    return Ok(Some(declared.Resolved()));
}

/// [`GATE_POLICY_FILE`]'s own shape, as written.
///
/// Separate from [`GatePolicyFile`] so `serde` never reaches the domain types. Deriving
/// `Deserialize` on [`Suppression`] and its siblings would make a file able to name a
/// `SubjectId` digest directly, which is exactly the unauthorable identity this module
/// exists to compute rather than accept, and would widen the crate's public surface on
/// behalf of one caller's format.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct DeclaredPolicy
{
    suppressions: Vec<DeclaredSuppression>,
    baseline: Vec<DeclaredDebt>,
    adoption: Vec<DeclaredCalibration>,
    coverage: DeclaredCoverage,
}

impl DeclaredPolicy
{
    /// This declaration as the policy types a run judges against.
    fn Resolved(self) -> GatePolicyFile
    {
        return GatePolicyFile {
            suppressions: SuppressionPolicy { suppressions: self.suppressions.into_iter().map(DeclaredSuppression::Resolved).collect() },
            baseline: BaselinePolicy { debt: self.baseline.into_iter().map(DeclaredDebt::Resolved).collect() },
            adoption: AdoptionPolicy { calibrated: self.adoption.into_iter().map(DeclaredCalibration::Resolved).collect() },
            coverage: self.coverage.Resolved(),
        };
    }
}

/// One suppression, as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredSuppression
{
    rule: String,
    path: String,
    disposition: DeclaredDisposition,
    rationale: String,
    owner: String,
}

impl DeclaredSuppression
{
    /// This entry as the domain type, with its subject computed from its path.
    fn Resolved(self) -> Suppression
    {
        return Suppression {
            rule: RuleId::New(&self.rule),
            subject: Subject_Of_Path(&self.path),
            disposition: self.disposition.Resolved(),
            rationale: self.rationale,
            owner: self.owner,
        };
    }
}

/// One baseline entry, as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredDebt
{
    rule: String,
    path: String,
    rationale: String,
}

impl DeclaredDebt
{
    /// This entry as the domain type, with its subject computed from its path.
    fn Resolved(self) -> BaselineDebt
    {
        return BaselineDebt { rule: RuleId::New(&self.rule), subject: Subject_Of_Path(&self.path), rationale: self.rationale };
    }
}

/// One rule calibration, as written.
///
/// Names no path, because [`RuleCalibration`] matches by rule alone -- a calibration is the
/// coarse, rule-wide override, and giving it a path here would suggest a narrowing it does
/// not perform.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredCalibration
{
    rule: String,
    rationale: String,
}

impl DeclaredCalibration
{
    /// This entry as the domain type.
    fn Resolved(self) -> RuleCalibration
    {
        return RuleCalibration { rule: RuleId::New(&self.rule), rationale: self.rationale };
    }
}

/// The six dispositions, as written.
///
/// Spelled in `kebab-case` rather than the Rust variant names because this is a hand-authored
/// file and not a serialization of the enum; the mapping is stated once, here.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclaredDisposition
{
    InlineSuppression,
    RepositoryPolicyException,
    TemporaryWaiver,
    AcceptedBaselineDebt,
    FalsePositive,
    FormalRiskAcceptance,
}

impl DeclaredDisposition
{
    /// This spelling as the domain variant.
    fn Resolved(self) -> SuppressionDisposition
    {
        return match self
        {
            Self::InlineSuppression => SuppressionDisposition::InlineSuppression,
            Self::RepositoryPolicyException => SuppressionDisposition::RepositoryPolicyException,
            Self::TemporaryWaiver => SuppressionDisposition::TemporaryWaiver,
            Self::AcceptedBaselineDebt => SuppressionDisposition::AcceptedBaselineDebt,
            Self::FalsePositive => SuppressionDisposition::FalsePositiveDisposition,
            Self::FormalRiskAcceptance => SuppressionDisposition::FormalRiskAcceptance,
        };
    }
}

/// The coverage floor, as written.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclaredCoverage
{
    /// Coverage debt does not affect disposition -- the state a file that says nothing is in.
    #[default]
    Unset,
    /// Coverage debt downgrades an otherwise-`Passed` run to `Indeterminate`.
    RequireCompleteness,
}

impl DeclaredCoverage
{
    /// This spelling as the domain variant.
    fn Resolved(self) -> CoveragePolicy
    {
        return match self
        {
            Self::Unset => CoveragePolicy::Unset,
            Self::RequireCompleteness => CoveragePolicy::RequireCompleteness,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::{GatePolicyError, GatePolicyFile, Resolve_Gate_Policy, GATE_POLICY_FILE};
    use crate::{CoveragePolicy, GateCommand, SuppressionDisposition};
    use nomos_model::Subject_Of_Path;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    /// A tree of this test's own, named so two tests never share one.
    fn Scratch_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(format!("nomos-gate-policy-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("writable");

        return root;
    }

    /// Writes `contents` as the policy file under a fresh root and returns that root.
    fn Root_With_Policy(name: &str, contents: &str) -> PathBuf
    {
        let root = Scratch_Root(name);
        std::fs::write(root.join(GATE_POLICY_FILE), contents).expect("writable");

        return root;
    }

    #[test]
    fn Test_A_Root_With_No_Policy_File_Should_Resolve_To_Nothing_Rather_Than_Refusing()
    {
        let root = Scratch_Root("absent");

        assert_eq!(Resolve_Gate_Policy(&root, &StdFileSystem), Ok(None));
    }

    /// The end-to-end authoring case: every one of the four policies non-default, from text.
    #[test]
    fn Test_A_Declared_File_Should_Resolve_All_Four_Policies()
    {
        let root = Root_With_Policy(
            "all-four",
            r#"{
                "suppressions": [
                    {
                        "rule": "naming-convention",
                        "path": "src/generated.rs",
                        "disposition": "false-positive",
                        "rationale": "generated code",
                        "owner": "author"
                    }
                ],
                "baseline": [
                    { "rule": "todo-format", "path": "src/legacy.rs", "rationale": "pre-existing" }
                ],
                "adoption": [
                    { "rule": "deprecation", "rationale": "adopting incrementally" }
                ],
                "coverage": "require-completeness"
            }"#,
        );

        let resolved = Resolve_Gate_Policy(&root, &StdFileSystem).expect("well formed").expect("present");

        assert_eq!(resolved.suppressions.suppressions.len(), 1);
        assert_eq!(resolved.baseline.debt.len(), 1);
        assert_eq!(resolved.adoption.calibrated.len(), 1);
        assert_eq!(resolved.coverage, CoveragePolicy::RequireCompleteness);
        assert_eq!(resolved.suppressions.suppressions.first().expect("one entry").disposition, SuppressionDisposition::FalsePositiveDisposition);
    }

    /// The property that makes a path authorable at all: the identity this computes is the
    /// one a real walk files a finding under, because both call the same kernel function.
    #[test]
    fn Test_A_Declared_Path_Should_Resolve_To_The_Subject_A_Walk_Would_File_It_Under()
    {
        let root = Root_With_Policy(
            "subject-identity",
            r#"{ "baseline": [ { "rule": "todo-format", "path": "src/legacy.rs", "rationale": "pre-existing" } ] }"#,
        );

        let resolved = Resolve_Gate_Policy(&root, &StdFileSystem).expect("well formed").expect("present");

        assert_eq!(resolved.baseline.debt.first().expect("one entry").subject, Subject_Of_Path("src/legacy.rs"));
    }

    #[test]
    fn Test_A_File_That_Is_Not_The_Declared_Shape_Should_Refuse_Rather_Than_Read_As_Empty()
    {
        let root = Root_With_Policy("malformed", "{ not json");

        assert!(matches!(Resolve_Gate_Policy(&root, &StdFileSystem), Err(GatePolicyError::Malformed(_))));
    }

    /// `deny_unknown_fields` is what makes a misspelled key a refusal instead of a silently
    /// ignored line, which for this file is the difference between a suppression that
    /// applies and one that does not.
    #[test]
    fn Test_A_Misspelled_Key_Should_Refuse_Rather_Than_Be_Ignored()
    {
        let root = Root_With_Policy("unknown-key", r#"{ "supressions": [] }"#);

        assert!(matches!(Resolve_Gate_Policy(&root, &StdFileSystem), Err(GatePolicyError::Malformed(_))));
    }

    #[test]
    fn Test_A_Policy_A_Caller_Built_Should_Win_Over_The_File()
    {
        let from_file = GatePolicyFile { coverage: CoveragePolicy::RequireCompleteness, ..GatePolicyFile::default() };
        let command = GateCommand { coverage: CoveragePolicy::Unset, ..GateCommand::default() };

        assert_eq!(from_file.Resolved_Over(&command).coverage, CoveragePolicy::RequireCompleteness);

        let stated = GateCommand { coverage: CoveragePolicy::RequireCompleteness, ..GateCommand::default() };
        let silent_file = GatePolicyFile::default();

        assert_eq!(silent_file.Resolved_Over(&stated).coverage, CoveragePolicy::RequireCompleteness);
    }
}
