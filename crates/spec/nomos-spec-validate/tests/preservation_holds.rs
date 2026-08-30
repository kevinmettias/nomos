//! The preservation rules over a real store.

use nomos_spec_ingest::{Ingest_Block_Dispositions, Ingest_Source_Document};
use nomos_spec_store::SpecificationStore;
use nomos_spec_validate::{DECLARED_RULES, Registered, RuleOutcome, Validate_Rules};

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
/// Renaming this test breaks the claim in `validation_run.rs`, and `nomos check` reports a claim that
/// resolves to nothing as a phantom mirror — Blocking, which is the severity ordering
/// `mirror.rs` sets. Rename both or neither.
#[test]
fn Test_The_Registry_Should_Match_The_Manifest()
{
    let run = Validate_Rules(&SpecificationStore::In_Memory().expect("opens"), &Registered());

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

/// `Registered()` itself, independent of the reconciliation `Test_The_Registry_Should_Match_
/// The_Manifest` performs through `Validate_Rules`: it must build exactly one rule object per
/// identifier `DECLARED_RULES` names, with nothing missing and nothing extra.
#[test]
fn Test_Registered_Should_Build_One_Rule_Object_Per_Declared_Identifier()
{
    let rules = Registered();
    let ids: Vec<&str> = rules.iter().map(|rule| rule.Id()).collect();

    assert_eq!(rules.len(), DECLARED_RULES.len(), "{ids:?}");
    for id in DECLARED_RULES
    {
        assert!(ids.contains(id), "Registered() built no rule for {id}: {ids:?}");
    }
}

/// The rule that would have caught v15.0's dropped content: a block with no disposition
/// and no omission is a violation, named by document and ordinal.
#[test]
fn Test_An_Undisposed_Block_Should_Violate_Preserve_002()
{
    let store = Ingested();

    let run = Validate_Rules(&store, &Registered());

    assert!(!run.Is_Passed(), "blocks with no disposition must not pass");
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

    let run = Validate_Rules(&store, &Registered());

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

    Omit_Every_Block(&store);

    let run = Validate_Rules(&store, &Registered());
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

/// Every block covered by an omission, which is the sanctioned exit from PRESERVE-002.
fn Omit_Every_Block(store: &SpecificationStore)
{
    store
        .Connection()
        .execute(
            "INSERT INTO omissions (source_block_uid, reason, justification, decision_record)
             SELECT uid, 'superseded', 'replaced by the v15 records', 'D-129'
             FROM source_blocks",
            [],
        )
        .expect("records omissions");
}

/// A requirement and the statement it carries, with nothing yet connecting the statement to
/// the text it came from.
fn Put_An_Untraced_Statement(store: &SpecificationStore)
{
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
}

/// The lineage row that connects that statement back to the block it was read out of.
fn Trace_It_To_Its_Block(store: &SpecificationStore)
{
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
}

/// NSV-PRESERVE-006, the violation v15.0 shipped for all 363 requirements: statements
/// that exist with nothing connecting them to the text they came from.
#[test]
fn Test_A_Statement_Without_Preserved_Lineage_Should_Violate_Preserve_006()
{
    let mut store = Ingested();
    Dispose_All(&mut store);
    Put_An_Untraced_Statement(&store);

    let run = Validate_Rules(&store, &Registered());

    assert!(!run.Is_Passed());
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
    Put_An_Untraced_Statement(&store);
    Trace_It_To_Its_Block(&store);

    let run = Validate_Rules(&store, &Registered());

    assert!(run.Is_Passed(), "{}\n{:?}", run.Summary(), run.Violations());
}

/// A rule that examined nothing must be visible as such. "0 violations over 0 subjects"
/// and "0 violations over 2533 subjects" print the same and mean opposite things.
#[test]
fn Test_An_Empty_Store_Should_Report_Vacuous_Rules()
{
    let run = Validate_Rules(&SpecificationStore::In_Memory().expect("opens"), &Registered());

    assert!(run.Is_Passed(), "an empty store has nothing to violate");
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

    let run = Validate_Rules(&store, &Registered());

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

    let run = Validate_Rules(&store, &Registered());

    assert!(run.Is_Passed(), "{}\n{:?}", run.Summary(), run.Violations());

    let vacuous = run.Vacuous_Rules();
    assert!(!vacuous.contains(&"NSV-PRESERVE-001"), "the heading rule examined nothing");
    assert!(!vacuous.contains(&"NSV-PRESERVE-002"), "the block rule examined nothing");
}

/// A document whose paragraph body repeats, verbatim, across three sections — the shape
/// `OD-SPEC-004` measured hollowing 44 restored members with one undeclared adjective.
const REPEATED_UNDECLARED: &str = "---\nid: X\n---\n# Title\n\n## A\n\nRepeated body.\n\n## B\n\nRepeated body.\n\n## C\n\nRepeated body.\n";

/// The same shape, but the repeated text is one `FILLER_PATTERNS` already names.
const REPEATED_DECLARED: &str = "---\nid: X\n---\n# Title\n\n## A\n\nThis section groups related specification material.\n\n## B\n\nThis section groups related specification material.\n\n## C\n\nThis section groups related specification material.\n";

/// The rule that would have caught v15.0's hollowed sections: a body carried by
/// `SHARED_BY` or more sections, with no declared pattern naming it, is a template
/// nobody accounted for.
#[test]
fn Test_An_Undeclared_Repeated_Body_Should_Violate_Preserve_004()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", REPEATED_UNDECLARED).expect("ingests");

    let run = Validate_Rules(&store, &Registered());

    assert!(!run.Is_Passed(), "an undeclared template must not pass");
    let violations = run.Violations();
    assert!(
        violations.iter().any(|violation| violation.detail.contains("Repeated body.")),
        "the violation must name the repeated text: {violations:?}"
    );
}

/// The negative control. The same repetition shape, naming a pattern `Is_Filler` already
/// declares, must not violate — the blocklist stays the naming layer, not a second gate.
#[test]
fn Test_A_Declared_Repeated_Body_Should_Satisfy_Preserve_004()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", REPEATED_DECLARED).expect("ingests");

    let run = Validate_Rules(&store, &Registered());

    let template_rule = run
        .results
        .iter()
        .find(|result| result.id == "NSV-PRESERVE-004")
        .expect("the rule ran");

    assert!(
        matches!(template_rule.outcome, RuleOutcome::Satisfied { .. }),
        "a declared pattern must not violate: {:?}",
        template_rule.outcome
    );
}
