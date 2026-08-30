//! What this module promises, exercised.

use super::*;
use super::spelling::{Normalize_Path, Subject_Of};
use nomos_model::Intersection;

/// The property the normalization exists for. Two spellings of one file must be one
/// subject, or the ledger hands out overlapping territory believing it is disjoint.
/// Every spelling this test asserts must reduce to the same subject as the canonical path.
fn Equivalent_Spellings_Of_One_File() -> [&'static str; 5]
{
    return [
        "./crates/kernel/nomos-model/src/digest.rs",
        "crates\\kernel\\nomos-model\\src\\digest.rs",
        "crates//kernel/nomos-model/src/digest.rs",
        "  crates/kernel/nomos-model/src/digest.rs  ",
        "crates/kernel/nomos-model/src/Digest.rs",
    ];
}

#[test]
fn Test_Spellings_Of_One_Path_Should_Be_One_Subject()
{
    let canonical = Subject_Of("crates/kernel/nomos-model/src/digest.rs");

    for spelling in Equivalent_Spellings_Of_One_File()
    {
        assert_eq!(
            Subject_Of(spelling),
            canonical,
            "`{spelling}` must denote the same subject"
        );
    }
}

/// The negative control. If normalization collapsed everything, the test above would
/// pass while the ledger refused every concurrent claim in the repository.
#[test]
fn Test_Different_Paths_Should_Be_Different_Subjects()
{
    assert_ne!(Subject_Of("src/a.rs"), Subject_Of("src/b.rs"));
    assert_ne!(Subject_Of("src/a.rs"), Subject_Of("tests/a.rs"));
    assert_ne!(Subject_Of("a/b.rs"), Subject_Of("a-b.rs"));
}

#[test]
fn Test_Of_Files_Should_Be_Disjoint_Across_Distinct_Territories()
{
    let left = Territory::Of_Files(["crates/a/src/lib.rs"]);
    let right = Territory::Of_Files(["crates/b/src/lib.rs"]);

    assert_eq!(left.Intersect(&right), Intersection::Disjoint);
}

/// The same file written two ways in two items must still collide. This is the
/// end-to-end version of the normalization test, through the type the ledger uses.
#[test]
fn Test_One_File_Written_Two_Ways_Should_Overlap()
{
    let left = Territory::Of_Files(["crates/a/src/lib.rs"]);
    let right = Territory::Of_Files(["./crates\\a\\src\\LIB.rs"]);

    assert!(!left.Intersect(&right).Permits_Concurrency());
}

/// The case this comparison exists for. A directory and a file inside it are the
/// same work; comparing their identities says they are unrelated, because as
/// identities they are.
#[test]
fn Test_A_Directory_Should_Contain_Its_Files()
{
    let crate_root = Territory::Of_Files(["crates/spec/nomos-spec-model"]);
    let one_file = Territory::Of_Files(["crates/spec/nomos-spec-model/src/normalizer.rs"]);

    assert!(
        !crate_root.Intersect(&one_file).Permits_Concurrency(),
        "a directory claim must exclude a file inside it"
    );
    assert!(
        !one_file.Intersect(&crate_root).Permits_Concurrency(),
        "containment must be caught from either side"
    );

    // And the identity comparison genuinely cannot see it, which is why the
    // containment check is not redundant.
    assert_eq!(
        crate_root
            .As_Subject_Set()
            .Intersect(&one_file.As_Subject_Set()),
        Intersection::Disjoint
    );
}

/// The negative control for containment. Prefix matching on raw strings would make
/// `crates/a` contain `crates/abc`, and the whole repository would serialize.
#[test]
fn Test_A_Sibling_With_A_Shared_Prefix_Should_Not_Be_Contained()
{
    let short = Territory::Of_Files(["crates/nomos-spec"]);
    let similar = Territory::Of_Files(["crates/nomos-spec-model/src/lib.rs"]);

    assert_eq!(short.Intersect(&similar), Intersection::Disjoint);
}

/// The repository root contains everything, so an item reserving it excludes all
/// others rather than colliding with none of them.
#[test]
fn Test_The_Repository_Root_Should_Contain_Everything()
{
    let everything = Territory::Of_Files(["."]);
    let something = Territory::Of_Files(["crates/a/src/lib.rs"]);

    assert!(!everything.Intersect(&something).Permits_Concurrency());
}

#[test]
fn Test_A_Pattern_Should_Make_Comparison_Unknown()
{
    let vague = Territory::Empty().With_Pattern("crates/spec/**");
    let concrete = Territory::Of_Files(["crates/host/nomos-cli/src/work.rs"]);

    let answer = vague.Intersect(&concrete);

    assert!(matches!(answer, Intersection::Unknown(_)));
    assert!(!answer.Permits_Concurrency());
}

/// Territory stated at two resolutions cannot be compared by matching paths, and
/// answering "disjoint" would be a confident wrong answer.
#[test]
fn Test_Mismatched_Resolutions_Should_Be_Unknown()
{
    let by_file = Territory::Of_Files(["src/a.rs"]);
    let mut by_symbol = Territory::Of_Files(["src/a.rs"]);
    by_symbol.resolution = SetResolution::Symbol;

    let answer = by_file.Intersect(&by_symbol);

    assert!(matches!(answer, Intersection::Unknown(_)));
    assert!(!answer.Permits_Concurrency());
}

/// Two entries denoting one subject is a mistake by whoever wrote the item, and the
/// ledger should say so rather than quietly treating the pair as one.
#[test]
fn Test_Ambiguous_Paths_Should_Be_Reported()
{
    let confused = Territory::Of_Files(["src/Main.rs", "src/main.rs", "src/other.rs"]);

    let ambiguous = confused.Ambiguous_Paths();

    assert_eq!(ambiguous.len(), 1);
    assert!(
        Territory::Of_Files(["src/a.rs", "src/b.rs"])
            .Ambiguous_Paths()
            .is_empty(),
        "distinct paths are not ambiguous"
    );
}

/// A territory serialized for review must show paths, not digests. This is the
/// property that justifies the ledger being a file.
#[test]
fn Test_The_Serialized_Form_Should_Show_Paths()
{
    let territory = Territory::Of_Files(["crates/kernel/nomos-model/src/digest.rs"]);

    let rendered = serde_json::to_string(&territory).unwrap();

    assert!(
        rendered.contains("crates/kernel/nomos-model/src/digest.rs"),
        "a reviewer must be able to read what an item reserves: {rendered}"
    );
}

/// The defect `OD-LEDGER-016` closes, with both spellings written out.
///
/// `docs/records/OD-LEDGER-006` is the identifier an item reserves and
/// `docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md`
/// is the file that identifier was allocated for. They are one record, so they are one
/// subject; before the folding they were siblings and compared `Disjoint`.
#[test]
fn Test_A_Record_Identifier_And_Its_File_Should_Be_One_Subject()
{
    let identifier = "docs/records/OD-LEDGER-006";
    let file = "docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-\
                survive-it.md";

    assert_eq!(
        Subject_Of(file),
        Subject_Of(identifier),
        "`{file}` is the file `{identifier}` names, so the two must be one subject"
    );
}

/// The same thing through the type the ledger actually asks, in both directions.
///
/// An item amending an existing record and an item that reserved that record's
/// identifier must exclude each other, and which of them wrote which spelling must not
/// decide it.
#[test]
fn Test_Reserving_A_Record_Should_Exclude_The_Writer_Of_Its_File()
{
    let by_identifier = Territory::Of_Files(["docs/records/OD-LEDGER-006"]);
    let by_file = Territory::Of_Files([
        "docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md",
    ]);

    assert!(
        !by_identifier.Intersect(&by_file).Permits_Concurrency(),
        "reserving a record's identifier must exclude the item that writes its file"
    );
    assert!(
        !by_file.Intersect(&by_identifier).Permits_Concurrency(),
        "and from the other side, because exclusion is symmetric"
    );

    // Case and separator folding has to survive the reduction, or the hole closes for
    // one spelling of the filename and stays open for the next.
    let shouted = Territory::Of_Files([
        r"docs\Records\OD-LEDGER-006-A-Reason-Attached-To-A-Transition-Does-Not-Survive-It.MD",
    ]);
    assert!(!by_identifier.Intersect(&shouted).Permits_Concurrency());
}

/// Two records are still two records. The reduction must fold a filename onto *its*
/// identifier and not onto a neighbouring one.
#[test]
fn Test_Two_Records_Should_Still_Be_Two_Subjects()
{
    let one = Territory::Of_Files(["docs/records/OD-LEDGER-006"]);
    let another = Territory::Of_Files([
        "docs/records/OD-LEDGER-009-a-documents-validity-must-not-depend-on-when-it-is-read.md",
    ]);

    assert_eq!(one.Intersect(&another), Intersection::Disjoint);
}

/// The nearest miss, and the reason the ordinal is compared as a whole component.
///
/// `od-ledger-001` is a prefix of the *text* `od-ledger-0011-...`, and a rule written
/// with `starts_with` would fold the hundred-and-first record onto the first — the same
/// shape as `crates/a` swallowing `crates/abc`, which is why containment appends a
/// separator rather than matching bare text.
#[test]
fn Test_A_Longer_Ordinal_Should_Be_A_Different_Record()
{
    let first = Territory::Of_Files(["docs/records/OD-LEDGER-001"]);
    let hundred_and_first =
        Territory::Of_Files(["docs/records/OD-LEDGER-0011-a-much-later-record.md"]);

    assert_eq!(first.Intersect(&hundred_and_first), Intersection::Disjoint);
    assert_ne!(
        Subject_Of("docs/records/OD-LEDGER-0011-a-much-later-record.md"),
        Subject_Of("docs/records/OD-LEDGER-001")
    );
}

/// The blast-radius control. The reduction is scoped to one directory, and everywhere
/// else a name that merely looks like an identifier is a coincidence.
///
/// Both misses are constructed rather than found: a fixture pair outside the record
/// directory, and a directory whose *name* extends the record directory's. The second
/// is the one a prefix test gets wrong — `docs/records-archive` starts with
/// `docs/records` as text and is not inside it.
#[test]
fn Test_The_Reduction_Should_Not_Escape_The_Record_Directory()
{
    let case = Territory::Of_Files(["tests/fixtures/case-001"]);
    let expectation = Territory::Of_Files(["tests/fixtures/case-001-expected.md"]);

    assert_eq!(
        case.Intersect(&expectation),
        Intersection::Disjoint,
        "outside `docs/records` an ordinal in a filename means nothing"
    );

    let record = Territory::Of_Files(["docs/records/OD-LEDGER-001"]);
    let neighbour =
        Territory::Of_Files(["docs/records-archive/OD-LEDGER-001-an-old-copy.md"]);

    assert_eq!(
        record.Intersect(&neighbour),
        Intersection::Disjoint,
        "a directory whose name extends `docs/records` is not `docs/records`"
    );
}

/// A leading digit group is a date, not an allocation, and folding it would invent a
/// record called `docs/records/2026` that two unrelated notes would then share.
#[test]
fn Test_A_Leading_Digit_Group_Should_Not_Be_An_Ordinal()
{
    assert_eq!(
        Normalize_Path("docs/records/2026-08-09-notes.md"),
        "docs/records/2026-08-09-notes.md"
    );
    assert_eq!(
        Territory::Of_Files(["docs/records/2026-08-09-notes.md"])
            .Intersect(&Territory::Of_Files(["docs/records/2026-08-10-notes.md"])),
        Intersection::Disjoint
    );
}

/// What is left alone. A file under `docs/records` that carries no ordinal is a file,
/// and a registration file elsewhere keeps its extension — that directory is named by
/// identifier too, and `records/OD-LEDGER-016.record` is a real path that resolves.
/// Paths that carry no record filename for `Normalize_Path` to fold.
fn Paths_With_No_Record_Identifier() -> [&'static str; 4]
{
    return [
        "docs/records/readme.md",
        "docs/records/OD-LEDGER-006",
        "crates/spec/nomos-spec-store/records/OD-LEDGER-016.record",
        "docs/records",
    ];
}

#[test]
fn Test_A_Path_Without_A_Record_Identifier_Should_Be_Untouched()
{
    for path in Paths_With_No_Record_Identifier()
    {
        assert_eq!(
            Normalize_Path(path),
            path.to_lowercase(),
            "`{path}` carries no record filename to reduce"
        );
    }
}

#[test]
fn Test_An_Empty_Territory_Should_Be_Empty()
{
    assert!(Territory::Empty().Is_Empty());
    assert!(!Territory::Of_Files(["a.rs"]).Is_Empty());
    assert!(!Territory::Empty().With_Pattern("a/**").Is_Empty());
}
