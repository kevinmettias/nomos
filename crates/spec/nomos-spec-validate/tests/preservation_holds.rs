//! The preservation rules over a real store.

use nomos_spec_ingest::{Ingest_Block_Dispositions, Ingest_Source_Document};
use nomos_spec_store::SpecificationStore;
use nomos_spec_validate::{DECLARED_RULES, Registered, RuleOutcome, Validate};

const DOCUMENT: &str = "---\nid: X\n---\n# Title\n\nOne.\n\n## Section\n\nTwo.\n";

fn Ingested() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", DOCUMENT).expect("ingests");
    return store;
}

fn Dispose_All(store: &mut SpecificationStore)
{
    let dispositions: Vec<(u32, String)> = (1..=4)
        .map(|ordinal| (ordinal, "preserved-verbatim".to_owned()))
        .collect();
    Ingest_Block_Dispositions(store, "a.md", "v14.36", &dispositions).expect("disposes");
}

/// The mirror [`DECLARED_RULES`] names at its declaration site.
///
/// It reconciles the manifest against the identifiers the registered rule objects return,
/// in both directions: a declared rule nothing builds and a built rule nothing declares are
/// each a failure here.
///
/// Renaming this test breaks the claim in `run.rs`, and `nomos check` reports a claim that
/// resolves to nothing as a phantom mirror — Blocking, which is the severity ordering
/// `mirror.rs` sets. Rename both or neither.
#[test]
fn Test_The_Registry_Should_Match_The_Manifest()
{
    let run = Validate(&SpecificationStore::In_Memory().expect("opens"), &Registered());

    // Two empty lists reconcile perfectly. Without these the assertions below would pass
    // having compared nothing, which is the defect this test is now the declared mirror
    // for — and a mirror that can report clean over nothing is worse than none, because
    // the claim at the site reads as coverage.
    assert!(
        !DECLARED_RULES.is_empty(),
        "an empty manifest reconciles against anything"
    );
    assert!(
        !run.results.is_empty(),
        "no rule was registered, so the reconciliation below compares nothing"
    );

    assert!(run.unregistered.is_empty(), "declared but not built: {:?}", run.unregistered);
    assert!(run.undeclared.is_empty(), "built but not declared: {:?}", run.undeclared);
    assert_eq!(run.results.len(), DECLARED_RULES.len());
}

/// The rule that would have caught v15.0's dropped content: a block with no disposition
/// and no omission is a violation, named by document and ordinal.
#[test]
fn Test_An_Undisposed_Block_Should_Violate_Preserve_002()
{
    let store = Ingested();

    let run = Validate(&store, &Registered());

    assert!(!run.Passed(), "blocks with no disposition must not pass");
    let violations = run.Violations();
    assert!(
        violations.iter().any(|violation| violation.subject.contains("a.md#")),
        "the violation must name the block: {violations:?}"
    );
}

/// The negative control. Dispose of every block and the same rule must go quiet.
#[test]
fn Test_Disposed_Blocks_Should_Satisfy_Preserve_002()
{
    let mut store = Ingested();
    Dispose_All(&mut store);

    let run = Validate(&store, &Registered());

    let block_rule = run
        .results
        .iter()
        .find(|result| result.id == "NSV-PRESERVE-002")
        .expect("the rule ran");

    assert!(
        matches!(block_rule.outcome, RuleOutcome::Satisfied { checked: 4 }),
        "{:?}",
        block_rule.outcome
    );
}

/// An omission is the sanctioned exit. A block covered by one satisfies the rule without
/// being preserved — which is what makes the gate block *silent* loss rather than
/// editing.
#[test]
fn Test_An_Omitted_Block_Should_Satisfy_Preserve_002()
{
    let store = Ingested();

    store
        .Connection()
        .execute(
            "INSERT INTO omissions (source_block_uid, reason, justification, decision_record)
             SELECT uid, 'superseded', 'replaced by the v15 records', 'D-129'
             FROM source_blocks",
            [],
        )
        .expect("records omissions");

    let run = Validate(&store, &Registered());
    let block_rule = run
        .results
        .iter()
        .find(|result| result.id == "NSV-PRESERVE-002")
        .expect("the rule ran");

    assert!(
        matches!(block_rule.outcome, RuleOutcome::Satisfied { .. }),
        "an explicitly omitted block is disposed of: {:?}",
        block_rule.outcome
    );
}

/// NSV-PRESERVE-006, the violation v15.0 shipped for all 363 requirements: statements
/// that exist with nothing connecting them to the text they came from.
#[test]
fn Test_A_Statement_Without_Preserved_Lineage_Should_Violate_Preserve_006()
{
    let mut store = Ingested();
    Dispose_All(&mut store);

    store
        .Connection()
        .execute(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('AGT-001', 'requirement', 'canonical', 'record', 'a requirement')",
            [],
        )
        .expect("inserts node");
    store
        .Connection()
        .execute(
            "INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash)
             SELECT uid, 'AGT-001', 'Requirement', 'Nomos shall.', 'sha256:aa'
             FROM nodes WHERE node_id = 'AGT-001'",
            [],
        )
        .expect("inserts statement");

    let run = Validate(&store, &Registered());

    assert!(!run.Passed());
    assert!(
        run.Violations()
            .iter()
            .any(|violation| violation.subject == "AGT-001"),
        "the untraceable statement must be named"
    );
}

/// The negative control for 006.
#[test]
fn Test_A_Traced_Statement_Should_Satisfy_Preserve_006()
{
    let mut store = Ingested();
    Dispose_All(&mut store);

    store
        .Connection()
        .execute(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('AGT-001', 'requirement', 'canonical', 'record', 'a requirement')",
            [],
        )
        .expect("inserts node");
    store
        .Connection()
        .execute(
            "INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash)
             SELECT uid, 'AGT-001', 'Requirement', 'Nomos shall.', 'sha256:aa'
             FROM nodes WHERE node_id = 'AGT-001'",
            [],
        )
        .expect("inserts statement");
    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition, target_statement)
             SELECT b.uid, 'preserved-verbatim', s.uid
             FROM source_blocks b, normative_statements s
             WHERE b.ordinal = 2 AND s.statement_id = 'AGT-001'",
            [],
        )
        .expect("links lineage");

    let run = Validate(&store, &Registered());

    assert!(run.Passed(), "{}\n{:?}", run.Summary(), run.Violations());
}

/// A rule that examined nothing must be visible as such. "0 violations over 0 subjects"
/// and "0 violations over 2533 subjects" print the same and mean opposite things.
#[test]
fn Test_An_Empty_Store_Should_Report_Vacuous_Rules()
{
    let run = Validate(&SpecificationStore::In_Memory().expect("opens"), &Registered());

    assert!(run.Passed(), "an empty store has nothing to violate");
    assert_eq!(
        run.Vacuous_Rules().len(),
        DECLARED_RULES.len(),
        "every rule examined nothing and that must be visible"
    );
}

/// A store with real content must not report vacuous rules — otherwise the guard above
/// would pass on a validator wired to the wrong tables.
#[test]
fn Test_A_Populated_Store_Should_Not_Report_Vacuous_Block_Rules()
{
    let mut store = Ingested();
    Dispose_All(&mut store);

    let run = Validate(&store, &Registered());

    assert!(
        !run.Vacuous_Rules().contains(&"NSV-PRESERVE-002"),
        "the block rule saw nothing in a store with four blocks"
    );
}

/// The payoff for seeding the governing records: the validator that governs them runs
/// over them and reports clean having actually looked. Both heading and block rules must
/// be non-vacuous, because a seed that produced identity without content would satisfy
/// every rule by giving it nothing to examine.
#[test]
fn Test_A_Seeded_Store_Should_Pass_Preservation_Non_Vacuously()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    nomos_spec_store::Seed_Governing_Records(&mut store).expect("seeds");

    let run = Validate(&store, &Registered());

    assert!(run.Passed(), "{}\n{:?}", run.Summary(), run.Violations());

    let vacuous = run.Vacuous_Rules();
    assert!(!vacuous.contains(&"NSV-PRESERVE-001"), "the heading rule examined nothing");
    assert!(!vacuous.contains(&"NSV-PRESERVE-002"), "the block rule examined nothing");
}
