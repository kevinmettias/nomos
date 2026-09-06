//! Turning this repository's own committed requirement assessments into the one fact this
//! capability answers.

use crate::assessment::REGISTRY;
use crate::guarantee::{Declared_Guarantee, PROVIDER};
use crate::payload::{Encode_Payload, RequirementTracePayload};
use crate::predicates::{Divergences_With_No_Record, Partials_With_No_Gap, Unresolved_Gaps, Unresolved_Records, Unresolved_Sites};
use crate::registry::Entries;
use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, Guarantee, ProviderId, SnapshotId,
    SubjectId,
};
use nomos_platform::FileSystem;
use std::path::Path;

/// Where in the workspace's history a fact is being produced — the same four-field shape
/// every other real provider's own `FactContext` carries, for the identical reason: these
/// four always travel together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}

/// The one fact this capability's `IncrementalGranularity::WholeWorkspace` ceiling allows,
/// together with the subject it was filed under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}

/// Reads `root`'s own `tests/contract/requirements/` and materializes the one fact this
/// capability answers for the workspace as a whole.
///
/// Never fails: [`Discover_Workspace`] treats every reason it cannot fully answer — no such
/// directory, a directory it cannot enumerate, an entry it cannot parse — as "nothing to
/// report" rather than a materialization failure, per its own module doc. A repository with
/// no committed requirement corpus at all (every repository this rule judges, except this
/// one) produces a fact reporting zero problems, not an absent fact and not a spurious one.
#[must_use]
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> TraceFact
{
    let payload = Discover_Workspace(root, filesystem);
    let subject = nomos_model::Subject_Of_Path("");
    let guarantee = Declared_Guarantee();
    let payload_bytes = Encode_Payload(&payload);
    let key = Compute_Fact_Key(subject, guarantee, context);
    let fact = MaterializedFact {
        identity: key.At(context.generation),
        snapshot: context.snapshot,
        evidence: EvidenceClass::Verified,
        guarantee,
        payload: FactPayload::New(crate::contract::Payload_Schema(), payload_bytes),
    };

    return TraceFact { subject, fact };
}

/// The requirement-trace judgment `root`'s own committed corpus produces against the real
/// tree — every unresolved site, then every unresolved gap, then every unresolved record,
/// then every divergence with no record, then every partial with no gap, per
/// [`RequirementTracePayload`]'s own module doc.
///
/// `root/tests/contract/requirements/` missing entirely, unreadable, or holding an entry
/// this crate's own [`crate::registry::Parse`] refuses are all the identical answer: a
/// payload reporting zero problems. `OD-ANALYSIS-012` named the general shape this
/// collapses — a rule that judged an empty population should be reported apart from one
/// that judged a real population clean — as a decision with nothing built yet
/// (`CheckOutcome::Judged`'s own per-rule population count, its own "What This Record Does
/// Not Do" section says so explicitly); until that mechanism exists, this crate follows the
/// same "an absent or unreadable optional capability answers as if it declared nothing"
/// idiom `nomos-repo-policy`'s own four `nomos.cap.*.policy` providers already use for a
/// missing `standards.json`, rather than inventing a sixth reported case this repository's
/// own contract does not name. What matters for a repository that is not this one is that
/// the ordinary case — no `tests/contract/requirements/` directory at all — never raises a
/// finding: this predicate corpus is nomos's own, not a convention every judged repository
/// is expected to have adopted.
#[must_use]
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> RequirementTracePayload
{
    let Ok(assessments) = Entries(&root.join(REGISTRY), filesystem)
    else
    {
        return RequirementTracePayload::default();
    };

    let mut problems = Unresolved_Sites(root, &assessments, filesystem);
    problems.extend(Unresolved_Gaps(root, &assessments, filesystem));
    problems.extend(Unresolved_Records(root, &assessments, filesystem));
    problems.extend(Divergences_With_No_Record(&assessments));
    problems.extend(Partials_With_No_Gap(&assessments));

    return RequirementTracePayload { problems };
}

/// The key this fact is filed under.
///
/// `semantic_inputs` is empty, deliberately, the same choice every `crates/repository/`-
/// shaped provider's own `Compute_Fact_Key` already makes for the identical reason: this
/// provider's real input is the committed corpus's own current text, which no caller has
/// independently, so a caller building a lookup key has nothing to reconstruct it from.
fn Compute_Fact_Key(subject: SubjectId, guarantee: Guarantee, context: FactContext) -> FactKey
{
    return FactKey {
        contract: crate::contract::Capability(),
        contract_version: crate::contract::CONTRACT_VERSION,
        subject,
        semantic_inputs: InputDigest::Of(&[]),
        provider: ProviderId::New(PROVIDER),
        provider_version: crate::contract::CONTRACT_VERSION,
        guarantee: GuaranteeDigest::Of(&guarantee),
        variant: context.variant,
        configuration: context.configuration,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::payload::{Parse_Payload, Problem, ProblemKind};
    use nomos_contracts::Digest128;
    use nomos_platform::FileSystemError;
    use nomos_platform_std::StdFileSystem;

    #[test]
    fn Test_Materialize_Workspace_Should_Materialize_This_Repositorys_Own_Committed_Corpus()
    {
        let TraceFact { subject, fact } = Materialize_Workspace(&Repository_Root(), Context(), &StdFileSystem);

        assert_eq!(subject, nomos_model::Subject_Of_Path(""));
        assert_eq!(fact.guarantee, Declared_Guarantee());
        assert_eq!(fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
        assert_eq!(fact.payload.schema, crate::contract::Payload_Schema());

        let decoded = Parse_Payload(&fact.payload.bytes).expect("this crate's own encoding");
        assert!(
            decoded.problems.is_empty(),
            "this repository's own committed requirement assessments must all resolve: {:#?}",
            decoded.problems
        );
    }

    #[test]
    fn Test_Discover_Workspace_Should_Report_Nothing_When_The_Registry_Directory_Is_Absent()
    {
        let payload = Discover_Workspace(&std::path::PathBuf::from("this/path/does/not/exist"), &StdFileSystem);

        assert_eq!(payload, RequirementTracePayload::default());
    }

    /// A [`FileSystem`] whose `Read_Directory` always refuses, distinguishing "the registry
    /// directory is absent" (this fixture) from "the registry directory exists and is
    /// merely empty" (`FakeFileSystem` with an empty listing, below) -- both collapse to
    /// the same reported answer today, per this module's own doc, but the two are reached
    /// through different fixtures so a future population-count mechanism can tell them
    /// apart without this test changing.
    struct NoDirectory;

    impl FileSystem for NoDirectory
    {
        fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
        {
            return Err(FileSystemError::NotFound { path: path.display().to_string() });
        }

        fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
        {
            unimplemented!("this fixture never writes")
        }

        fn Exists(&self, _path: &Path) -> bool
        {
            return false;
        }

        fn Read_Directory(&self, path: &Path) -> Result<Vec<std::path::PathBuf>, FileSystemError>
        {
            return Err(FileSystemError::NotFound { path: path.display().to_string() });
        }
    }

    #[test]
    fn Test_Discover_Workspace_Should_Report_Nothing_When_Read_Directory_Refuses()
    {
        let payload = Discover_Workspace(Path::new("anywhere"), &NoDirectory);

        assert_eq!(payload, RequirementTracePayload::default());
    }

    /// A [`FileSystem`] backed by an in-memory directory listing and file map, so a
    /// provider-level fixture can name exactly which entries exist without touching the
    /// real disk.
    struct FakeFileSystem
    {
        listing: Vec<std::path::PathBuf>,
        files: std::collections::BTreeMap<std::path::PathBuf, String>,
    }

    impl FileSystem for FakeFileSystem
    {
        fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
        {
            return self
                .files
                .get(path)
                .cloned()
                .ok_or_else(|| return FileSystemError::NotFound { path: path.display().to_string() });
        }

        fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
        {
            unimplemented!("this fixture never writes")
        }

        fn Exists(&self, path: &Path) -> bool
        {
            return self.files.contains_key(path);
        }

        fn Read_Directory(&self, path: &Path) -> Result<Vec<std::path::PathBuf>, FileSystemError>
        {
            if path.ends_with(REGISTRY)
            {
                return Ok(self.listing.clone());
            }

            return Err(FileSystemError::NotFound { path: path.display().to_string() });
        }
    }

    #[test]
    fn Test_Discover_Workspace_Should_Report_A_Vanished_Site_Against_A_Fake_Tree()
    {
        let root = Path::new("fixture-root");
        let entry = root.join(REGISTRY).join("CHK-003.assessment");
        let filesystem = FakeFileSystem {
            listing: vec![entry.clone()],
            files: std::collections::BTreeMap::from([(
                entry,
                "verdict: Met\nsite: no/such/file.rs#Anything\n".to_owned(),
            )]),
        };

        let payload = Discover_Workspace(root, &filesystem);

        assert_eq!(
            payload,
            RequirementTracePayload {
                problems: vec![Problem {
                    kind: ProblemKind::UnresolvedSite,
                    requirement: "CHK-003".to_owned(),
                    message: "CHK-003: site no/such/file.rs is not a file in this workspace".to_owned(),
                }],
            }
        );
    }

    fn Repository_Root() -> std::path::PathBuf
    {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(std::path::PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
    }

    /// `AGT-001.assessment`'s content immediately before `a8215910` replaced it --
    /// recovered with `git show a8215910~1:tests/contract/requirements/AGT-001.assessment`
    /// from inside a worktree that shares this repository's full history, not
    /// reconstructed from memory. Every citation this entry names -- `LedgerItem`,
    /// `Territory`, `RuleSelector`, `ScopeSelector`, `Claim` and `WorkflowStep` -- exists at
    /// its cited path in this tree today.
    const HISTORICAL_AGT_001: &str = "verdict: Partial\n\
        site: crates/substrate/nomos-ledger/src/item/ledger_item.rs#LedgerItem\n\
        site: crates/substrate/nomos-scope-verification/src/territory.rs#Territory\n\
        site: crates/orchestration/nomos-gate-orchestration/src/policy/rule_selector.rs#RuleSelector\n\
        site: crates/orchestration/nomos-gate-orchestration/src/policy/scope_selector.rs#ScopeSelector\n\
        gap: crates/substrate/nomos-ledger/src/claim.rs#Claim\n\
        gap: crates/contracts/nomos-contracts/src/workflow_step.rs#WorkflowStep\n";

    /// Real everywhere except the one directory a fixture wants to replace -- so a test can
    /// name one assessment's own content without either touching the real committed corpus
    /// (adding a fixture file under `tests/contract/requirements/` would be adding a real,
    /// governed assessment) or fabricating the six real site files this historical entry
    /// cites, which is the whole property [`Test_A_Historical_Assessment_Whose_Citations_
    /// All_Resolve_Should_Not_Be_Reported`] below needs: every site and gap lookup falls
    /// through to [`StdFileSystem`] and reads this real, current tree.
    struct RealTreeExceptRequirements
    {
        directory: std::path::PathBuf,
        file: std::path::PathBuf,
        text: String,
    }

    impl FileSystem for RealTreeExceptRequirements
    {
        fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
        {
            if path == self.file
            {
                return Ok(self.text.clone());
            }

            return StdFileSystem.Read_To_String(path);
        }

        fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
        {
            unimplemented!("this fixture never writes")
        }

        fn Exists(&self, path: &Path) -> bool
        {
            return StdFileSystem.Exists(path);
        }

        fn Read_Directory(&self, path: &Path) -> Result<Vec<std::path::PathBuf>, FileSystemError>
        {
            if path == self.directory
            {
                return Ok(vec![self.file.clone()]);
            }

            return StdFileSystem.Read_Directory(path);
        }
    }

    /// The boundary `OD-TRACE-002` already drew, proven rather than merely stated:
    /// citation-existence checking catches a vanished or renamed citation, never a
    /// citation whose prose quietly became wrong about code that is still there under the
    /// same name. This historical entry's own verdict was later corrected
    /// (`P53-GATE-020-SHARED-DERIVATION-EXISTS`, `a8215910`) for exactly that reason -- a
    /// shared derivation the prose described had since been superseded -- and every one of
    /// its six citations was true both before and after that correction. This test is not
    /// evidence the old *verdict* was right; it is evidence that a rule built only from
    /// [`crate::predicates`] cannot tell a stale verdict from a current one when every
    /// citation it names still resolves, which is exactly what this predicate corpus was
    /// designed to check and exactly the gap semantic drift leaves open.
    #[test]
    fn Test_A_Historical_Assessment_Whose_Citations_All_Resolve_Should_Not_Be_Reported()
    {
        let root = Repository_Root();
        let directory = root.join(REGISTRY);
        let filesystem = RealTreeExceptRequirements {
            file: directory.join("AGT-001.assessment"),
            directory,
            text: HISTORICAL_AGT_001.to_owned(),
        };

        let payload = Discover_Workspace(&root, &filesystem);

        assert!(
            payload.problems.is_empty(),
            "every citation in this historical entry resolves in the current tree, so a \
             rule that only checks citation existence must report nothing about it: {:#?}",
            payload.problems
        );
    }

    #[test]
    fn Test_A_Fact_Key_Should_Depend_On_The_Guarantee()
    {
        let subject = nomos_model::Subject_Of_Path("");
        let weaker = Guarantee::New(
            nomos_contracts::FactVariant::Syntactic,
            nomos_contracts::Assurance::Unsound,
            nomos_contracts::Assurance::Unknown,
            nomos_contracts::IncrementalGranularity::WholeWorkspace,
        );

        let strong_key = Compute_Fact_Key(subject, Declared_Guarantee(), Context());
        let weak_key = Compute_Fact_Key(subject, weaker, Context());

        assert_ne!(strong_key.Digest(), weak_key.Digest(), "two offers of the same subject at different guarantees must file apart");
    }

    const VARIANT_DIGEST_FILL: u8 = 2;
    const CONFIGURATION_DIGEST_FILL: u8 = 3;

    fn Context() -> FactContext
    {
        return FactContext {
            snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
            configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([
                CONFIGURATION_DIGEST_FILL;
                Digest128::BYTE_LENGTH
            ])),
            generation: GenerationId::INITIAL,
        };
    }
}
