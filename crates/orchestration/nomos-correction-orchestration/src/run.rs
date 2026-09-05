//! Composing an already-walked tree into a real correction run, apart from choosing a
//! platform, walking a tree, or rendering the answer.
//!
//! Moved here from `nomos-cli`'s own `correct.rs`, the same migration `P13-GATE-RUN-SEAM-
//! CRATE` made for `gate.rs`: that module's own composition — walk, judge, find the one
//! claim, plan, seed a workspace, stage, validate, and optionally commit — is this file's
//! [`Run_Correction`] now, and `nomos-cli`'s `correct.rs` is a thin renderer over it.

use crate::correction_command::CorrectionCommand;
use crate::correction_outcome::CorrectionOutcome;
use crate::phantom_mirror::{Candidate_For, ClaimError, Finding_Reference, Phantom_Claim, PhantomClaim};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{ConfigurationId, EvidenceClass, ProviderId, RuleId};
use nomos_corrections::{CorrectionPlan, ValidatedPlan};
use nomos_model::{Content_Digest, Evidence};
use nomos_platform::{FileSystem, ProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

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

/// Judges `walked` for a blocking phantom-mirror claim, and if it finds one, plans,
/// previews, seeds a workspace, stages and validates the fix -- committing it and writing
/// the corrected file through `environment.filesystem` when `command.commit` is set.
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

    let Some(claim) = findings.iter().find_map(Phantom_Claim)
    else
    {
        return CorrectionOutcome::Clean;
    };

    let (candidate, before, after) = match Candidate_For(&command.root, &claim, filesystem)
    {
        Ok(triple) => triple,
        Err(ClaimError::Unreadable(error)) => return CorrectionOutcome::Refused(format!("could not read `{}`: {error}", claim.path)),
        Err(ClaimError::Ambiguous { occurrences }) =>
        {
            return CorrectionOutcome::Refused(format!(
                "`{}` names `{}` {occurrences} time(s) in `{}`, not exactly once; refusing to guess which line is the real declaration",
                claim.finding.subject_name, claim.claimed, claim.path
            ));
        }
    };

    let plan = match CorrectionPlan::New(vec![candidate])
    {
        Ok(plan) => plan,
        Err(error) => return CorrectionOutcome::Refused(error.to_string()),
    };
    let preview = plan.Preview().Rendered().to_vec();

    let workspace_configuration = ConfigurationId::From_Digest(Content_Digest(command.root.display().to_string().as_bytes()));
    let mut workspace = Workspace::Empty(variant, workspace_configuration);
    let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present(claim.path, before.clone());
    // A one-member, non-empty change set applied to a fresh empty workspace cannot refuse
    // -- see `nomos-cli`'s own former `Seeded_Workspace` for the full reasoning this
    // carries forward unchanged. An unreachable refusal here is not reported further;
    // `Stage` below would then see no content at `claim.path` and refuse the plan as
    // stale on its own.
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
        return CorrectionOutcome::Staged { path: claim.path.to_owned(), claimed: claim.claimed.clone(), preview };
    }

    return Committed(Committing { root: &command.root, claim: &claim, validated, workspace: &mut workspace, after: &after, preview, filesystem });
}

/// What [`Judged`] judges a walked tree against, apart from the walk itself and the
/// platform used to run it -- the same grouping [`nomos_check_orchestration`]'s own
/// callers use to stay within this crate's parameter-count limit.
struct JudgeContext<'a>
{
    variant: BuildVariant,
    root: &'a std::path::Path,
}

/// Runs `Check_Completeness_Mirrors` over `sources`, or the [`CorrectionOutcome`] a
/// non-`Judged` check outcome already decides.
fn Judged<Launcher: ProcessLauncher, Fs: FileSystem>(sources: &[SourceFile], launcher: &Launcher, filesystem: &Fs, context: JudgeContext<'_>) -> Result<Vec<nomos_contracts::Finding>, CorrectionOutcome>
{
    let selected = [RuleId::New(nomos_rules::COMPLETENESS_MIRROR)];
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

/// Everything [`Committed`] needs to commit a validated plan and write the corrected file,
/// grouped into one value so that function stays within this crate's own parameter-count
/// limit.
struct Committing<'a, Fs: FileSystem>
{
    root: &'a std::path::Path,
    claim: &'a PhantomClaim<'a>,
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
    let Committing { root, claim, validated, workspace, after, preview, filesystem } = committing;

    let evidence = Evidence {
        class: EvidenceClass::Derived,
        producer: ProviderId::New("nomos-correction-orchestration-phantom-mirrors"),
        supporting: vec![Finding_Reference(claim)],
    };

    let committed = match validated.Commit(workspace, evidence)
    {
        Ok(committed) => committed,
        Err(error) => return CorrectionOutcome::Refused(error.to_string()),
    };

    if let Err(error) = filesystem.Replace_Atomically(&root.join(claim.path), after)
    {
        return CorrectionOutcome::Refused(format!("committed to the workspace model but could not write `{}`: {error}", claim.path));
    }

    return CorrectionOutcome::Committed {
        path: claim.path.to_owned(),
        claimed: claim.claimed.clone(),
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

    fn Walked(root: &std::path::Path) -> Option<Vec<SourceFile>>
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
    fn Test_A_Tree_With_No_Phantom_Should_Be_Clean()
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
            CorrectionOutcome::Staged { path: staged_path, claimed, .. } =>
            {
                assert_eq!(staged_path, "a.rs");
                assert_eq!(claimed, "Test_Nonexistent_Check_That_Does_Not_Exist");
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
}
