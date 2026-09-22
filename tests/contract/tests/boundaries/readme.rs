//! The README's zone table and its two ledger vocabularies, parsed back out of the file a
//! reader edits.
//!
//! # Why a vocabulary is here and not only a table
//!
//! `OD-AGENT-004` version 1 measured this file as one of the two that stayed correct while
//! fifteen module-doc restatements went stale, and named the reason: this suite asserts the
//! README's *tables* against the real workspace, both directions. Version 2 extended the rule
//! to the text a command prints and stated it as a condition — enumerate a compiled vocabulary
//! only where a test compares that enumeration against its authority, otherwise route.
//!
//! Closing that condition across `nomos-cli` found five enumerations in this README that no
//! table check could ever have seen, because a pipe-joined vocabulary inside a fenced command
//! block is not a table. One of them was already stale. So the README is not correct *because
//! it is the README*; it is correct where it is checked, which is version 1's finding holding
//! at a finer grain rather than an exception to it.
//!
//! Two of the five are compared here. The other three route instead, and the reason is
//! reachability rather than preference: `nomos work list`'s state vocabulary is produced by
//! functions private to a binary crate that exports no library, so no test outside it can read
//! the set, and restating that set here would be the second authority the record refuses.

use crate::bands::{Declared_Architecture, Repository_Root};
use nomos_cap_architecture::ArchitecturePayload;
use nomos_ledger::{ItemKind, ItemOrigin};

/// The component table the README shows a reader, parsed back out of the file they edit.
///
/// A row is `| <component> | `<crate>` | … |`. Anything else in the README is prose as far as
/// this is concerned, including the `D-128` table in `OD-PROJECT-001` and any table whose
/// first cell does not name one of the components this repository declares.
fn Readme_Zones(architecture: &ArchitecturePayload) -> Vec<(String, String)>
{
    let path = Repository_Root().join("README.md");
    let text = std::fs::read_to_string(&path)
        // The README is the one document this table is parsed back out of. Unreadable, it
        // yields no rows, and a comparison against no rows finds no disagreement — the
        // zone table would be checked against nothing and report itself intact.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    let mut rows = Vec::new();
    for line in text.lines()
    {
        let row = Zone_Row(architecture, line);

        rows.extend(row);
    }

    return rows;
}

/// One row, or nothing if the line is not one.
///
/// A table row opens with the delimiter, so the first cell is the empty string before it. A
/// line that merely contains a pipe does not. The first cell must name one of
/// one of the declared components exactly, which is what tells a real row apart from a
/// heading (`| Zone | Crate | Owns |`) or a Markdown separator (`|---|---|---|`) without a
/// second, separately-maintained list of what counts as a row.
fn Zone_Row(architecture: &ArchitecturePayload, line: &str) -> Option<(String, String)>
{
    let mut cells = line.split('|').map(str::trim);
    if cells.next() != Some("")
    {
        return None;
    }

    let (Some(zone_text), Some(name)) = (cells.next(), cells.next())
    else
    {
        return None;
    };
    let component = architecture.components.iter().find(|component| return *component == zone_text)?;
    let name = name.strip_prefix('`').and_then(|rest| rest.strip_suffix('`'))?;

    return Some((name.to_owned(), component.clone()));
}

/// The README describes this workspace, and nothing checked that it still did.
///
/// `D-128` requires the maintained overview documents to be freshness-validated, and
/// `OD-PROJECT-001` records why this README is not one of them: no content kind in the
/// projection system selects a crate's zone, so no profile can render this file. What
/// that record gives up is freshness for the prose. What it does not give up is the
/// table, because the table restates this repository's own `nomos-architecture.json` — the one
/// declaration, read here through [`crate::bands`] rather than kept as a second copy — and a
/// restatement can be compared.
///
/// It had already drifted when this check was first written: twenty-two members, eleven of
/// them listed, and the missing eleven included `nomos-spec-project`, the crate that
/// renders the projections `OD-PROJECT-001` is about.
///
/// Compared against the declaration rather than against the member list, because
/// [`crate::graph::Test_Every_Member_Should_Declare_A_Band`] already ties those two
/// together. Two checks reaching the same conclusion by different routes is how they come
/// to disagree.
#[test]
fn Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band()
{
    let architecture = Declared_Architecture();
    let listed = Readme_Zones(&architecture);

    assert!(
        !listed.is_empty(),
        "no zone row was found in README.md. Every assertion below iterates over these \
         rows, so an empty set passes having read nothing — and the layout tables are \
         written as `| <zone> | `<crate>` | … |`"
    );

    Assert_No_Crate_Is_Listed_Twice(&listed);
    Assert_Every_Member_Is_Listed(&architecture, &listed);
    Assert_Every_Listing_Is_A_Member(&architecture, &listed);
}

/// Two rows for one crate can disagree with each other, so there is only ever one.
fn Assert_No_Crate_Is_Listed_Twice(listed: &[(String, String)])
{
    use std::collections::BTreeSet;

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for (name, _) in listed
    {
        assert!(
            seen.insert(name.as_str()),
            "{name} appears in more than one README zone row, so the two can disagree"
        );
    }
}

/// Every crate the declaration places appears in the table a reader is shown.
fn Assert_Every_Member_Is_Listed(architecture: &ArchitecturePayload, listed: &[(String, String)])
{
    let missing: Vec<&str> = architecture
        .membership
        .iter()
        .filter(|placed| return !listed.iter().any(|(name, _)| return *name == placed.package))
        .map(|placed| return placed.package.as_str())
        .collect();

    assert!(
        missing.is_empty(),
        "these crates are in the workspace and not in README.md's layout tables: \
         {missing:?}.\n\
         OD-PROJECT-001 keeps this file hand-authored on the condition that the part of \
         it a machine can check is checked."
    );
}

/// The other direction, in both halves: a row naming no crate, and a row naming the wrong
/// component for one that exists.
fn Assert_Every_Listing_Is_A_Member(architecture: &ArchitecturePayload, listed: &[(String, String)])
{
    let invented: Vec<&(String, String)> = listed
        .iter()
        .filter(|(name, _)| return architecture.Component_Of(name).is_none())
        .collect();
    let disagreeing: Vec<String> = listed
        .iter()
        .filter_map(|(name, component)| {
            let declared = architecture.Component_Of(name)?;

            return (declared != component).then(|| {
                return format!("{name}: README says {component}, nomos-architecture.json says {declared}");
            });
        })
        .collect();

    assert!(
        invented.is_empty(),
        "README.md lists crates this workspace does not have: {invented:?}"
    );
    assert!(disagreeing.is_empty(), "{disagreeing:#?}");
}

/// The alternatives a `--flag a|b|c` run in the README's command synopsis lists.
///
/// The first occurrence wins, which is what makes this answer about the synopsis: every flag
/// below is spelled bare there and backticked in the prose around it, and the bare one is the
/// line a reader copies.
fn Alternatives_After(flag: &str) -> Vec<String>
{
    let path = Repository_Root().join("README.md");
    // Unreadable, this yields no alternatives, and a comparison against none finds no
    // disagreement -- the guard below refuses an empty parse for exactly that reason, so the
    // failure reports the file rather than reporting the vocabulary as intact.
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let Some((_, rest)) = text.split_once(&format!("{flag} "))
    else
    {
        return Vec::new();
    };
    let listed = rest
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|character| return matches!(character, '[' | ']' | '<' | '>'));

    return listed.split('|').map(str::to_owned).collect();
}

/// A list of words in ascending order, so that two of them can be compared as sets.
fn Sorted_Words(words: impl Iterator<Item = String>) -> Vec<String>
{
    let mut sorted: Vec<String> = words.collect();
    sorted.sort();
    sorted.dedup();

    return sorted;
}

/// The spelling `work add --kind` takes for a kind, as an exhaustive match, so that a variant
/// added to [`ItemKind`] stops this file compiling rather than passing.
fn Spelled_Kind(kind: ItemKind) -> &'static str
{
    return match kind
    {
        ItemKind::Capability => "capability",
        ItemKind::Decision => "decision",
        ItemKind::Validation => "validation",
        ItemKind::Correction => "correction",
        ItemKind::Cleanup => "cleanup",
    };
}

/// The spelling `work add --origin` takes for an origin, as an exhaustive match.
fn Spelled_Origin(origin: ItemOrigin) -> &'static str
{
    return match origin
    {
        ItemOrigin::Required => "required",
        ItemOrigin::Proposed => "proposed",
    };
}

/// The kinds the README's synopsis lists are the kinds an item can declare.
///
/// `nomos-cli`'s own help text already carries this comparison against the same enum. This one
/// is not that one repeated: it is about a different artifact, edited by different people for
/// different reasons, which is precisely why a reader trusting the README and a reader running
/// `--help` were able to be told different things.
#[test]
fn Test_The_Readmes_Listed_Kinds_Should_Be_Every_Kind_An_Item_Can_Declare()
{
    let listed = Alternatives_After("--kind");

    assert!(
        !listed.is_empty(),
        "no --kind alternative was parsed out of README.md, so this compared nothing"
    );
    assert_eq!(
        Sorted_Words(listed.into_iter()),
        Sorted_Words(
            [
                ItemKind::Capability,
                ItemKind::Decision,
                ItemKind::Validation,
                ItemKind::Correction,
                ItemKind::Cleanup,
            ]
            .into_iter()
            .map(|kind| return Spelled_Kind(kind).to_owned())
        ),
        "README.md and ItemKind disagree about what --kind takes"
    );
}

/// The origins the README's synopsis lists are the origins an item can declare.
/// [`Test_The_Readmes_Listed_Kinds_Should_Be_Every_Kind_An_Item_Can_Declare`]'s reasoning, over
/// the other of `OD-LEDGER-024`'s two closed sets.
#[test]
fn Test_The_Readmes_Listed_Origins_Should_Be_Every_Origin_An_Item_Can_Declare()
{
    let listed = Alternatives_After("--origin");

    assert!(
        !listed.is_empty(),
        "no --origin alternative was parsed out of README.md, so this compared nothing"
    );
    assert_eq!(
        Sorted_Words(listed.into_iter()),
        Sorted_Words(
            [ItemOrigin::Required, ItemOrigin::Proposed]
                .into_iter()
                .map(|origin| return Spelled_Origin(origin).to_owned())
        ),
        "README.md and ItemOrigin disagree about what --origin takes"
    );
}

/// The count the zones paragraph states, if it states one this function can read.
///
/// Over text rather than over the file, so that a disagreement can be constructed. The
/// assertion below is about a number that is correct today, and a guard believed because it
/// passes over a tree that already satisfies it has proved nothing -- which is the failure
/// `OD-COMPLETENESS-001` names and the one this module's own siblings were written against.
///
/// Number words rather than digits, because that is how the sentence is written and prose is
/// the artifact being checked. A count this function cannot read is reported by the caller
/// as an absence rather than defaulted, since a sentence that stopped saying how many zones
/// there are is a different edit from one that says the wrong number.
fn Stated_Zone_Count(text: &str) -> Option<usize>
{
    const OPENING: &str = "ordered into ";
    const CLOSING: &str = " named zones";

    let from = text.find(OPENING)?.checked_add(OPENING.len())?;
    let rest = text.get(from..)?;
    let to = rest.find(CLOSING)?;
    let word = rest.get(..to)?;

    return NUMBER_WORDS.iter().find_map(|(spelled, value)| return (*spelled == word).then_some(*value));
}

/// The number words this repository's own prose could plausibly reach for.
///
/// Bounded deliberately: a zone set large enough to need a word past twenty is a different
/// architecture, and a table that guessed at one would be answering a question nobody has
/// asked yet.
const NUMBER_WORDS: [(&str, usize); 16] = [
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
    ("thirteen", 13),
    ("fourteen", 14),
    ("fifteen", 15),
    ("sixteen", 16),
    ("seventeen", 17),
    ("eighteen", 18),
    ("nineteen", 19),
    ("twenty", 20),
];

/// The zone count the README states in prose is the number of components declared.
///
/// The rows below this sentence are asserted in both directions by
/// [`Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band`], and the sentence above
/// them was asserted by nothing: a thirteenth component would leave the word stale while every
/// row stayed correct, one line from the table that would have caught it anywhere else.
///
/// A comparison rather than a removal, which is what distinguishes this from the three
/// vocabularies [`Test_The_Readme_Should_Not_Relist_A_Vocabulary_It_Routes_To`] forbids. Those
/// were removed because their authority is private to a binary crate and no test outside it
/// can read the set. This number's authority is `nomos-architecture.json`, which
/// [`Declared_Architecture`] already reads for the rows, so `OD-AGENT-004`'s rule sends it the
/// other way: a fact another artifact checks is routed to rather than restated, and a
/// restatement that can be compared is checked rather than deleted.
#[test]
fn Test_The_Readmes_Stated_Zone_Count_Should_Be_The_Number_Of_Declared_Components()
{
    let path = Repository_Root().join("README.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    let stated = Stated_Zone_Count(&text).unwrap_or_else(|| {
        panic!(
            "README.md no longer says how many zones crates are ordered into, or says it in a \
             spelling this test cannot read. Either is an edit worth noticing: the sentence is \
             the one thing above the zone table that names its size."
        )
    });

    let declared = Declared_Architecture().components.len();

    assert_eq!(
        stated, declared,
        "README.md says crates are ordered into {stated} named zones and \
         nomos-architecture.json declares {declared} components. The rows are checked in both \
         directions and would not have caught this, because a component gained or lost changes \
         the count without making any single row wrong."
    );
}

/// The reading rejects a sentence that disagrees with the declaration.
///
/// The negative control. Without it the assertion above is only known to pass over a README
/// that already agrees, which is indistinguishable from an assertion that cannot fail.
#[test]
fn Test_A_Stated_Count_That_Disagrees_Should_Be_Read_As_A_Different_Number()
{
    let agreeing = "Crates are ordered into twelve named zones -- the rest is prose.";
    let disagreeing = "Crates are ordered into thirteen named zones -- the rest is prose.";
    let silent = "Crates are ordered into zones, and this sentence names no number.";

    assert_eq!(
        Stated_Zone_Count(agreeing),
        Some(12),
        "the sentence README.md actually carries was not read as a number"
    );
    assert_eq!(
        Stated_Zone_Count(disagreeing),
        Some(13),
        "a sentence stating a different count was not read as a different number, so the \
         assertion above could not tell a stale prose count from a current one"
    );
    assert_eq!(
        Stated_Zone_Count(silent),
        None,
        "a sentence naming no count was read as one anyway"
    );
}
/// The vocabularies the README routes to instead of listing stay routed.
///
/// Three of the five enumerations this file used to carry were removed rather than compared,
/// because their authority is private to a binary crate and restating it here would be the
/// second authority `OD-AGENT-004` refuses. A removal is not self-sustaining the way a
/// comparison is -- nothing stops a later author pasting the list back, and it would read as an
/// improvement. This is what refuses it.
///
/// Asserted on the words rather than on the flags, because the flags themselves stay: the
/// synopsis still shows `--state <state>`, and it is the alternatives that must not come back.
#[test]
fn Test_The_Readme_Should_Not_Relist_A_Vocabulary_It_Routes_To()
{
    let path = Repository_Root().join("README.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    for relisted in ["snagged", "stranded", "feature-request", "design-spec", "feature-result"]
    {
        assert!(
            !text.contains(relisted),
            "README.md names `{relisted}` again. That vocabulary is routed to rather than \
             listed, because nothing here can compare it against its authority -- see this \
             module's own doc and OD-AGENT-004."
        );
    }
}
