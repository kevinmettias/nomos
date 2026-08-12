//! Reading and writing the document, and refusing the ones that cannot be trusted.
//!
//! A ledger that is missing reads as empty; one that is corrupt is an error. Those must not
//! be the same answer, because the first is a board nobody has started and the second is a
//! board whose contents were lost.

use crate::common::*;

/// A missing ledger is a repository that has not started tracking work. A *corrupt*
/// ledger is somebody's roadmap that got damaged, and treating it as empty would let
/// every agent claim everything.
#[test]
fn Test_A_Corrupt_Ledger_Should_Be_An_Error_Not_An_Empty_One()
{
    let directory = Temp_Dir("corrupt");
    let ledger = Ledger_At(&directory, &AT_NOW);

    std::fs::write(directory.join("ledger.json"), "{ this is not json").expect("write");

    let error = ledger.Load().expect_err("a corrupt ledger must not read as empty");

    assert!(matches!(error, LedgerError::Malformed { .. }));
}

#[test]
fn Test_A_Missing_Ledger_Should_Read_As_Empty()
{
    let directory = Temp_Dir("missing");
    let ledger = Ledger_At(&directory, &AT_NOW);

    let document = ledger.Load().expect("a missing ledger is not an error");

    assert!(document.items.is_empty());
}

/// An invalid document must be refused *before* it is written. A ledger that is written
/// and then found invalid is one somebody has to repair by hand, and until they do
/// every agent is reading something the system itself says is wrong.
#[test]
fn Test_Saving_An_Invalid_Ledger_Should_Be_Refused_Before_The_Write()
{
    let directory = Temp_Dir("refuse-invalid");
    let ledger = Ledger_At(&directory, &AT_NOW);

    let mut blocked = Item("T-1", &["src/a.rs"]);
    blocked.state = ItemState::Blocked;

    let error = ledger
        .Save(&Document(vec![blocked]))
        .expect_err("an invalid ledger must not be persisted");

    assert!(matches!(error, LedgerError::Invalid { .. }));
    assert!(
        !directory.join("ledger.json").exists(),
        "nothing may be written when validation fails"
    );
}

/// Round-tripping must be lossless. If it is not, an agent's claim silently changes
/// meaning the next time somebody else writes the file.
#[test]
fn Test_The_Ledger_Should_Round_Trip_Losslessly()
{
    let directory = Temp_Dir("round-trip");
    let ledger = Ledger_At(&directory, &AT_NOW);

    let original = Document(vec![
        Held_By(Item("T-1", &["src/a.rs", "src/b.rs"]), "agent-a", NOW + 3_600),
        Item("T-2", &["src/c.rs"]),
    ]);

    ledger.Save(&original).expect("valid");
    let reloaded = ledger.Load().expect("readable");

    assert_eq!(reloaded, original);
}

// ---------------------------------------------------------------------------
// Durability — a build that cannot account for the document does not write it.
//
// P10-STALE-WRITER, decided in `OD-LEDGER-008`. The test above this comment is the reason
// this section exists: round-tripping a document built from `LedgerItem` only ever contains
// keys `LedgerItem` declares, so it asserted that the *declared* shape survives, which was
// never in question. Everything below writes JSON to disk by hand.
// ---------------------------------------------------------------------------

/// One item as JSON text, with `extra` spliced in as further keys on it.
///
/// Written out rather than built from [`LedgerItem`] on purpose. A fixture built from the type
/// cannot carry a key the type does not declare, and the key it does not declare is exactly what
/// an older binary was dropping.
fn Raw_Ledger(schema_version: u32, extra: &str) -> String
{
    return format!(
        "{{\n  \"schema_version\": {schema_version},\n  \"items\": [\
         {{\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\"patterns\":[]}},\
         \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
         \"verification\":null,\"verified\":null,\"abandoned\":[]{extra}}}\
         ]\n}}\n"
    );
}

/// The subject. A key no declared type recognises must refuse the read.
///
/// Serde's default is to ignore it, and ignoring it is what cost an abandonment reason on
/// 2026-08-09: the key was dropped on the way in and therefore absent on the way out, so a
/// lossy write and a clean one were the same event at the surface reporting them.
#[test]
fn Test_A_Ledger_Carrying_An_Undeclared_Key_Should_Not_Load()
{
    let directory = Temp_Dir("undeclared-key");
    let ledger = Ledger_At(&directory, &AT_NOW);

    let raw = Raw_Ledger(
        SCHEMA_VERSION,
        ",\"a_field_this_build_does_not_know\":{\"holder\":\"agent-a\"}",
    );
    std::fs::write(directory.join("ledger.json"), raw).expect("write");

    let error = ledger
        .Load()
        .expect_err("a document carrying a key this build cannot account for must not load");

    assert!(
        format!("{error}").contains("a_field_this_build_does_not_know"),
        "the refusal must name the key it could not account for: {error}"
    );
}

/// Every object in the document refuses, not a list of types somebody maintained.
///
/// A hand-written list of the types carrying the attribute is only as complete as the hand —
/// `OD-COMPLETENESS-001` — and the attribute is invisible to the public-surface snapshot, so
/// nothing else in this workspace can see it removed from one nested type. The universe is
/// therefore derived from a fully populated document: serialize it, walk it, and probe every
/// object node that walk finds.
///
/// This is what covers a type that does not exist yet. A field added to [`LedgerItem`] whose
/// own container forgot the attribute fails here without anybody extending this test.
#[test]
fn Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key()
{
    let whole =
        serde_json::to_value(Document(vec![Fully_Populated()])).expect("the document serializes");

    let mut pointers = Vec::new();
    Object_Pointers(&whole, "", &mut pointers);
    for pointer in &pointers
    {
        assert!(
            Probed(&whole, pointer).is_err(),
            "an undeclared key was accepted at `{pointer}`, so a build that predates a field \
             there would drop it and write the document back"
        );
    }

    assert!(
        pointers.len() >= 11,
        "only {} object(s) were probed, so the fixture below has stopped being fully \
         populated — the guard did not shrink, the universe did",
        pointers.len()
    );
}

/// The same document with one undeclared key inserted at `pointer`, read back strictly.
fn Probed(whole: &serde_json::Value, pointer: &str) -> Result<LedgerDocument, serde_json::Error>
{
    let mut probed = whole.clone();
    probed
        .pointer_mut(pointer)
        .expect("every pointer was collected from this same value")
        .as_object_mut()
        .expect("only object nodes were collected")
        .insert("nomos_probe".to_owned(), serde_json::Value::Null);

    return serde_json::from_value::<LedgerDocument>(probed);
}

/// An item with every optional field set and every list non-empty.
///
/// The walk above reaches an object only if the document actually contains one, so an empty
/// list serializes as `[]`, contributes no node, and leaves the guard silent about whatever
/// type lives inside it. Populating everything is what makes the count at the end mean
/// something.
fn Fully_Populated() -> LedgerItem
{
    let held = Item("T-1", &["src/a.rs"]);
    let mut item = Held_By(held, "agent-a", NOW + 3_600);
    item.state = ItemState::Blocked;
    item.blocked = Some(Blocker::Dependency {
        items: vec![ItemId::New("T-0")],
    });
    item.depends_on = vec![ItemId::New("T-0")];
    item.territory = ItemTerritory::Of_Files(["src/a.rs"]).With_Pattern("src/**");
    item.verification = Some(VerificationPredicate::New(vec!["cargo".to_owned()]));
    item.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned()],
        exit_code: 0,
        output_tail: "ok".to_owned(),
        verified_at: At(NOW),
        gate: Some(GateOutcome {
            argv: vec!["cargo".to_owned(), "--version".to_owned()],
            exit_code: 0,
        }),
    });
    item.abandoned = vec![Abandonment {
        holder: "agent-b".to_owned(),
        reason: "went to look at something else".to_owned(),
        abandoned_at: At(NOW),
    }];
    // The nested type `OD-LEDGER-012` added.
    item.displaced = vec![Claim {
        holder: "dead-agent".to_owned(),
        acquired_at: At(NOW),
        lease_expires_at: At(NOW + 60),
    }];

    return item;
}

/// Every object node in a value, as JSON pointers.
///
/// The document's shape rather than a list of types, which is the point of the test above.
fn Object_Pointers(value: &serde_json::Value, at: &str, found: &mut Vec<String>)
{
    match value
    {
        serde_json::Value::Object(fields) =>
        {
            found.push(at.to_owned());
            for (key, nested) in fields
            {
                Object_Pointers(nested, &format!("{at}/{key}"), found);
            }
        }
        serde_json::Value::Array(elements) =>
        {
            for (index, nested) in elements.iter().enumerate()
            {
                Object_Pointers(nested, &format!("{at}/{index}"), found);
            }
        }
        _ =>
        {}
    }
}

/// A file newer than this build says so, rather than being called damaged.
///
/// The two failures reach `Load` by the same route and the operator's next action is opposite:
/// rebuild the reader, or repair the file. Reporting the first as the second sends somebody to
/// edit a file that is correct.
#[test]
fn Test_A_Ledger_Newer_Than_This_Build_Should_Say_So_Rather_Than_Malformed()
{
    // The spliced key was `"displaced":[]` when this test was written, chosen as a field a
    // later build might add. `OD-LEDGER-012` added it, so it became declared and the parse it
    // was here to make fail started succeeding. Named for what it is instead, which is the
    // same lesson `Raw_Ledger`'s own fixture learned: a probe key must not be one the schema
    // can catch up with.
    let raw = Raw_Ledger(9_999, ",\"a_field_this_build_does_not_know\":[]");
    let error = Load_Failure("newer-than-build", &raw);

    let LedgerError::Unrecognized {
        understood, found, ..
    } = &error
    else
    {
        panic!("a file newer than this build must not be reported as damaged: {error}");
    };
    assert_eq!(*understood, SCHEMA_VERSION);
    assert_eq!(*found, 9_999);
    Names_The_Versions_And_The_Remedy(&format!("{error}"));
}

/// Both numbers and the action, because a message naming only one of them leaves the reader
/// with nothing to compare and no next step.
fn Names_The_Versions_And_The_Remedy(said: &str)
{
    assert!(said.contains("9999"), "{said}");
    assert!(said.contains(&format!("{SCHEMA_VERSION}")), "{said}");
    assert!(
        said.to_lowercase().contains("rebuild"),
        "the message must name the remedy: {said}"
    );
}

/// What `Load` says about a document written by hand.
///
/// These files are the ones `Save` refuses to produce, so writing the bytes directly is the
/// only way to reach the arm under test — and the tree goes away with the value returned.
fn Load_Failure(name: &str, raw: &str) -> LedgerError
{
    let directory = Temp_Dir(name);
    std::fs::write(directory.join("ledger.json"), raw).expect("write");

    return Ledger_At(&directory, &AT_NOW)
        .Load()
        .expect_err("this document must not load");
}

/// The control that keeps the distinction honest.
///
/// Without it, `Unrecognized` could swallow every parse failure and an operator with a genuinely
/// damaged file would be told to rebuild a binary that is fine. Asserted on the variant rather
/// than only on `is_err`, which `Test_A_Corrupt_Ledger_Should_Be_An_Error_Not_An_Empty_One`
/// already covers from the other side.
#[test]
fn Test_A_Ledger_That_Is_Merely_Broken_Should_Still_Be_Malformed()
{
    let directory = Temp_Dir("merely-broken");
    let ledger = Ledger_At(&directory, &AT_NOW);

    std::fs::write(directory.join("ledger.json"), "{ this is not json").expect("write");

    let error = ledger.Load().expect_err("a broken document must not load");

    assert!(
        matches!(error, LedgerError::Malformed { .. }),
        "a damaged file must not be reported as one this build is too old for: {error}"
    );
}

/// What is written says what this build understands, not what the file it read said.
///
/// The one state the schema version cannot explain is a document carrying keys a build invented
/// while claiming the older number it loaded: an older reader then fails the strict parse, probes
/// the version, finds nothing newer than itself, and reports the file damaged — sending somebody
/// to repair a file that is correct. Stamping on the way out is what forecloses that, and the
/// version is the only field in the document that `Save` does not take from its argument.
///
/// Asserted with a version this build could not have produced, because a fixture already holding
/// `SCHEMA_VERSION` cannot tell a stamp from an echo while the constant sits at one.
#[test]
fn Test_Saving_Should_Stamp_The_Version_This_Build_Understands()
{
    let directory = Temp_Dir("stamps-version");
    let ledger = Ledger_At(&directory, &AT_NOW);

    ledger
        .Save(&LedgerDocument {
            schema_version: 9_999,
            items: vec![Item("T-1", &["src/a.rs"])],
        })
        .expect("valid");

    let written = std::fs::read_to_string(directory.join("ledger.json")).expect("readable");

    assert!(
        written.contains(&format!("\"schema_version\": {SCHEMA_VERSION}")),
        "the write echoed the version it was handed rather than stamping this build's:\n{written}"
    );
}

/// The other direction, and the control `done_when` asks for by name.
///
/// A guard that refused every write would stop the board rather than protect it, and the
/// asymmetry is the whole trade: a new build still reads an old file, because every added field
/// carries `serde(default)`, and an old build no longer reads a new one. Every item in this
/// repository's own committed ledger was written without `abandoned` at some point, and this is
/// that case.
#[test]
fn Test_A_Document_Written_Before_A_Field_Existed_Should_Still_Load()
{
    let directory = Temp_Dir("older-than-build");
    std::fs::write(
        directory.join("ledger.json"),
        "{\n  \"schema_version\": 1,\n  \"items\": [\
         {\"id\":\"T-1\",\"title\":\"item T-1\",\"why\":\"because\",\
         \"done_when\":\"the tests pass\",\
         \"territory\":{\"resolution\":\"File\",\"paths\":[\"src/a.rs\"],\"patterns\":[]},\
         \"state\":\"Ready\",\"depends_on\":[],\"blocked\":null,\"claim\":null,\
         \"verification\":null,\"verified\":null}\
         ]\n}\n",
    )
    .expect("write");

    let document = Ledger_At(&directory, &AT_NOW)
        .Load()
        .expect("a document written before a field existed must still read");
    assert_eq!(document.items.len(), 1);
    assert!(
        document
            .items
            .first()
            .is_some_and(|item| item.abandoned.is_empty() && item.displaced.is_empty()),
        "a missing field must read as absent rather than refusing the file"
    );
}
