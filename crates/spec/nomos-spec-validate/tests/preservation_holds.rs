//! The preservation rules over a real store.

use nomos_spec_ingest::{Ingest_Block_Dispositions, Ingest_Source_Document, Is_Template_Eligible,
                        SHARED_BY, TEMPLATE_FLOOR};
use nomos_spec_store::SpecificationStore;
use nomos_spec_validate::{DECLARED_RULES, Registered, RuleOutcome, Validate_Rules};

const DOCUMENT: &str = "---\nid: X\n---\n# Title\n\nOne.\n\n## Section\n\nTwo.\n";

/// The blocks [`DOCUMENT`] segments into: the title, the paragraph under it, the second heading
/// and the paragraph under that. [`Dispose_All`] disposes of exactly these, and `checked` on the
/// block rule reports the same four back.
const DOCUMENT_BLOCKS: u32 = 4;

fn Ingested() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("in-memory opens no file, so only the schema can fail");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", DOCUMENT).expect("the store is empty and in memory");
    return store;
}

fn Dispose_All(store: &mut SpecificationStore)
{
    let dispositions: Vec<(u32, String)> = (1..=DOCUMENT_BLOCKS)
        .map(|ordinal| (ordinal, "preserved-verbatim".to_owned()))
        .collect();
    Ingest_Block_Dispositions(store, "a.md", "v14.36", &dispositions).expect("the document this names was ingested above");
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
    let store = SpecificationStore::In_Memory()
        .expect("in-memory opens no file, so only the schema can fail");
    let run = Validate_Rules(&store, &Registered());

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
/// The_Manifest` performs through `Validate_Rules`: it must build exactly one rule per
/// identifier `DECLARED_RULES` names, with nothing missing and nothing extra.
#[test]
fn Test_Registered_Should_Build_One_Rule_Per_Declared_Identifier()
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
        matches!(block_rule.outcome, RuleOutcome::Satisfied { checked: DOCUMENT_BLOCKS }),
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
    let store = SpecificationStore::In_Memory()
        .expect("in-memory opens no file, so only the schema can fail");
    let run = Validate_Rules(&store, &Registered());

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
    let mut store = SpecificationStore::In_Memory().expect("in-memory opens no file, so only the schema can fail");
    nomos_spec_store::Seed_Governing_Records(&mut store).expect("the records are compiled in and inserted as written");

    let run = Validate_Rules(&store, &Registered());

    assert!(run.Is_Passed(), "{}\n{:?}", run.Summary(), run.Violations());

    let vacuous = run.Vacuous_Rules();
    assert!(!vacuous.contains(&"NSV-PRESERVE-001"), "the heading rule examined nothing");
    assert!(!vacuous.contains(&"NSV-PRESERVE-002"), "the block rule examined nothing");
}

/// A document whose paragraph body repeats, verbatim, across three sections — the shape
/// `OD-SPEC-004` measured hollowing 44 restored members with one undeclared adjective.
///
/// The body was `Repeated body.` until the eligibility floor landed. Fourteen characters
/// is below [`TEMPLATE_FLOOR`], so this fixture would have gone on passing while proving
/// nothing: the rule would have dismissed it for its length and the test would have read
/// that as the rule working. It repeats something substantive now, and
/// `Test_An_Undeclared_Repeated_Body_Should_Violate_Preserve_004` asserts the length so
/// the same thing cannot happen quietly again.
const REPEATED_UNDECLARED: &str = "---\nid: X\n---\n# Title\n\n## A\n\nThis paragraph is repeated verbatim below.\n\n## B\n\nThis paragraph is repeated verbatim below.\n\n## C\n\nThis paragraph is repeated verbatim below.\n";

/// The same shape, but the repeated text is one `FILLER_PATTERNS` already names.
const REPEATED_DECLARED: &str = "---\nid: X\n---\n# Title\n\n## A\n\nThis section groups related specification material.\n\n## B\n\nThis section groups related specification material.\n\n## C\n\nThis section groups related specification material.\n";

/// The rule that would have caught v15.0's hollowed sections: a body carried by
/// `SHARED_BY` or more sections, with no declared pattern naming it, is a template
/// nobody accounted for.
#[test]
fn Test_An_Undeclared_Repeated_Body_Should_Violate_Preserve_004()
{
    let mut store = SpecificationStore::In_Memory().expect("in-memory opens no file, so only the schema can fail");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", REPEATED_UNDECLARED).expect("the store is empty and in memory");

    let run = Validate_Rules(&store, &Registered());

    assert!(
        Is_Template_Eligible(REPEATED_BODY),
        "this fixture repeats {} characters, under the floor of {TEMPLATE_FLOOR}, so the \
         rule would dismiss it for its length and the assertion below would be about \
         nothing",
        REPEATED_BODY.chars().count()
    );
    assert!(!run.Is_Passed(), "an undeclared template must not pass");
    let violations = run.Violations();
    assert!(
        violations.iter().any(|violation| violation.detail.contains("This paragraph is repeated verbatim below.")),
        "the violation must name the repeated text: {violations:?}"
    );
}

/// The negative control. The same repetition shape, naming a pattern `Is_Filler` already
/// declares, must not violate — the blocklist stays the naming layer, not a second gate.
#[test]
fn Test_A_Declared_Repeated_Body_Should_Satisfy_Preserve_004()
{
    let mut store = SpecificationStore::In_Memory().expect("in-memory opens no file, so only the schema can fail");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", REPEATED_DECLARED).expect("the store is empty and in memory");

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

/// The normalized hash the fixture's one repeated body groups under, so a test can declare it.
fn Repeated_Hash(store: &SpecificationStore) -> String
{
    return store
        .Connection()
        .query_row(
            "SELECT normalized_hash FROM source_blocks WHERE kind = 'prose'
             GROUP BY normalized_hash HAVING count(*) >= ?1",
            [SHARED_BY],
            |row| return row.get(0),
        )
        .expect("the fixture repeats one body across SHARED_BY or more sections");
}

/// One declaration row: the normalized hash of the repeated body, the role it plays, and the
/// multiplicity that role implies.
fn Declare_Repeated_Text(store: &mut SpecificationStore, normalized_hash: &str, role: &str, multiplicity: i64)
{
    store
        .Connection()
        .execute(
            "INSERT INTO repeated_text_declarations (normalized_hash, role, multiplicity) VALUES (?1, ?2, ?3)",
            rusqlite::params![normalized_hash, role, multiplicity],
        )
        .expect("the declaration table accepts this row");
}

/// A block declared for three named roles, appearing in exactly those three places, is
/// admitted and is not filler.
#[test]
fn Test_A_Declared_Repetition_Should_Satisfy_Preserve_004()
{
    let mut store = SpecificationStore::In_Memory().expect("in-memory opens no file, so only the schema can fail");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", REPEATED_UNDECLARED).expect("the store is empty and in memory");

    let hash = Repeated_Hash(&store);
    Declare_Repeated_Text(&mut store, &hash, "edition-line", 1);
    Declare_Repeated_Text(&mut store, &hash, "suite-title", 1);
    Declare_Repeated_Text(&mut store, &hash, "volume-abstract", 1);

    let run = Validate_Rules(&store, &Registered());
    let template_rule = run.results.iter().find(|result| result.id == "NSV-PRESERVE-004").expect("the rule ran");

    assert!(
        matches!(template_rule.outcome, RuleOutcome::Satisfied { .. }),
        "a repetition admitted by its declaration must not violate: {:?}",
        template_rule.outcome
    );
}

/// The falsifier: the same declaration, but the block appears a fourth time. The three
/// declared occurrences are admitted; the fourth is still an undeclared repetition.
#[test]
fn Test_A_Declared_Repetition_With_An_Extra_Occurrence_Should_Violate_Preserve_004()
{
    let document = Document_With_Repeated_Body(REPEATED_BODY, DELIBERATE_SECTIONS + 1);
    let mut store = SpecificationStore::In_Memory().expect("in-memory opens no file, so only the schema can fail");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", &document).expect("the store is empty and in memory");

    let hash = Repeated_Hash(&store);
    Declare_Repeated_Text(&mut store, &hash, "edition-line", 1);
    Declare_Repeated_Text(&mut store, &hash, "suite-title", 1);
    Declare_Repeated_Text(&mut store, &hash, "volume-abstract", 1);

    let run = Validate_Rules(&store, &Registered());
    let violations = run.Violations();

    assert!(!run.Is_Passed(), "an extra occurrence beyond the declaration must violate");
    assert!(
        violations.iter().any(|violation| violation.detail.contains("declared for 3 occurrences but appears 4")),
        "the violation must name the declared count and the actual count: {violations:?}"
    );
}

/// The body [`REPEATED_UNDECLARED`] repeats, named so the test can measure it.
const REPEATED_BODY: &str = "This paragraph is repeated verbatim below.";

/// Five sections carrying the longest fragment that collided by accident in a real corpus.
const BELOW_FLOOR: &str = "Recommended language: Rust one";

/// Three sections carrying the shortest body a real corpus repeated on purpose.
const AT_STRUCTURAL_MINIMUM: &str = "This volume owns the domain material shown.";

// The two measured populations the floor separates, one below it and one above. Each is a body
// length and the number of sections carrying it: thirty characters is the longest body that
// collided by accident anywhere in the sibling suites, carried by more sections than the
// threshold needs so that length is what dismisses it; forty-three is the shortest the domain
// volumes repeat on purpose, carried by exactly the threshold so that length is what admits it.
const ACCIDENTAL_BODY_CEILING: usize = 30;
const ACCIDENTAL_SECTIONS: usize = 5;
const DELIBERATE_BODY_FLOOR: usize = 43;
const DELIBERATE_SECTIONS: usize = 3;

/// `body` repeated across `sections` sections of one document.
fn Document_With_Repeated_Body(body: &str, sections: usize) -> String
{
    let mut document = String::from("---\nid: X\n---\n# Title\n");
    for section in 0..sections
    {
        let title = char::from(b'A'.saturating_add(u8::try_from(section).unwrap_or(0)));
        document.push_str(&format!("\n## {title}\n\n{body}\n"));
    }

    return document;
}

/// The outcome `NSV-PRESERVE-004` reached over one document.
fn Template_Outcome(document: &str) -> RuleOutcome
{
    let mut store = SpecificationStore::In_Memory().expect("in-memory opens no file, so only the schema can fail");
    Ingest_Source_Document(&mut store, "a.md", "v14.36", document).expect("the store is empty and in memory");

    return Validate_Rules(&store, &Registered())
        .results
        .into_iter()
        .find(|result| return result.id == "NSV-PRESERVE-004")
        .expect("the rule ran")
        .outcome;
}

/// Repetition is necessary and not sufficient, which is the whole of `OD-SPEC-004` version 3.
///
/// [`ACCIDENTAL_BODY_CEILING`] characters is the longest body that collided by accident
/// anywhere in the sibling suites, and it is carried here by [`ACCIDENTAL_SECTIONS`] sections
/// rather than the three the threshold needs — so nothing about this document is marginal on
/// repetition. It is dismissed on length or the floor is not doing its job.
#[test]
fn Test_A_Body_Below_The_Floor_Should_Not_Violate_Preserve_004_However_Often_It_Repeats()
{
    assert_eq!(
        BELOW_FLOOR.chars().count(),
        ACCIDENTAL_BODY_CEILING,
        "this fixture is the measured ceiling of the accidental population and has drifted"
    );

    let repeated = Document_With_Repeated_Body(BELOW_FLOOR, ACCIDENTAL_SECTIONS);
    let outcome = Template_Outcome(&repeated);

    assert!(
        matches!(outcome, RuleOutcome::Satisfied { .. }),
        "a {ACCIDENTAL_BODY_CEILING}-character body repeated {ACCIDENTAL_SECTIONS} times is a \
         lexical collision, not a form letter: {outcome:?}"
    );
}

/// The other side of the same band, and the reason the floor is not higher.
///
/// [`DELIBERATE_BODY_FLOOR`] characters is the shortest body the domain volumes repeat on
/// purpose. A floor that dismissed it would be giving the right answer for the wrong reason:
/// that text is contentful, and what admits it is a corpus declaring the roles it is projected
/// into, which is a separate mechanism. Until that exists this must still report.
#[test]
fn Test_A_Body_At_The_Shortest_Deliberate_Length_Should_Still_Violate_Preserve_004()
{
    assert_eq!(
        AT_STRUCTURAL_MINIMUM.chars().count(),
        DELIBERATE_BODY_FLOOR,
        "this fixture is the measured floor of the deliberate population and has drifted"
    );

    let repeated = Document_With_Repeated_Body(AT_STRUCTURAL_MINIMUM, DELIBERATE_SECTIONS);
    let outcome = Template_Outcome(&repeated);

    assert!(
        matches!(outcome, RuleOutcome::Violated { .. }),
        "the floor must not swallow the shortest body a corpus repeats by design: {outcome:?}"
    );
}

/// The boundary decided, rather than left where an off-by-one would put it.
///
/// Both populations this floor separates are measured, and neither lies next to it — 30 and
/// 43 against a floor of 36. So nothing real turns on whether the comparison is `>=` or `>`,
/// and that is exactly why it is pinned here: an off-by-one would never show up in either
/// corpus, and would surface years later against a body nobody had measured.
#[test]
fn Test_The_Floor_Should_Admit_A_Body_Of_Exactly_Its_Own_Length()
{
    let under: String = "x".repeat(TEMPLATE_FLOOR.saturating_sub(1));
    let at: String = "x".repeat(TEMPLATE_FLOOR);

    assert!(!Is_Template_Eligible(&under), "one character under the floor is not eligible");
    assert!(Is_Template_Eligible(&at), "a body of exactly the floor is eligible");
}

/// How many times the test below repeats `"ab "`: twenty-four letters and twelve spaces, which
/// normalize to thirty-five characters and pad to eighty-four.
const NORMALIZING_REPEATS: usize = 12;

/// Length is counted over the normalized text, not the text as written.
///
/// `normalized_hash` is what groups a template, so two bodies that differ only in whitespace
/// are one template — and if eligibility were counted on the raw text, one of them could be
/// eligible while the other was not, which would make the answer depend on which member of
/// the group the query happened to sample.
#[test]
fn Test_Eligibility_Should_Be_Counted_Over_The_Normalized_Text()
{
    // This many of them normalize to 35 characters, one under the floor, and the padded
    // form is 84 before normalizing. A fixture comfortably over the floor in both forms
    // would agree under either implementation and assert nothing -- which is what the
    // first version of this test did.
    let compact: String = "ab ".repeat(NORMALIZING_REPEATS);
    let padded = compact.replace(' ', "     ");

    assert!(
        padded.chars().count() > TEMPLATE_FLOOR,
        "the padded form must clear the floor on raw length, or counting raw length would \
         give the same answer and this test would be about nothing"
    );
    assert!(
        !Is_Template_Eligible(&compact),
        "35 normalized characters is under the floor"
    );
    assert!(
        !Is_Template_Eligible(&padded),
        "padding the same 35 characters with whitespace must not buy eligibility: counted \
         raw it is {} characters",
        padded.chars().count()
    );
}
