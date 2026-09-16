//! Rebuildability: which profiles a checkout with no corpus can render.
//!
//! The three corpora live outside this repository and CI has none of them, so the store a
//! runner assembles holds this repository's governing records and nothing else. Which
//! profiles that store can answer for decides which ones may be *required* by the gate, and
//! it was folklore until it was written down here.
//!
//! The declaration lives in this test file rather than in the crate's public surface, and
//! that is a choice rather than a shortcut. Nothing in the crate or the host reads it — the
//! gate reads --require and the renderer reads sections — so a public method would be API
//! with no consumer, which is the shape this repository argues against wherever it decides
//! that only what something uses should exist. What the declaration is *for* is the
//! comparison below, so it lives with the comparison. `OD-PROJECT-002` states the rule for a
//! reader.

use crate::store::{Profile_Named, Shipped};
use nomos_spec_project::{Build, Content, Format, Profile, Select_Projection};
use nomos_spec_store::SpecificationStore;
use std::collections::BTreeSet;

/// The content kinds a store holding only the governing records can answer for.
///
/// Wrong the first time it was written: `Rows` was missing. Governing records carry markdown
/// tables and `Write_Record` writes their rows, and the census below is what said so.
///
/// `Seed_Governing_Records` writes nodes, source documents, source blocks, headings, table
/// rows, front matter and relations, and the heading and block dispositions it records are
/// lineage. It writes no suites, no normative statements and no omissions.
const SEEDED_BY_RECORDS: &[Content] = &[
    Content::Documents,
    Content::Headings,
    Content::Blocks,
    Content::Rows,
    Content::Nodes,
    Content::Relations,
    Content::Lineage,
    // A records-only store answers for this the moment it answers for `Relations`: a
    // neighbourhood is those rows resolved one hop to the nodes at their far end, and
    // `OD-PROJECT-005` fixed that hop at one. Added when `implementation-context-pack` became
    // subject-scoped -- before then nothing selected it, and this list's own message says why
    // omitting a kind the store answers for is not harmless.
    Content::Neighbourhood,
    // Answered for the same reason `Neighbourhood` is: a family view is the relation rows
    // rolled up by the identifier each end carries, and a records-only store holds both.
    // `OD-PROJECT-006` decided this view is deliberately not required, which is a different
    // question from whether a seeded store can answer for it.
    Content::Families,
];

/// Whether every section of a profile reaches a kind a seeded store answers for.
///
/// **Necessary for rebuildability and not sufficient**, and the name is chosen to say so. An
/// earlier version of this was called `Rebuildable_From_Records` and returned true for
/// `feature-design`, which refuses on a seeded store: its `Concepts` section reaches `Nodes`,
/// a seeded kind, and filters it on node kind `concept`, which only a corpus has. A governing
/// record is kind `decision` or `architecture` and nothing else.
///
/// So the missing half is data-dependent and cannot be read off a profile at all. Whether a
/// filter selects anything is a question about a store, which is why requirability is
/// established by rendering and this is only the cheap screen in front of it.
fn Is_Reaching_Only_Seeded_Content(profile: &Profile) -> bool
{
    return profile
        .sections
        .iter()
        .all(|section| return SEEDED_BY_RECORDS.contains(&section.content));
}

/// A one-section profile used to ask a store whether it can answer for one content kind.
///
/// `may_be_empty` is set so the question comes back as a count rather than as a refusal — the
/// refusal is what production wants and is the wrong instrument for a census.
fn Probe(content: Content) -> Profile
{
    return Profile {
        id: format!("probe-{}", content.Label()),
        title: format!("Probe {}", content.Label()),
        format: Format::Markdown,
        output: format!("probe/{}.md", content.Label()),
        sections: vec![nomos_spec_project::ProfileSection {
            title: format!("{content:?}"),
            content,
            filter: nomos_spec_project::Filter::default(),
            may_be_empty: true,
        }],
    };
}

fn Seeded() -> SpecificationStore
{
    let mut store =
        SpecificationStore::In_Memory().expect("an in-memory store is constructed for this test");
    nomos_spec_store::Seed_Governing_Records(&mut store)
        .expect("the seed wrote the governing records into the store this test opened");

    return store;
}

/// A set of profile identifiers, so the assertions below compare sets rather than counts.
fn Named(ids: &[&str]) -> BTreeSet<String>
{
    return ids.iter().map(|id| return (*id).to_owned()).collect();
}

/// How many items a `may_be_empty` probe for one content kind came back with.
fn Answered_Items(store: &SpecificationStore, content: Content) -> usize
{
    let probe = Probe(content);
    let projection = Select_Projection(store, &probe).expect("a may_be_empty probe never refuses");

    return projection
        .sections
        .first()
        .map_or(0, |section| return section.items.len());
}

/// The declaration against a store that was really seeded, in both directions.
///
/// One direction alone is the failure a completeness check runs into every time it is written
/// once. Asserting only that every declared kind answers would let the list shrink to nothing
/// and still pass; asserting only that every undeclared kind is silent would let it grow to
/// all ten. So the two sets are compared for equality.
#[test]
fn Test_A_Seeded_Store_Should_Answer_For_Exactly_The_Declared_Content_Kinds()
{
    let store = Seeded();
    let mut answered: BTreeSet<&str> = BTreeSet::new();
    for content in Content::All()
    {
        if Answered_Items(&store, *content) > 0
        {
            answered.insert(content.Label());
        }
    }
    let declared: BTreeSet<&str> = SEEDED_BY_RECORDS
        .iter()
        .map(|content| return content.Label())
        .collect();

    assert_eq!(
        answered, declared,
        "SEEDED_BY_RECORDS disagrees with what Seed_Governing_Records writes. A kind the \
         store answered for but the list omits would make a rebuildable profile look \
         corpus-backed; a kind the list claims but the store cannot answer for would make a \
         profile requirable that refuses on every runner."
    );
}

/// What one pass over the shipped profiles found each of them doing over a seeded store.
#[derive(Default)]
struct Outcomes
{
    renders: BTreeSet<String>,
    screened_but_refuses: BTreeSet<String>,
    skipped: BTreeSet<String>,
}

/// One pass over the shipped profiles, sorting each into what it did.
fn Screen_The_Shipped_Profiles(store: &SpecificationStore) -> Outcomes
{
    let mut screened = Outcomes::default();
    for profile in Shipped().Profiles()
    {
        // A subject profile refuses for a reason that is not about the corpus, so it would
        // report a false disagreement. Excluded, and the exclusion is asserted below so it
        // cannot quietly grow.
        if profile.Is_Per_Subject()
        {
            screened.skipped.insert(profile.id.clone());
        }
        else
        {
            Sort_One_Profile(store, profile, &mut screened);
        }
    }

    return screened;
}

/// Whether one profile renders, and whether the screen said it would.
fn Sort_One_Profile(store: &SpecificationStore, profile: &Profile, screened: &mut Outcomes)
{
    let reaches = Is_Reaching_Only_Seeded_Content(profile);
    let rendered = Build(store, profile).is_ok();

    assert!(
        !rendered || reaches,
        "{} renders over a seeded store while reaching a kind SEEDED_BY_RECORDS does not \
         admit, so the declaration is missing a kind that is really seeded",
        profile.id
    );

    if rendered
    {
        screened.renders.insert(profile.id.clone());
    }
    else if reaches
    {
        screened.screened_but_refuses.insert(profile.id.clone());
    }
}

/// The screen against the behaviour, in the one direction that holds — and the witnesses to
/// the direction that does not.
///
/// Rendering implies reaching only seeded kinds. The converse fails, and the two profiles that
/// pass the screen and still refuse are named, because reading the screen as the answer is the
/// error this asserts against. `api-documentation` and `requirement-catalog` refuse for the
/// same filter reason but appear in neither set: they also reach `statements` and so fail the
/// screen first, which is why the witness list is shorter than the list of profiles a
/// corpus-kind filter stops.
///
/// The set that renders is pinned too. Without that, a change making a rebuildable profile
/// refuse would move it quietly into the witness list and pass.
#[test]
fn Test_Rendering_Over_A_Seeded_Store_Should_Imply_Reaching_Only_Seeded_Content()
{
    let store = Seeded();
    let screened = Screen_The_Shipped_Profiles(&store);

    Assert_The_Rendering_Profiles_Are_The_Pinned_Set(&screened);
    Assert_The_Witnesses_Are_The_Pinned_Pair(&screened);
    Assert_The_Skipped_Profiles_Are_The_Pinned_Set(&screened);
}

/// The profiles that render over a seeded store: the set the Required projections step may
/// draw from, so a change here changes what the gate can ask for.
fn Assert_The_Rendering_Profiles_Are_The_Pinned_Set(screened: &Outcomes)
{
    assert_eq!(
        screened.renders,
        // `relation-families` renders here and is deliberately *not* required.
        // `OD-PROJECT-006` decided the required relation projection is the full one, because a
        // projection is required for being a re-render obligation and this is a reading aid
        // nobody compares against the store. That it *could* be required is what this set
        // says; what the gate asks for is `Test_The_Required_Projections_Should_Render`.
        Named(&[
            "diagram-set",
            "domain-specification",
            "html-site",
            "relation-families",
            "traceability-matrix",
        ]),
        "the set of profiles that render without a corpus moved. This is the set the Required \
         projections step may draw from, so a change here changes what the gate can ask for."
    );
}

/// The profiles the screen passes and the build still refuses: the witnesses to
/// `Is_Reaching_Only_Seeded_Content` being necessary and not sufficient. An empty set would mean
/// the screen had silently become the answer.
fn Assert_The_Witnesses_Are_The_Pinned_Pair(screened: &Outcomes)
{
    assert_eq!(
        screened.screened_but_refuses,
        Named(&["feature-design", "release-specification"]),
        "the witnesses to Is_Reaching_Only_Seeded_Content being insufficient moved. Each reaches \
         only seeded kinds and still refuses, because it filters nodes on a kind only a corpus \
         has. An empty set here would mean the screen had silently become the answer."
    );
}

/// The subject-scoped profiles this comparison says nothing about, which the gate exercises
/// through `Test_Every_Required_Profile_Should_Render_Over_A_Seeded_Store` instead.
fn Assert_The_Skipped_Profiles_Are_The_Pinned_Set(screened: &Outcomes)
{
    assert_eq!(
        screened.skipped,
        // `implementation-context-pack` joined the subject-scoped four when `OD-PROJECT-005`
        // scoped it: its output path carries `{subject}` and it cannot be built without one,
        // so it is screened out here for the same reason they are rather than for one of its
        // own. `Test_The_Required_Projections_Should_Render` is where a subject-scoped profile
        // is really exercised.
        Named(&[
            "implementation-context-pack",
            "subject-contract",
            "subject-dossier",
            "subject-model",
            "subject-report",
        ]),
        "the set of profiles excluded from this comparison moved. A profile excluded here is a \
         profile this test says nothing about."
    );
}

/// Every profile the gate requires, rendered rather than predicted.
///
/// This is the authority on requirability, because the screen cannot see a filter. A change
/// making a required profile refuse fails here instead of on the next person's commit, which
/// is the difference between a red gate somebody caused and one somebody inherited.
///
/// The pair is written here and in `.github/workflows/gate.yml`. That duplication is chosen:
/// the workflow is what CI runs and this is what a local `cargo test` can check, and deriving
/// one from the other would put a workflow parser in this band. `OD-PROJECT-002` records it.
/// How many profiles the gate names as required, which is the length of the pair below.
const REQUIRED_PROFILE_COUNT: usize = 2;

/// Every profile the gate names as required, in `.github/workflows/gate.yml`.
fn Required_Profile_Ids() -> [&'static str; REQUIRED_PROFILE_COUNT]
{
    return ["diagram-set", "domain-specification"];
}

#[test]
fn Test_Every_Required_Profile_Should_Render_Over_A_Seeded_Store()
{
    let store = Seeded();

    for id in Required_Profile_Ids()
    {
        let profile = Profile_Named(id);

        assert!(
            Is_Reaching_Only_Seeded_Content(&profile),
            "{id} is required by the gate and reaches a content kind no runner can answer for"
        );
        Build(&store, &profile)
            // This panic is the test. Both profiles are named in `.github/workflows/gate.yml`,
            // so a refusal here is a gate that will go red on the next commit, and the point
            // of saying so with the id and the cause is that it fails for the person who
            // caused it rather than for whoever pushes next.
            .unwrap_or_else(|error| panic!("{id} is required by the gate and refused: {error}"));
    }
}
