//! What this module promises, exercised.

use super::*;
use super::roots::DOMAIN_VOLUMES;

/// A name for the variable, invented here. The composition root owns the real one;
/// naming it in this file would make every test below look like a test that reads a
/// corpus, and the corpus-gate census counts those.
const VARIABLE: &str = "A_CORPUS_VARIABLE";

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
    let assembly = Assemble_Corpus(&Request(None)).expect("assembles");

    assert!(
        assembly.store.Node_Summary("D-129").expect("queries").is_some(),
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

    let assembly = Assemble_Corpus(&Request(None)).expect("assembles");

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

    let assembly = Assemble_Corpus(&Request(Some(root))).expect("assembles");

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
    std::fs::create_dir_all(&root).expect("creates");

    let assembly = Assemble_Corpus(&Request(Some(root))).expect("assembles");

    assert_eq!(
        assembly.absent.len(),
        3,
        "{}",
        assembly.Describe_Absences()
    );
}

#[test]
fn Test_A_Readable_Volume_Should_Reach_The_Store_And_Not_Be_Reported_Absent()
{
    let root = A_Corpus_With_One_Volume();
    let assembly = Assemble_Corpus(&Request(Some(root))).expect("assembles");
    let (found, _) = assembly
        .store
        .Documents_Named("05_domain_model.md", None)
        .expect("queries");
    let uid = *found.first().expect("the volume is in the store");

    assert_eq!(assembly.store.Table_Lines(uid, None, None).expect("queries").len(), 3);
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
    std::fs::create_dir_all(&volumes).expect("creates");
    std::fs::write(
        volumes.join("05_domain_model.md"),
        "# Canonical domain model\n\n| Concept | Meaning |\n| --- | --- |\n| Ledger | a claim |\n",
    )
    .expect("writes");

    return root;
}
