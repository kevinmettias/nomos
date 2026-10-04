//! Explicit release measurements over real source through the public check seam.
//! Source bytes are frozen once; all edits happen in memory and leave the corpus untouched.

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::{CheckOutcome, RuleReassessmentCache, RunContext, Run_Reassessing};
use nomos_contracts::{Finding, RuleId};
use nomos_integration_tests::{Host_Variant, Walk};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::{SourceFile, COMPLETENESS_MIRROR, REQUIREMENT_TRACE_STALENESS};
use nomos_workspace::Workspace;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

struct Resident
{
    workspace: Option<Workspace>,
    store: MemoryFactStore,
    cache: RuleReassessmentCache,
}

impl Resident
{
    fn Empty() -> Self
    {
        return Self { workspace: None, store: MemoryFactStore::New(), cache: RuleReassessmentCache::New() };
    }

    fn Measure(&mut self, root: &Path, sources: &[SourceFile]) -> Measurement
    {
        let providers = nomos_composer_providers::Standard_Providers();
        let selected = [RuleId::New(COMPLETENESS_MIRROR), RuleId::New(REQUIREMENT_TRACE_STALENESS)];
        let before = self.store.Materializations();
        let context = RunContext {
            variant: Host_Variant(), root, launcher: &StdProgramLauncher, filesystem: &StdFileSystem,
            environment: &StdEnvironment, workspace: &mut self.workspace, store: &mut self.store, providers: &providers,
        };
        let started = Instant::now();
        let outcome = Run_Reassessing(sources, context, &selected, &mut self.cache);
        let elapsed = started.elapsed();
        let CheckOutcome::Judged { findings, .. } = outcome
        else
        {
            panic!("the configured real corpus did not produce a judgment: {outcome:?}");
        };
        return Measurement { elapsed, facts: self.store.Materializations().checked_sub(before).expect("materialization count must never decrease"), findings };
    }
}

struct Measurement
{
    elapsed: Duration,
    facts: u32,
    findings: Vec<Finding>,
}

impl Measurement
{
    fn Print(&self, label: &str)
    {
        eprintln!("release-incremental {label}: elapsed_ms={:.3} materializations={} findings={}",
            self.elapsed.as_secs_f64() * 1_000.0, self.facts, self.findings.len());
    }
}

fn Configured_Corpus() -> (PathBuf, Vec<SourceFile>)
{
    let root = PathBuf::from(std::env::var_os("NOMOS_RELEASE_CORPUS").expect("explicit measurement requires NOMOS_RELEASE_CORPUS"));
    assert!(root.is_dir(), "the configured corpus is not a directory: {}", root.display());
    let corpus = Walk(&root);
    assert!(!corpus.files.is_empty(), "the configured corpus contains no recognized source");
    assert!(corpus.unreadable.is_empty(), "unreadable source: {:?}", corpus.unreadable);
    let sources = corpus.files.into_iter().map(|file| return SourceFile::New(&file.path, file.subject, &file.source)).collect();
    return (root, sources);
}

fn With_One_Edit(sources: &[SourceFile]) -> Vec<SourceFile>
{
    let mut edited = sources.to_vec();
    let file = edited.iter_mut().find(|file| return file.path.ends_with("/lib.rs") || file.path == "lib.rs")
        .expect("this measurement requires a Rust crate root to edit");
    file.text.push_str("\n/// Mirrored by `Test_Release_Incremental_Ghost`.\npub const RELEASE_INCREMENTAL: &[&str] = &[];\n");
    eprintln!("release-incremental edited_path={}", file.path);
    return edited;
}

fn Assert_Edit_Was_Judged(cold: &Measurement, edited: &Measurement, fresh: &Measurement)
{
    assert!(cold.facts > 1, "the cold population must have more than the edited file's fact");
    assert!(edited.facts > 0 && edited.facts < cold.facts, "the edit must materialize a strict subset of cold work");
    assert_eq!(edited.findings, fresh.findings, "cached reanalysis must agree with a fresh run of the edited input");
    assert!(edited.findings.iter().any(|finding| return finding.summary.contains("Test_Release_Incremental_Ghost")),
        "the injected missing mirror must appear in the judgment, rather than a cached answer ignoring the edit");
    assert!(!cold.findings.iter().any(|finding| return finding.summary.contains("Test_Release_Incremental_Ghost")));
}

#[test]
#[ignore = "requires NOMOS_RELEASE_CORPUS and an explicit release measurement"]
fn Test_Real_Corpus_Should_Measure_Incremental_Checks_And_Match_Fresh_Edit_Judgment()
{
    let (root, sources) = Configured_Corpus();
    eprintln!("release-incremental corpus={} files={} rules={COMPLETENESS_MIRROR},{REQUIREMENT_TRACE_STALENESS}", root.display(), sources.len());
    let mut resident = Resident::Empty();
    let cold = resident.Measure(&root, &sources);
    let unchanged = resident.Measure(&root, &sources);
    assert_eq!(unchanged.facts, 0, "unchanged reanalysis must publish no new facts");
    assert_eq!(unchanged.findings, cold.findings);
    let edited_sources = With_One_Edit(&sources);
    let edited = resident.Measure(&root, &edited_sources);
    let fresh = Resident::Empty().Measure(&root, &edited_sources);
    Assert_Edit_Was_Judged(&cold, &edited, &fresh);
    cold.Print("cold");
    unchanged.Print("unchanged");
    edited.Print("one_file_edit");
    fresh.Print("fresh_edited");
}
