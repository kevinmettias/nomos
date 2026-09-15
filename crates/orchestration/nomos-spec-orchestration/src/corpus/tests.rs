//! What this module promises, exercised.

use super::*;
use super::roots::DOMAIN_VOLUMES;

/// A name for the variable, invented here. The composition root owns the real one;
/// naming it in this file would make every test below look like a test that reads a
/// corpus, and the corpus-gate census counts those.
const VARIABLE: &str = "A_CORPUS_VARIABLE";

/// The three inputs a corpus root owes -- the domain volumes, the statements and the
/// catalogue. A root that exists and holds none of them is three absences, never one
/// collapsed absence.
const INPUTS_A_CORPUS_ROOT_OWES: usize = 3;

/// The rows the one staged domain volume's table holds. `A_Corpus_With_One_Volume` writes a
/// table with a header, a separator and exactly one body row.
const ROWS_IN_THE_STAGED_VOLUME_TABLE: usize = 3;

fn Request(root: Option<PathBuf>) -> CorpusRequest
{
    return CorpusRequest {
        variable: VARIABLE.to_owned(),
        root,
        revision: DEFAULT_REVISION.to_owned(),
    };
}

#[test]
fn Test_Assemble_Corpus_Should_Include_Governing_Records_Even_With_No_Corpus()
{
    let assembly =
        Assemble_Corpus(&Request(None)).expect("assembly reads the embedded records without a corpus root");

    assert!(
        assembly.store.Node_Summary("D-129").expect("the store answers a summary query for a seeded node").is_some(),
        "the embedded records travel with the binary and do not depend on a corpus"
    );
}

/// The property the whole module exists for. An unset variable must produce a named
/// absence, not a smaller store nobody was told about.
#[test]
fn Test_An_Unset_Corpus_Should_Be_An_Absence_That_Names_What_Was_Expected()
{
    use super::roots::CATALOG;
    use super::roots::STATEMENTS;

    let assembly =
        Assemble_Corpus(&Request(None)).expect("assembly reads the embedded records without a corpus root");

    assert!(!assembly.Is_Complete());
    let described = assembly.Describe_Absences();
    assert!(described.contains(VARIABLE), "{described}");
    assert!(described.contains(DOMAIN_VOLUMES), "{described}");
    assert!(described.contains(STATEMENTS), "{described}");
    assert!(described.contains(CATALOG), "{described}");
    assert!(described.contains("not in this store"), "{described}");
}

#[test]
fn Test_A_Corpus_Pointed_At_Nothing_Should_Name_The_Path_It_Was_Pointed_At()
{
    let root = PathBuf::from("no/such/corpus/anywhere");

    let assembly = Assemble_Corpus(&Request(Some(root)))
        .expect("assembly reports a missing corpus root as absences, not a refusal");

    assert!(!assembly.Is_Complete());
    assert!(
        assembly.Describe_Absences().contains("no/such/corpus/anywhere")
            || assembly.Describe_Absences().contains("no\\such\\corpus\\anywhere"),
        "{}",
        assembly.Describe_Absences()
    );
}

/// A corpus root that exists and holds none of the three inputs is three absences,
/// not one. Collapsing them would let two thirds of a corpus read as a whole one.
#[test]
fn Test_An_Empty_Corpus_Root_Should_Account_For_Each_Input_Separately()
{
    let root = std::env::temp_dir().join("nomos-spec-orchestration-empty-corpus-root");
    std::fs::create_dir_all(&root).expect("the scratch corpus root is created under this process's temp dir");

    let assembly = Assemble_Corpus(&Request(Some(root)))
        .expect("assembly tolerates a corpus root holding none of its three inputs");

    assert_eq!(
        assembly.absent.len(),
        INPUTS_A_CORPUS_ROOT_OWES,
        "{}",
        assembly.Describe_Absences()
    );
}

#[test]
fn Test_A_Readable_Volume_Should_Reach_The_Store_And_Not_Be_Reported_Absent()
{
    let root = A_Corpus_With_One_Volume();
    let assembly = Assemble_Corpus(&Request(Some(root)))
        .expect("assembly ingests the one readable volume this root holds");
    let (found, _) = assembly
        .store
        .Documents_Named("05_domain_model.md", None)
        .expect("the store answers a query for a document it was seeded with");
    let uid = *found.first().expect("the volume is in the store");

    let rows = assembly.store.Table_Lines(uid, None, None).expect("the store answers a query for a table it holds");
    assert_eq!(rows.len(), ROWS_IN_THE_STAGED_VOLUME_TABLE);
    assert!(
        assembly
            .absent
            .iter()
            .all(|absence| return !absence.subject.contains("domain volume")),
        "{}",
        assembly.Describe_Absences()
    );
}

/// One readable domain volume, carrying a table so the store has rows to hold.
fn A_Corpus_With_One_Volume() -> std::path::PathBuf
{
    let root = std::env::temp_dir().join("nomos-spec-orchestration-one-volume-corpus");
    let volumes = root.join(DOMAIN_VOLUMES);
    std::fs::create_dir_all(&volumes).expect("the staged volume directory is created under the scratch root");
    std::fs::write(
        volumes.join("05_domain_model.md"),
        "# Canonical domain model\n\n| Concept | Meaning |\n| --- | --- |\n| Ledger | a claim |\n",
    )
    .expect("the staged domain volume is written into the scratch root");

    return root;
}
