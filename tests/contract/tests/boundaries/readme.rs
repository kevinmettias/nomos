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
//!
//! # Why the prose around the table is compared, and where the reading stops
//!
//! `OD-ROADMAP-005` decision 6 supersedes `OD-AGENT-004` version 2's decline of generated
//! README tables and authorizes the narrower thing in its own words: a fact
//! `nomos-architecture.json` holds stops being restated in prose that nothing compares. That is
//! checking rather than generating, and the record says the reason is decisive -- version 2's
//! measurement stands, so the tables are checked *because* they are authored.
//!
//! So the assertions below read the prose and never a table row. [`Prose_Of`] is where that
//! boundary is drawn and its own doc carries the reason: `OD-PACKAGE-004` version 3 classifies
//! the table's third column, `Owns`, as free region -- nothing compares it, nothing may write
//! it -- having measured that the only source a comparison could use would be either the same
//! prose in a second file or a shorter fact that would not be that column. `OD-PACKAGE-017`
//! then measured the listing as two tables whose split has no declared source either, so the
//! row set stays compared as the union of both and which table a row belongs in is asserted
//! nowhere. Both are deliberate absences rather than gaps, and naming them is the point.

use crate::bands::{Declared_Architecture, Repository_Root};
use nomos_cap_architecture::{ArchitecturePayload, Exception, Membership};
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

/// A fenced block's delimiter, three backticks opening a line.
const FENCE: &str = "```";

/// The words the Layout paragraph uses for the declaration's excepted package pairs.
const SAME_ZONE_CLAIM: &str = "same-zone edges";

/// The README's own bytes.
///
/// Panics rather than returning an error, for the reason [`Readme_Zones`] gives: a comparison
/// against a file that could not be read finds no disagreement, so the prose would report
/// itself intact.
fn Readme_Text() -> String
{
    let path = Repository_Root().join("README.md");

    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

/// The README's prose: every line that is neither a row of one of its tables nor inside one of
/// its fenced blocks.
///
/// Both exclusions are load-bearing, and neither is tidiness.
///
/// **The table**, because its third column is free region. `OD-PACKAGE-004` version 3 measured
/// that nothing compares the `Owns` cells -- 33,102 characters over 72 rows -- and decided a
/// mechanism must never write them, naming a whole-file phrase constraint as *not* a region
/// check: it binds the column without owning it and reports a different defect when it fires.
/// `band_zero.rs` is the worked case over this very file, whose single `OD-CONTRACTS-001`
/// occurrence sits inside the `nomos-contracts` row's `Owns` cell. Every assertion built on
/// this reads the prose and not the file, so none of them reaches that column.
///
/// **The fenced blocks**, because a fence delimiter is three backticks and [`Backticked`]
/// pairs them. An opening fence leaves the count odd, so *inside* the block the reading
/// returns the gaps between backticked spans rather than the spans themselves, and the closing
/// fence restores the parity again. The damage is therefore confined to the block and silent
/// outside it -- which is worse than a shift that ran to the end of the file, because nothing
/// downstream looks wrong. It was measured the other way round first: a control that fenced a
/// command synopsis between two delimiters passed with this exclusion deleted, since two
/// fences are six backticks and the spans after them were never moved at all.
/// [`Test_A_Fenced_Blocks_Body_Should_Not_Be_Read_As_Prose`] is the discriminator that
/// replaced it.
fn Prose_Of(text: &str) -> String
{
    let mut prose = String::new();
    let mut fenced = false;
    for line in text.lines()
    {
        if line.starts_with(FENCE)
        {
            fenced = !fenced;
        }
        else if !fenced && !line.starts_with('|')
        {
            prose.push_str(line);
            prose.push('\n');
        }
    }

    return prose;
}

/// Every backticked span in `text`, in the order it carries them.
///
/// Pairs delimiters: the text between the first and the second, the third and the fourth, and
/// so on. [`Prose_Of`] drops fenced blocks before this runs, and its own doc says why that is
/// a correctness matter here rather than a preference.
fn Backticked(text: &str) -> Vec<&str>
{
    return text.split('`').skip(1).step_by(2).collect();
}

/// Whether `token` is spelled the way this workspace spells a crate.
///
/// A shape rather than a list, so a crate the prose names for the first time is compared
/// without anybody adding it here. The `nomos-spec-*` family glob in the repo-tooling
/// paragraph is excluded by that same shape, `*` not being a character a crate name is spelled
/// with -- which is the point of reading a shape rather than enumerating exceptions to one.
fn Is_A_Crate_Name(token: &str) -> bool
{
    return token.starts_with("nomos-") && token.chars().all(Is_A_Name_Character);
}

/// Whether one character may stand in a crate name here.
fn Is_A_Name_Character(character: char) -> bool
{
    return character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-';
}

/// Every crate the prose names, in the order it names them.
fn Crates_Named_In(prose: &str) -> Vec<String>
{
    return Backticked(prose)
        .into_iter()
        .filter(|token| return Is_A_Crate_Name(token))
        .map(str::to_owned)
        .collect();
}

/// Every crate the README's prose names is one the declaration places.
///
/// The rows are compared in both directions by
/// [`Test_The_Readme_Should_List_Every_Member_At_Its_Declared_Band`]; the crates named in the
/// sentences *around* them were compared by nothing. A crate renamed or dropped leaves the
/// prose naming something this workspace does not have while every row stays correct -- the
/// drift `OD-PROJECT-001` found in the table itself, one paragraph up from where it looked.
///
/// One direction, and that is a decision rather than an omission. The other -- every declared
/// member is named in the prose -- is not a claim the prose makes. It is the table's claim, and
/// [`Assert_Every_Member_Is_Listed`] already holds it. A paragraph is not an enumeration, so
/// requiring one to be complete would assert something nobody decided.
#[test]
fn Test_Every_Crate_The_Readmes_Prose_Names_Should_Be_One_The_Declaration_Places()
{
    let architecture = Declared_Architecture();
    let named = Crates_Named_In(&Prose_Of(&Readme_Text()));

    assert!(
        !named.is_empty(),
        "no crate name was read out of README.md's prose, so this compared nothing. The \
         paragraphs around the zone tables name crates in backticks, and this reading drops \
         table rows and fenced blocks before it looks."
    );

    let unplaced: Vec<&String> = named
        .iter()
        .filter(|name| return architecture.Component_Of(name).is_none())
        .collect();

    assert!(
        unplaced.is_empty(),
        "README.md's prose names {unplaced:?}, which nomos-architecture.json does not place. \
         The rows are compared in both directions and would not have caught it, because a \
         crate named in a sentence is in no row."
    );
}

/// The reading finds a crate this workspace does not have.
///
/// The negative control. Without it the assertion above is known only to pass over a README
/// whose every name is real, which from a green run is indistinguishable from a reading that
/// finds no names at all.
#[test]
fn Test_A_Prose_Naming_A_Crate_That_Does_Not_Exist_Should_Read_As_Naming_It()
{
    let naming = "The rule crate is `nomos-rules`, and `nomos-not-a-crate` is not one of them.";
    let family = "The `nomos-spec-*` family carries the identical distinction in its own prose.";

    assert_eq!(
        Crates_Named_In(naming),
        vec!["nomos-rules".to_owned(), "nomos-not-a-crate".to_owned()],
        "a sentence naming a crate this workspace does not have was not read as naming one, so \
         the assertion above could not tell a stale name from a current one"
    );
    assert!(
        Crates_Named_In(family).is_empty(),
        "the nomos-spec-* family glob was read as a crate name, and no declaration places a glob"
    );
}

/// A name inside a table row is not prose, so nothing above reaches the `Owns` column.
///
/// The boundary `OD-PACKAGE-004` version 3 draws, constructed rather than asserted: its third
/// question asks for the region to be made wrong on purpose and the comparison watched. Here
/// the wrong thing is put in the free column, and the reading must not see it.
#[test]
fn Test_A_Crate_Named_Only_In_A_Table_Row_Should_Not_Be_Read_As_Prose()
{
    let table = "Prose above the table.\n\
                 | Substrate | `nomos-model` | What it owns, which mentions `nomos-not-a-crate`. |\n\
                 Prose below the table.";

    assert!(
        Crates_Named_In(&Prose_Of(table)).is_empty(),
        "a crate named inside a table row was read as prose, so every assertion in this module \
         reaches the Owns column -- the region OD-PACKAGE-004 version 3 classifies as free and \
         says nothing watches."
    );
}

/// A fenced block's body is not prose, and dropping the block is what makes that true.
///
/// The falsifier for [`Prose_Of`]'s second exclusion, and the second one written: the first
/// fenced a command synopsis and passed with the exclusion deleted, because two fence
/// delimiters are six backticks and the spans *after* a balanced pair are where they always
/// were. What moves is the reading inside the block, which returns the gaps between spans
/// instead of the spans -- so an identifier that is not backticked there is read as though it
/// had been.
///
/// The body is built to exhibit exactly that and is not copied from `README.md`, whose own
/// fenced blocks are command synopses whose gaps carry spaces and so name no crate. A guard
/// whose only fixture is the artifact it guards is an example rather than a discriminator, and
/// this module's own siblings were written against that.
#[test]
fn Test_A_Fenced_Blocks_Body_Should_Not_Be_Read_As_Prose()
{
    let fenced = "Before the fence.\n\
                  ```\n\
                  `a`nomos-ghost`b`\n\
                  ```\n\
                  After it, `nomos-ledger` is named.";

    assert_eq!(
        Crates_Named_In(&Prose_Of(fenced)),
        vec!["nomos-ledger".to_owned()],
        "something inside a fenced block was read as a crate the prose names, or the name \
         after the block was not. Inside a block the reading returns the gaps between \
         backticked spans, so a bare identifier there reads as a backticked one."
    );
}

/// The excepted pairs whose two ends the declaration does not place in one component.
///
/// Over a declaration rather than over the file, so the disagreement can be constructed:
/// [`Test_An_Excepted_Pair_Whose_Ends_Sit_In_Two_Zones_Should_Be_Found`] builds one and watches
/// this find it.
///
/// A pair with an end the declaration never places is reported too. Such an end has no
/// component, so the pair cannot be a same-zone edge, and reading two absences as agreement
/// would leave the one case nothing else can check looking checked.
fn Excepted_Pairs_Crossing_A_Zone(architecture: &ArchitecturePayload) -> Vec<String>
{
    let mut crossing = Vec::new();
    for excepted in &architecture.exceptions
    {
        crossing.extend(Crossing_Description(architecture, excepted));
    }

    return crossing;
}

/// One excepted pair described, or nothing when the declaration places both ends together.
fn Crossing_Description(architecture: &ArchitecturePayload, excepted: &Exception) -> Option<String>
{
    const UNPLACED: &str = "no declared component";

    let from = architecture.Component_Of(&excepted.from);
    let to = architecture.Component_Of(&excepted.to);
    if from.is_some() && from == to
    {
        return None;
    }

    return Some(format!(
        "{} ({}) -> {} ({})",
        excepted.from,
        from.unwrap_or(UNPLACED),
        excepted.to,
        to.unwrap_or(UNPLACED)
    ));
}

/// The three parts that paragraph says the declaration holds, each with something in it.
///
/// A guard rather than an assertion of its own. An empty `exceptions` would leave the
/// comparison it guards passing having compared nothing, which is the vacuous pass every
/// quantified assertion in this module opens against.
fn Assert_The_Declaration_Holds_The_Three_Parts_The_Prose_Names(architecture: &ArchitecturePayload)
{
    assert!(
        !architecture.components.is_empty(),
        "nomos-architecture.json declares no zones, and README.md says crates are ordered into them"
    );
    assert!(
        !architecture.permissions.is_empty(),
        "nomos-architecture.json declares no zone may depend on any other, and README.md says it \
         names the zones each may depend on"
    );
    assert!(
        !architecture.exceptions.is_empty(),
        "nomos-architecture.json excepts no package pair, so the comparison this guards would \
         pass having read nothing"
    );
}

/// The excepted package pairs are the same-zone edges the README's prose calls them.
///
/// The Layout paragraph describes this repository's declaration as "a declared set of zones and
/// the zones each may depend on, plus a short, named list of the same-zone edges a real crate
/// needs". The first two are `components` and `permissions`; the third is `exceptions`, and
/// nothing compared the description against it. A cross-zone exception would make that sentence
/// false while every row, every permission and the zone count stayed correct.
///
/// The prose is read as well as the declaration, deliberately. An assertion that pinned the
/// declaration without opening the file it is about would be the shape `OD-PACKAGE-004`
/// version 3 names in `transport_registry.rs`: a claim about `README.md` quoted in a check's
/// own doc and failure message, with that file never opened -- cited by a check and read by
/// none. If the sentence goes, this comparison's subject goes with it, which is an edit worth
/// noticing rather than a check quietly outliving its claim.
#[test]
fn Test_The_Excepted_Pairs_Should_Be_The_Same_Zone_Edges_The_Prose_Calls_Them()
{
    let prose = Prose_Of(&Readme_Text());

    assert!(
        prose.contains(SAME_ZONE_CLAIM),
        "README.md's prose no longer calls the declaration's excepted pairs `{SAME_ZONE_CLAIM}`. \
         That sentence is what nomos-architecture.json's exceptions are compared against here, \
         so its removal leaves this without a subject rather than leaving it passing."
    );

    let architecture = Declared_Architecture();
    Assert_The_Declaration_Holds_The_Three_Parts_The_Prose_Names(&architecture);

    let crossing = Excepted_Pairs_Crossing_A_Zone(&architecture);

    assert!(
        crossing.is_empty(),
        "nomos-architecture.json excepts {crossing:#?}, whose ends it places in different \
         components. README.md calls that list the same-zone edges a real crate needs, so either \
         the declaration gained an edge the paragraph does not describe or the paragraph is stale."
    );
}

/// The reading finds an excepted pair whose ends sit in two components.
///
/// The negative control, over a declaration built for it rather than over this repository's
/// own, which satisfies the assertion today and would go on satisfying it if the reading
/// compared nothing.
#[test]
fn Test_An_Excepted_Pair_Whose_Ends_Sit_In_Two_Zones_Should_Be_Found()
{
    let together = Declaration_Excepting(
        Placed { package: "nomos-a", component: "Substrate" },
        Placed { package: "nomos-b", component: "Substrate" },
    );
    let apart = Declaration_Excepting(
        Placed { package: "nomos-a", component: "Substrate" },
        Placed { package: "nomos-b", component: "Host" },
    );

    assert!(
        Excepted_Pairs_Crossing_A_Zone(&together).is_empty(),
        "a pair whose ends share a component was reported as crossing one"
    );
    assert_eq!(
        Excepted_Pairs_Crossing_A_Zone(&apart).len(),
        1,
        "a pair whose ends sit in two components was not found, so the assertion above could \
         not tell a same-zone exception list from one that had stopped being same-zone"
    );
}

/// The reading finds an excepted pair naming a package no component places.
///
/// The second way the description can stop being true, and it is not the first wearing another
/// name. Both pairs below are unplaced and only one of them is caught by comparing the two
/// components: a pair with one unplaced end compares `Some` against `None` and disagrees
/// anyway, while a pair with *both* ends unplaced compares `None` against `None` and reads as
/// agreement. That second one is what [`Crossing_Description`]'s `is_some` guard is for, and
/// dropping the guard leaves this test red on it while the first pair goes on passing -- which
/// is why the fixture carries both rather than the one that reads as the obvious case.
#[test]
fn Test_An_Excepted_Pair_Naming_A_Package_No_Component_Places_Should_Be_Found()
{
    let unplaced = ArchitecturePayload {
        components: vec!["Substrate".to_owned()],
        membership: vec![Membership { package: "nomos-a".to_owned(), component: "Substrate".to_owned() }],
        exceptions: vec![
            Exception { from: "nomos-a".to_owned(), to: "nomos-gone".to_owned() },
            Exception { from: "nomos-also-gone".to_owned(), to: "nomos-gone".to_owned() },
        ],
        ..ArchitecturePayload::default()
    };

    assert_eq!(
        Excepted_Pairs_Crossing_A_Zone(&unplaced).len(),
        2,
        "an excepted pair naming a package the declaration does not place was read as a \
         same-zone edge, which is the one case nothing else in this suite would report"
    );
}

/// One package and the component a constructed declaration places it in.
///
/// A named pair rather than two strings, the reason [`nomos_cap_architecture::Depending`] and
/// [`nomos_cap_architecture::Depended`] are named types: two `&str` in a row is an argument
/// order a caller can transpose silently.
///
/// Copied rather than borrowed at the call, because two shared references are a value and the
/// lint layer says so: taking it by reference draws `clippy::needless_pass_by_value` on every
/// caller instead, which `nomos gate run` reports as a finding of its own.
#[derive(Clone, Copy)]
struct Placed<'a>
{
    /// The package's name.
    package: &'a str,
    /// The component the declaration places it in.
    component: &'a str,
}

/// A declaration placing two packages and excepting the first to name the second.
fn Declaration_Excepting(from: Placed<'_>, to: Placed<'_>) -> ArchitecturePayload
{
    return ArchitecturePayload {
        components: vec![from.component.to_owned(), to.component.to_owned()],
        membership: vec![
            Membership { package: from.package.to_owned(), component: from.component.to_owned() },
            Membership { package: to.package.to_owned(), component: to.component.to_owned() },
        ],
        exceptions: vec![Exception { from: from.package.to_owned(), to: to.package.to_owned() }],
        ..ArchitecturePayload::default()
    };
}
