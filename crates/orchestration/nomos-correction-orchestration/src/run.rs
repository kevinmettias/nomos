//! Composing an already-walked tree into a real correction run, apart from choosing a
//! platform, walking a tree, or rendering the answer.
//!
//! Moved here from `nomos-cli`'s own `correct.rs`, the same migration `P13-GATE-RUN-SEAM-
//! CRATE` made for `gate.rs`: that module's own composition — walk, judge, find the one
//! claim, plan, seed a workspace, stage, validate, and optionally commit — is this file's
//! [`Run_Correction`] now, and `nomos-cli`'s `correct.rs` is a thin renderer over it.
//!
//! # A second correction family, and one shared pipeline for both
//!
//! `P40-CORRECTIONS-SECOND-FAMILY-3` adds [`crate::trailing_whitespace`] beside
//! [`crate::phantom_mirror`] -- a second rule (`no-trailing-whitespace`), a second real
//! fix, over a second real finding shape. [`Judged`] now selects both rules at once, and
//! [`Claimed_Fix`] tries each family's own claim-recognizer against the one resulting
//! `findings` list, in a fixed priority order, and hands whichever matches to the one
//! `plan`/`preview`/seed/stage/validate/commit pipeline [`Run_Correction`]'s own body
//! runs exactly once. Neither family's own module knows the other exists; both build the
//! same [`ClaimedFix`] shape, and everything downstream of that point -- roughly two
//! thirds of this function's own body -- is unaware which family produced it.
//!
//! ## What this proves about batching
//!
//! [`crate::trailing_whitespace`]'s own module doc makes the real claim: one candidate
//! covering every flagged line in one file, the first time this crate has batched more
//! than one finding into one edit. Phantom-mirror could not have shown this on its own --
//! `nomos_corrections::CorrectionPlan::New` refuses two candidates touching the same
//! path, so batching only becomes visible once a family's own fix naturally spans more
//! than one location in one file, which striking a single doc-comment line never does.
//!
//! ## What this leaves undecided about ranking
//!
//! [`Claimed_Fix`]'s priority order -- phantom-mirror before trailing-whitespace -- is
//! not a ranking policy. It is this pipeline's own arbitrary tie-break for the one case
//! two families create that one family never could: a tree presenting blocking findings
//! from both at once. Nothing here scores a candidate, compares competing fixes for the
//! same file, or lets a caller ask for a specific family; a real `CorrectionChoice`/
//! `ChoiceRecord` (`nomos-corrections` already declares both, unconstructed) is what
//! `COR-011`..`013`'s own ranking machinery would need a real second candidate to choose
//! between, and this pipeline still only ever proposes one candidate per run. A third
//! family choosing between two real, competing fixes for the *same* location is what
//! would make that decidable; a second family whose own fix shape never overlaps the
//! first's is not that population yet.

use crate::correction_command::CorrectionCommand;
use crate::correction_family::CorrectionFamily;
use crate::correction_outcome::CorrectionOutcome;
use crate::phantom_mirror::{self, ClaimError, Finding_Reference, Phantom_Claim, PhantomClaim};
use crate::trailing_whitespace::{self, TrailingWhitespaceClaim};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{ConfigurationId, EvidenceClass, Finding, ProviderId, RuleId};
use nomos_corrections::{CorrectionCandidate, CorrectionPlan, ValidatedPlan};
use nomos_model::{Content_Digest, Evidence, EvidenceRef};
use nomos_platform::{FileSystem, ProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};
use std::path::Path;

/// The build variant, process launcher and filesystem [`Run_Correction`] needs but does not
/// compute -- grouped into one value the same way `nomos_gate_orchestration::
/// GateEnvironment` groups its own three, for the identical reason: neither computes a
/// build variant, chooses a platform, or is the caller that gets to decide either.
pub struct CorrectionEnvironment<'a, Launcher: ProcessLauncher, Fs: FileSystem>
{
    pub variant: BuildVariant,
    pub launcher: &'a Launcher,
    pub filesystem: &'a Fs,
}

/// Judges `walked` for a blocking claim from either correction family, and if one
/// matches, plans, previews, seeds a workspace, stages and validates the fix --
/// committing it and writing the corrected file through `environment.filesystem` when
/// `command.commit` is set.
///
/// `walked` is the walk, already done and already decided by the composition root, the
/// same reason `nomos_gate_orchestration::Run_Gate` takes it rather than a root to read.
#[must_use]
pub fn Run_Correction<Launcher: ProcessLauncher, Fs: FileSystem>(walked: Option<Vec<SourceFile>>, environment: CorrectionEnvironment<'_, Launcher, Fs>, command: &CorrectionCommand) -> CorrectionOutcome
{
    let CorrectionEnvironment { variant, launcher, filesystem } = environment;

    let Some(sources) = walked
    else
    {
        return CorrectionOutcome::UnreadableRoot;
    };
    if sources.is_empty()
    {
        return CorrectionOutcome::NoSourceFound;
    }

    let findings = match Judged(&sources, launcher, filesystem, JudgeContext { variant: variant.clone(), root: &command.root })
    {
        Ok(findings) => findings,
        Err(outcome) => return outcome,
    };

    let fix = match Claimed_Fix(&command.root, &findings, filesystem)
    {
        None => return CorrectionOutcome::Clean,
        Some(Err(outcome)) => return outcome,
        Some(Ok(fix)) => fix,
    };

    let plan = match CorrectionPlan::New(vec![fix.candidate])
    {
        Ok(plan) => plan,
        Err(error) => return CorrectionOutcome::Refused(error.to_string()),
    };
    let preview = plan.Preview().Rendered().to_vec();

    let workspace_configuration = ConfigurationId::From_Digest(Content_Digest(command.root.display().to_string().as_bytes()));
    let mut workspace = Workspace::Empty(variant, workspace_configuration);
    let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present(fix.path.clone(), fix.before.clone());
    // A one-member, non-empty change set applied to a fresh empty workspace cannot refuse
    // -- see `nomos-cli`'s own former `Seeded_Workspace` for the full reasoning this
    // carries forward unchanged. An unreachable refusal here is not reported further;
    // `Stage` below would then see no content at `fix.path` and refuse the plan as stale
    // on its own.
    let _ignored = workspace.Apply(&initial);

    let staged = match plan.Stage(&workspace)
    {
        Ok(staged) => staged,
        Err(error) => return CorrectionOutcome::Refused(error.to_string()),
    };
    let validated: ValidatedPlan = match staged.Validate(&workspace)
    {
        Ok(validated) => validated,
        Err(error) => return CorrectionOutcome::Refused(error.to_string()),
    };

    if !command.commit
    {
        return CorrectionOutcome::Staged { path: fix.path, summary: fix.summary, preview };
    }

    return Committed(Committing {
        root: &command.root,
        path: fix.path,
        summary: fix.summary,
        evidence_reference: fix.evidence_reference,
        validated,
        workspace: &mut workspace,
        after: &fix.after,
        preview,
        filesystem,
    });
}

/// What [`Judged`] judges a walked tree against, apart from the walk itself and the
/// platform used to run it -- the same grouping [`nomos_check_orchestration`]'s own
/// callers use to stay within this crate's parameter-count limit.
struct JudgeContext<'a>
{
    variant: BuildVariant,
    root: &'a Path,
}

/// Runs both correction families' own rules over `sources`, or the [`CorrectionOutcome`] a
/// non-`Judged` check outcome already decides.
fn Judged<Launcher: ProcessLauncher, Fs: FileSystem>(sources: &[SourceFile], launcher: &Launcher, filesystem: &Fs, context: JudgeContext<'_>) -> Result<Vec<Finding>, CorrectionOutcome>
{
    let selected: Vec<RuleId> = CorrectionFamily::ALL.iter().map(|family| return family.Rule()).collect();
    let outcome = nomos_check_orchestration::Run(
        sources,
        nomos_check_orchestration::RunContext {
            variant: context.variant,
            root: context.root,
            launcher,
            filesystem,
            workspace: &mut None,
            store: &mut nomos_analysis::MemoryFactStore::New(),
        },
        &selected,
    );

    return match outcome
    {
        CheckOutcome::Judged { findings, .. } => Ok(findings),
        CheckOutcome::Unreadable => Err(CorrectionOutcome::UnreadableWorkspaceState),
        CheckOutcome::Contradictory(error) => Err(CorrectionOutcome::ContradictoryRegistry(error.to_string())),
        CheckOutcome::NoSource => Err(CorrectionOutcome::NoSourceFound),
        CheckOutcome::NoFacts { files } => Err(CorrectionOutcome::NoFactsMaterialized(files)),
    };
}

/// One family's real candidate for a claim it recognized, and everything the shared
/// pipeline downstream of [`Claimed_Fix`] needs to stage, validate and commit it without
/// knowing which family built it.
struct ClaimedFix
{
    path: String,
    /// The candidate's own description, carried through to [`CorrectionOutcome`]
    /// verbatim -- each family writes the human-readable summary its own fix deserves,
    /// and nothing downstream re-decides what it means.
    summary: String,
    candidate: CorrectionCandidate,
    before: String,
    after: String,
    evidence_reference: EvidenceRef,
}

/// Tries each [`CorrectionFamily`] against `findings`, in [`CorrectionFamily::ALL`]'s own
/// declared order -- the first to recognize a claim wins. `None` if none does, a clean
/// run. `Some(Err(...))` if a family recognized a claim but could not safely build a
/// candidate for it. This crate's own module doc says what that priority order is and is
/// not.
///
/// Iterating the family list and matching exhaustively over it is deliberate, and is the
/// mechanism the whole declaration rests on: a variant added to [`CorrectionFamily`] does
/// not compile until it is wired to a real recognizer here, so a family cannot be selected
/// by [`Judged`] and then be recognized by nobody. An `if let` chain, which is what this
/// was, would have accepted the new family in silence.
fn Claimed_Fix<Fs: FileSystem>(root: &Path, findings: &[Finding], filesystem: &Fs) -> Option<Result<ClaimedFix, CorrectionOutcome>>
{
    for family in CorrectionFamily::ALL
    {
        match family
        {
            CorrectionFamily::PhantomMirror =>
            {
                if let Some(claim) = findings.iter().find_map(Phantom_Claim)
                {
                    return Some(Phantom_Mirror_Fix(root, &claim, filesystem));
                }
            }
            CorrectionFamily::TrailingWhitespace =>
            {
                if let Some(claim) = trailing_whitespace::Trailing_Whitespace_Claim(findings)
                {
                    return Some(Trailing_Whitespace_Fix(root, &claim, filesystem));
                }
            }
        }
    }

    return None;
}

/// Builds the [`ClaimedFix`] phantom-mirror's own candidate makes for `claim`, or the
/// [`CorrectionOutcome`] its own refusal already decides.
fn Phantom_Mirror_Fix<Fs: FileSystem>(root: &Path, claim: &PhantomClaim<'_>, filesystem: &Fs) -> Result<ClaimedFix, CorrectionOutcome>
{
    let (candidate, before, after) = match phantom_mirror::Candidate_For(root, claim, filesystem)
    {
        Ok(triple) => triple,
        Err(ClaimError::Unreadable(error)) => return Err(CorrectionOutcome::Refused(format!("could not read `{}`: {error}", claim.path))),
        Err(ClaimError::Ambiguous { occurrences }) =>
        {
            return Err(CorrectionOutcome::Refused(format!(
                "`{}` names `{}` {occurrences} time(s) in `{}`, not exactly once; refusing to guess which line is the real declaration",
                claim.finding.subject_name, claim.claimed, claim.path
            )));
        }
    };

    return Ok(ClaimedFix {
        path: claim.path.to_owned(),
        summary: candidate.Description().to_owned(),
        candidate,
        before,
        after,
        evidence_reference: Finding_Reference(claim),
    });
}

/// Builds the [`ClaimedFix`] trailing-whitespace's own candidate makes for `claim`, or the
/// [`CorrectionOutcome`] its own refusal already decides.
fn Trailing_Whitespace_Fix<Fs: FileSystem>(root: &Path, claim: &TrailingWhitespaceClaim<'_>, filesystem: &Fs) -> Result<ClaimedFix, CorrectionOutcome>
{
    let (candidate, before, after) = trailing_whitespace::Candidate_For(root, claim, filesystem)
        .map_err(|error| return CorrectionOutcome::Refused(format!("could not read `{}`: {error}", claim.path)))?;

    return Ok(ClaimedFix {
        path: claim.path.to_owned(),
        summary: candidate.Description().to_owned(),
        candidate,
        before,
        after,
        evidence_reference: EvidenceRef {
            kind: "finding".to_owned(),
            locator: format!("{}::{}", nomos_rules::NO_TRAILING_WHITESPACE, claim.path),
        },
    });
}

/// Everything [`Committed`] needs to commit a validated plan and write the corrected file,
/// grouped into one value so that function stays within this crate's own parameter-count
/// limit.
struct Committing<'a, Fs: FileSystem>
{
    root: &'a Path,
    path: String,
    summary: String,
    evidence_reference: EvidenceRef,
    validated: ValidatedPlan,
    workspace: &'a mut Workspace,
    after: &'a str,
    preview: Vec<u8>,
    filesystem: &'a Fs,
}

/// Commits `committing.validated` through the workspace's one door and writes the
/// corrected file through `committing.filesystem`, or reports why either step refused.
fn Committed<Fs: FileSystem>(committing: Committing<'_, Fs>) -> CorrectionOutcome
{
    let Committing { root, path, summary, evidence_reference, validated, workspace, after, preview, filesystem } = committing;

    let evidence = Evidence {
        class: EvidenceClass::Derived,
        producer: ProviderId::New("nomos-correction-orchestration"),
        supporting: vec![evidence_reference],
    };

    let committed = match validated.Commit(workspace, evidence)
    {
        Ok(committed) => committed,
        Err(error) => return CorrectionOutcome::Refused(error.to_string()),
    };

    if let Err(error) = filesystem.Replace_Atomically(&root.join(&path), after)
    {
        return CorrectionOutcome::Refused(format!("committed to the workspace model but could not write `{path}`: {error}"));
    }

    return CorrectionOutcome::Committed {
        path,
        summary,
        preview,
        base: committed.Base().to_string(),
        after_snapshot: committed.After().to_string(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_model::Subject_Of_Path;
    use nomos_platform_std::{StdFileSystem, StdProcessLauncher};

    fn Fresh_Root(name: &str) -> std::path::PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }

    fn Test_Variant() -> BuildVariant
    {
        return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
    }

    fn Walked(root: &Path) -> Option<Vec<SourceFile>>
    {
        let mut sources = Vec::new();
        for entry in std::fs::read_dir(root).ok()?.flatten()
        {
            let path = entry.path();
            if path.extension().is_some_and(|extension| return extension == "rs")
            {
                let text = std::fs::read_to_string(&path).expect("readable");
                let relative = path.strip_prefix(root).expect("under root").display().to_string();
                sources.push(SourceFile::New(relative.clone(), Subject_Of_Path(&relative), text));
            }
        }
        return Some(sources);
    }

    #[test]
    fn Test_An_Unwalked_Root_Should_Be_Unreadable()
    {
        let command = CorrectionCommand { root: "does/not/exist".into(), commit: false };
        let outcome = Run_Correction(None, CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);

        assert_eq!(outcome, CorrectionOutcome::UnreadableRoot);
    }

    #[test]
    fn Test_An_Empty_Walk_Should_Report_No_Source_Found()
    {
        let command = CorrectionCommand { root: "irrelevant".into(), commit: false };
        let outcome = Run_Correction(Some(Vec::new()), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);

        assert_eq!(outcome, CorrectionOutcome::NoSourceFound);
    }

    #[test]
    fn Test_A_Tree_With_Neither_Claim_Should_Be_Clean()
    {
        let root = Fresh_Root("nomos-correction-orchestration-clean-tree");
        std::fs::write(root.join("a.rs"), "pub fn Something() -> u32 { return 1; }\n").expect("writable");
        let command = CorrectionCommand { root: root.clone(), commit: false };

        let outcome = Run_Correction(Walked(&root), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(outcome, CorrectionOutcome::Clean);
    }

    const PHANTOM_FIXTURE: &str = "/// A list of things this crate owns.\n\
        /// Mirrored by `Test_Nonexistent_Check_That_Does_Not_Exist`.\n\
        pub const THINGS: &[&str] = &[\"a\"];\n";

    #[test]
    fn Test_A_Dry_Run_Should_Stage_And_Validate_Without_Touching_Disk()
    {
        let root = Fresh_Root("nomos-correction-orchestration-dry-run");
        let path = root.join("a.rs");
        std::fs::write(&path, PHANTOM_FIXTURE).expect("writable");
        let command = CorrectionCommand { root: root.clone(), commit: false };

        let outcome = Run_Correction(Walked(&root), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);
        let on_disk = std::fs::read_to_string(&path).expect("still readable");

        let _ignored = std::fs::remove_dir_all(&root);
        match outcome
        {
            CorrectionOutcome::Staged { path: staged_path, summary, .. } =>
            {
                assert_eq!(staged_path, "a.rs");
                assert!(summary.contains("Test_Nonexistent_Check_That_Does_Not_Exist"), "{summary}");
            }
            other => panic!("expected Staged, got {other:?}"),
        }
        assert_eq!(on_disk, PHANTOM_FIXTURE, "a dry run must not touch the file");
    }

    #[test]
    fn Test_Committing_Should_Strike_The_Claim_On_Disk_And_Leave_A_Clean_Rerun()
    {
        let root = Fresh_Root("nomos-correction-orchestration-commit");
        let path = root.join("a.rs");
        std::fs::write(&path, PHANTOM_FIXTURE).expect("writable");
        let command = CorrectionCommand { root: root.clone(), commit: true };

        let outcome = Run_Correction(Walked(&root), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);

        match &outcome
        {
            CorrectionOutcome::Committed { path: committed_path, .. } => assert_eq!(committed_path, "a.rs"),
            other => panic!("expected Committed, got {other:?}"),
        }

        let corrected = std::fs::read_to_string(&path).expect("still readable");
        assert_eq!(corrected, "/// A list of things this crate owns.\npub const THINGS: &[&str] = &[\"a\"];\n", "only the phantom claim's own line should be gone");

        let second = Run_Correction(Walked(&root), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);
        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(second, CorrectionOutcome::Clean, "the corrected universe now declares no mirror at all, an admitted gap rather than a second phantom");
    }

    const TRAILING_WHITESPACE_FIXTURE: &str = "pub fn Something() -> u32 \n{\n    return 1; \t\n}\n";

    #[test]
    fn Test_A_Trailing_Whitespace_Dry_Run_Should_Batch_Every_Flagged_Line_In_One_Candidate()
    {
        let root = Fresh_Root("nomos-correction-orchestration-whitespace-dry-run");
        let path = root.join("a.rs");
        std::fs::write(&path, TRAILING_WHITESPACE_FIXTURE).expect("writable");
        let command = CorrectionCommand { root: root.clone(), commit: false };

        let outcome = Run_Correction(Walked(&root), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);
        let on_disk = std::fs::read_to_string(&path).expect("still readable");

        let _ignored = std::fs::remove_dir_all(&root);
        match outcome
        {
            CorrectionOutcome::Staged { path: staged_path, summary, .. } =>
            {
                assert_eq!(staged_path, "a.rs");
                assert!(summary.contains("2 line(s)"), "two lines carry trailing whitespace in the fixture: {summary}");
            }
            other => panic!("expected Staged, got {other:?}"),
        }
        assert_eq!(on_disk, TRAILING_WHITESPACE_FIXTURE, "a dry run must not touch the file");
    }

    #[test]
    fn Test_Committing_A_Trailing_Whitespace_Fix_Should_Strip_Every_Flagged_Line_At_Once()
    {
        let root = Fresh_Root("nomos-correction-orchestration-whitespace-commit");
        let path = root.join("a.rs");
        std::fs::write(&path, TRAILING_WHITESPACE_FIXTURE).expect("writable");
        let command = CorrectionCommand { root: root.clone(), commit: true };

        let outcome = Run_Correction(Walked(&root), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);

        match &outcome
        {
            CorrectionOutcome::Committed { path: committed_path, .. } => assert_eq!(committed_path, "a.rs"),
            other => panic!("expected Committed, got {other:?}"),
        }

        let corrected = std::fs::read_to_string(&path).expect("still readable");
        assert_eq!(corrected, "pub fn Something() -> u32\n{\n    return 1;\n}\n", "both flagged lines are stripped by the one committed candidate");

        let second = Run_Correction(Walked(&root), CorrectionEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem }, &command);
        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(second, CorrectionOutcome::Clean, "a corrected file must not still claim trailing whitespace on a rerun");
    }
}
