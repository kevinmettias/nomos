//! [`Handle_Work_Add`] and its own [`AddResponse`].

use nomos_ledger::{AddRefusal, LedgerItem, PredicateCoverage, RepositoryDeclarations, Territory};
use serde::Serialize;
use nomos_platform::FileSystem;
use std::path::Path;

/// Puts `item` on the board at `directory`, declaring `amending` as the published records it
/// edits rather than allocates, exactly as `nomos work add` would, and hands back a
/// JSON-serializable response.
#[must_use]
pub fn Handle_Work_Add(directory: &Path, item: &LedgerItem, amending: &Territory) -> AddResponse
{
    use super::Ledger_At;
    use nomos_composer_std::{FILE_SYSTEM, LAUNCHER};
    use nomos_work_orchestration::WorkCommand;

    let mut ledger = Ledger_At(directory);

    let outcome = nomos_work_orchestration::Run(
        &WorkCommand::Add { item: Box::new(item.clone()), amending: amending.clone() },
        &mut ledger,
        &LAUNCHER,
        || Repository_Declarations(directory, &FILE_SYSTEM),
    );

    let nomos_work_orchestration::WorkOutcome::Add(added) = outcome
    else
    {
        return AddResponse::Refused {
            cause: super::UNANSWERED_WORK_OUTCOME.to_owned(),
        };
    };

    return AddResponse::From(added);
}

/// What a real `nomos work add` produced, in a shape `serde_json` can hand across a wire.
///
/// `Refused` carries only `cause`, unlike the retryable/judged extra field
/// `ReservationOutcomeResponse`/`AbandonResponse`/`DeclineResponse`/`FinishResponse`
/// each add to theirs: `nomos_ledger::AddRefusal` exposes no single `Is_X`-style method the
/// way `ClaimRefusal`/`FinishRefusal` do -- `crates/host/nomos-cli/src/work/report.rs`'s own
/// `Code_For_Refusal` maps its six variants across three different exit codes, and nothing
/// here reduces that to one bit without becoming a second, uncanonical implementation of a
/// distinction only that function currently draws.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum AddResponse
{
    /// The item was recorded.
    Added,
    /// The item was not recorded.
    Refused
    {
        /// What went wrong, as `AddRefusal::Describe` renders it.
        cause: String,
    },
}

impl AddResponse
{
    pub(crate) fn From(result: Result<(), AddRefusal>) -> Self
    {
        return match result
        {
            Ok(()) => Self::Added,
            Err(refusal) => Self::Refused { cause: refusal.Describe() },
        };
    }
}

/// What the repository holding the board at `directory` declares: the records it has
/// published, and the coverage rules in its root's [`nomos_ledger::PREDICATE_COVERAGE`].
///
/// The published half is this crate's own twin of `nomos-cli`'s walk, below. The coverage half
/// is read by `nomos_ledger::PredicateCoverage::In_Repository`, the one reader both composition
/// roots call, because a declaration file has one format and two readers of one format is how
/// they come to disagree; where the root is stays this crate's to say, as the board's parent.
fn Repository_Declarations(directory: &Path, filesystem: &impl FileSystem) -> RepositoryDeclarations
{
    let coverage = directory.parent().map_or_else(PredicateCoverage::Undeclared, |root| {
        return PredicateCoverage::In_Repository(root, filesystem);
    });

    return RepositoryDeclarations { published: Published_Records(directory, filesystem), coverage };
}

/// Every file this repository has already published as a decision record, as a `Territory`
/// -- a deliberate twin of `crates/host/nomos-cli/src/work.rs`'s own `Published_Records`,
/// not a shared dependency of it, the same "a walk is a composition-root concern"
/// `crate::sources` already documents for Gate's own walk. `nomos_work_orchestration::Run`'s
/// own `declared` closure exists precisely so each composition root can answer this its own
/// way. Unlike the recursive source walk, this listing is one level, so it routes through
/// `OD-PLATFORM-002`'s `Read_Directory` -- the same port the function it twins uses, which is
/// what keeps the two answering the port question the same way rather than only claiming to.
/// Staying a composition-root function is the separate question, and its answer is the one
/// that function already gives: what this hands across `published` is a repository's own
/// convention about where records live, not a ledger-agnostic filesystem concern.
///
/// An unreadable or absent `docs/records` yields `Territory::Empty()` rather than refusing --
/// the one judgement worth stating here, because this repository's usual rule is the
/// opposite. It does not apply: this is input to the open-item comparison, not the check
/// itself, and a tree with no `docs/records` is a ledger being used somewhere that has none.
fn Published_Records(directory: &Path, filesystem: &impl FileSystem) -> Territory
{
    let Some(root) = directory.parent()
    else
    {
        return Territory::Empty();
    };

    let mut published = Record_Files(root, filesystem);
    published.sort();

    return Territory::Of_Files(published);
}

/// Where a repository's own published decision records live, relative to its root.
const RECORD_DIRECTORY: &str = "docs/records";

/// Every file directly under `root`'s record directory, as a territory is spelled --
/// repository-relative and forward-slashed.
///
/// Through `OD-PLATFORM-002`'s `Read_Directory` rather than `std::fs`, which is what makes
/// the twin above a real twin: the function it names does the same, and a twin that reached
/// the filesystem by a different route would be one in name only.
fn Record_Files(root: &Path, filesystem: &impl FileSystem) -> Vec<String>
{
    let Ok(entries) = filesystem.Read_Directory(&root.join(RECORD_DIRECTORY))
    else
    {
        return Vec::new();
    };

    let mut published = Vec::new();
    for entry in entries
    {
        if let Some(name) = entry.file_name().and_then(std::ffi::OsStr::to_str)
        {
            published.push(format!("{RECORD_DIRECTORY}/{name}"));
        }
    }

    return published;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::work::tests_support::{
        BoardWithAClaimableItem, OwnedBoard, Scratch_Board, Scratch_Board_With_A_Claimable_Item,
    };
    use nomos_ledger::{ItemId, ItemKind, ItemOrigin, ItemState};

    #[test]
    fn Test_Handle_Work_Add_Should_Record_A_Real_Well_Formed_Item_On_A_Fresh_Board()
    {
        let OwnedBoard { tree, directory } = Scratch_Board()
            .expect("the temp directory is writable and the scratch ledger is writable");
        let item = Real_New_Item(ItemId::New("SCRATCH-NEW"), "a");

        let response = Handle_Work_Add(&directory, &item, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(matches!(response, AddResponse::Added), "{response:?}");
    }

    #[test]
    fn Test_From_Should_Refuse_An_Item_Whose_Id_Is_Already_On_The_Board()
    {
        let BoardWithAClaimableItem { tree, directory, id } =
            Scratch_Board_With_A_Claimable_Item()
                .expect("the temp directory is writable and the scratch ledger is writable");
        let item = Real_New_Item(id, "b");

        let response = Handle_Work_Add(&directory, &item, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&tree);

        let AddResponse::Refused { cause } = response
        else
        {
            // This fixture builds a board that already carries `id`, so adding it again must
            // refuse -- reaching any other variant here means the duplicate-id check itself
            // stopped enforcing, not a condition this test should assert around.
            panic!("an id already on the board is a real refusal, not a record");
        };
        assert!(cause.contains("already on the ledger"), "{cause}");
    }

    #[test]
    fn Test_Assert_Round_Trips_As_Json_Should_Accept_A_Real_Added_Response()
    {
        let OwnedBoard { tree, directory } = Scratch_Board()
            .expect("the temp directory is writable and the scratch ledger is writable");
        let item = Real_New_Item(ItemId::New("SCRATCH-NEW"), "a");

        let response = Handle_Work_Add(&directory, &item, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&tree);

        crate::test_support::Assert_Round_Trips_As_Json(&response, "added")
            .expect("an added response serializes and parses back as a tagged object");
    }

    #[test]
    fn Test_Handle_Work_Add_Should_Record_An_Item_Reserving_A_Record_Its_Own_Tree_Never_Published()
    {
        // The add lists published records under the board directory's parent, and refuses an
        // item that reserves one of them without amending it. The fixture's tree holds only the
        // board, so nothing there has spent this identifier: the verdict depends on this tree,
        // never on a `docs/records` some other program left in the machine's temp directory.
        let OwnedBoard { tree, directory } = Scratch_Board()
            .expect("the temp directory is writable and the scratch ledger is writable");
        let item = Real_New_Item(ItemId::New("SCRATCH-RECORD"), "docs/records/OD-SCRATCH-001");

        let response = Handle_Work_Add(&directory, &item, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(matches!(response, AddResponse::Added), "{response:?}");
    }

    #[test]
    fn Test_Repository_Declarations_Should_Read_The_Coverage_Rules_Beside_The_Board()
    {
        // The declaration sits at the board directory's parent, which is the tree the fixture
        // owns, so the verdict depends on this tree and never on a declaration some other
        // program left in the machine's temporary directory.
        let OwnedBoard { tree, directory } = Scratch_Board()
            .expect("the temp directory is writable and the scratch ledger is writable");
        std::fs::write(
            tree.join(nomos_ledger::PREDICATE_COVERAGE),
            r#"{ "rules": [ { "record": "OD-EXAMPLE-001", "paths": ["crates/rules"], "requires": ["nomos-cli"] } ] }"#,
        )
        .expect("the fixture's own tree is writable");
        let mut short = Real_New_Item(ItemId::New("SCRATCH-SHORT"), "crates/rules/a.rs");
        short.verification = Some(Predicate_Of(&["cargo", "test", "-p", "nomos-rules"]));
        let mut covered = Real_New_Item(ItemId::New("SCRATCH-COVERED"), "crates/rules/a.rs");
        covered.verification = Some(Predicate_Of(&["cargo", "test", "-p", "nomos-cli"]));

        let declared = Repository_Declarations(&directory, &nomos_composer_std::FILE_SYSTEM);
        let refused = Handle_Work_Add(&directory, &short, &Territory::Empty());
        let added = Handle_Work_Add(&directory, &covered, &Territory::Empty());

        let _ignored = std::fs::remove_dir_all(&tree);

        assert!(matches!(declared.coverage, PredicateCoverage::Declared(ref rules) if rules.len() == 1), "{declared:?}");
        let AddResponse::Refused { cause } = refused
        else
        {
            panic!("a predicate without nomos-cli reaching crates/rules is refused: {refused:?}");
        };
        assert!(cause.contains("nomos-cli") && cause.contains("OD-EXAMPLE-001"), "{cause}");
        assert!(matches!(added, AddResponse::Added), "{added:?}");
    }

    fn Predicate_Of(argv: &[&str]) -> nomos_ledger::VerificationPredicate
    {
        return nomos_ledger::VerificationPredicate::From_String_Arguments(argv.iter().map(|argument| return (*argument).to_owned()).collect());
    }

    /// A well-formed, real `LedgerItem` this test's own -- `id` is the only field a caller
    /// varies below, since every other field is incidental to what `Add` itself judges.
    fn Real_New_Item(id: ItemId, territory_path: &str) -> LedgerItem
    {
        return LedgerItem {
            id,
            title: "t".to_owned(),
            why: "w".to_owned(),
            done_when: "d".to_owned(),
            kind: ItemKind::Capability,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files([territory_path]),
            state: ItemState::Ready,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification: None,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            widened: Vec::new(),
            declined: None,
        };
    }
}
