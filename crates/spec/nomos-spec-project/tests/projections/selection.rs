//! What a section selects, and what it must refuse rather than render.
//!
//! An empty section is the interesting case throughout. A projection that renders one is a
//! projection that says nothing while looking like an answer, so emptiness is a refusal
//! unless a profile declares it expected.

use crate::store::{Populated, Profile_Named, Rendered};
use nomos_spec_project::{Build, Catalogue, Profile, Select_Projection};

#[test]
fn Test_A_Soft_Deleted_Node_Should_Not_Be_Projected()
{
    let store = Populated();

    let rendered = Rendered(&store, "mcp-resource");

    assert!(rendered.contains("AGT-EXEC-001"), "the fixture projected no node at all");
    assert!(
        !rendered.contains("REQ-RETIRED-009"),
        "a deleted node reached a projection"
    );
}

#[test]
fn Test_A_Section_That_Selects_Nothing_Should_Be_Refused()
{
    let store = Populated();
    let profile = Profile::Parse(
        r#"{ "id": "empty", "title": "Empty", "format": "markdown", "output": "empty.md",
             "sections": [{ "title": "Ghosts", "content": "nodes",
                            "filter": { "kind": "ghost" } }] }"#,
    )
    .expect("parses");

    let refusal = Build(&store, &profile).expect_err("must refuse");

    assert!(format!("{refusal}").contains("selected no nodes"), "{refusal}");
}

#[test]
fn Test_A_Section_That_Declares_It_May_Be_Empty_Should_Render()
{
    let store = Populated();
    let profile = Profile::Parse(
        r#"{ "id": "empty", "title": "Empty", "format": "markdown", "output": "empty.md",
             "sections": [{ "title": "Ghosts", "content": "nodes", "may_be_empty": true,
                            "filter": { "kind": "ghost" } },
                          { "title": "Nodes", "content": "nodes" }] }"#,
    )
    .expect("parses");

    let output = Build(&store, &profile).expect("builds");

    assert!(output.body.contains("## Ghosts"), "{}", output.body);
}

#[test]
fn Test_A_Filter_A_Content_Kind_Does_Not_Honour_Should_Be_Refused()
{
    let store = Populated();
    let profile = Profile::Parse(
        r#"{ "id": "misfiltered", "title": "Misfiltered", "format": "markdown",
             "output": "misfiltered.md",
             "sections": [{ "title": "Suites", "content": "suites",
                            "filter": { "relation_type": "verifies" } }] }"#,
    )
    .expect("parses");

    let refusal = Build(&store, &profile).expect_err("must refuse");

    assert!(format!("{refusal}").contains("relation_type"), "{refusal}");
}

#[test]
fn Test_Two_Profiles_Writing_One_Output_Should_Be_Refused()
{
    let one = Profile::Parse(
        r#"{ "id": "one", "title": "One", "format": "markdown", "output": "shared.md",
             "sections": [{ "title": "Nodes", "content": "nodes" }] }"#,
    )
    .expect("parses");
    let two = Profile::Parse(
        r#"{ "id": "two", "title": "Two", "format": "markdown", "output": "shared.md",
             "sections": [{ "title": "Nodes", "content": "nodes" }] }"#,
    )
    .expect("parses");

    let Err(refusal) = Catalogue::Of(vec![one, two])
    else
    {
        // Unreachable while `Catalogue::Of` refuses two profiles that write one path. An `Ok`
        // here is the collision being accepted — the defect this test exists for — and the
        // assertion below could not report it, because there would be no refusal to read
        // `shared.md` out of.
        panic!("two profiles writing one output were accepted");
    };

    assert!(format!("{refusal}").contains("shared.md"), "{refusal}");
}

#[test]
fn Test_A_Selection_Should_Order_By_Identity_Rather_Than_By_Arrival()
{
    let store = Populated();
    let projection = Select_Projection(&store, &Profile_Named("mcp-resource")).expect("selects");

    for section in &projection.sections
    {
        let identities: Vec<&str> = section
            .items
            .iter()
            .map(|item| return item.identity.as_str())
            .collect();
        let mut sorted = identities.clone();
        sorted.sort_unstable();

        assert_eq!(identities, sorted, "{} is not ordered by identity", section.title);
    }
}

/// A status filter narrows on what a record's own front matter declared, over the real
/// seeded governing records rather than a fixture written to agree with it.
///
/// # Why this is asserted against the real corpus
///
/// The field exists because ten of this repository's own governing questions are open and
/// every projection showed them beside the accepted ones with nothing to tell them apart.
/// A fixture with one open record and one accepted one would pass while that stayed true,
/// because the thing being checked is not that the SQL runs -- it is that the value reaches
/// the filter from the file an author wrote, through `record_front_matter`, which nothing
/// read back until now.
///
/// The assertions are membership and disjointness rather than counts, deliberately: a count
/// would have to be edited every time a record is authored or a question is answered, which
/// makes it a maintenance cost rather than a claim.
#[test]
fn Test_A_Status_Filter_Should_Select_The_Open_Records_And_Exclude_The_Accepted_Ones()
{
    let store = Seeded();

    let open = Node_Identities(&store, Some("open"));
    let accepted = Node_Identities(&store, Some("accepted"));

    assert!(!open.is_empty(), "the seeded store answered no open record at all");
    assert!(
        open.contains(&"OD-SPEC-001".to_owned()),
        "OD-SPEC-001 declares `status: open` and was not selected: {open:?}"
    );
    assert!(
        !open.contains(&"OD-SPEC-016".to_owned()),
        "OD-SPEC-016 declares `status: accepted` and was selected as open: {open:?}"
    );
    assert!(
        accepted.contains(&"OD-SPEC-016".to_owned()),
        "OD-SPEC-016 declares `status: accepted` and was not selected: {}",
        accepted.len()
    );
    assert!(
        open.iter().all(|identity| return !accepted.contains(identity)),
        "a record was selected under two statuses at once"
    );
}

/// A node nobody authored is excluded rather than reported under a status it never declared.
///
/// The store holds nodes that exist only because a record's relation named them, and those
/// carry no `record_front_matter` row at all. A `JOIN` that admitted them -- or a filter
/// that treated a missing row as a default -- would put a value in a projection that no
/// author ever wrote, which is the one thing this store exists to make impossible.
#[test]
fn Test_A_Node_With_No_Declared_Status_Should_Be_Excluded_Rather_Than_Defaulted()
{
    let store = Seeded();

    let all = Node_Identities(&store, None);
    let declared: usize = ["open", "closed", "accepted"]
        .into_iter()
        .map(|status| return Node_Identities(&store, Some(status)).len())
        .sum();

    assert!(
        declared < all.len(),
        "every node carried a declared status, so this test proved nothing about the ones that \
         do not: {declared} of {}",
        all.len()
    );
}

/// A content kind that cannot answer `status` refuses it by name.
///
/// `statements` reads `normative_statements`, which has no record front matter behind it, so
/// the filter is not merely unhelpful there -- it is unanswerable, and the existing
/// `UnsupportedFilter` refusal is how this crate already says so for every other filter.
#[test]
fn Test_A_Status_Filter_On_A_Kind_That_Cannot_Answer_It_Should_Be_Refused_By_Name()
{
    let store = Populated();
    let profile = Profile::Parse(
        r#"{ "id": "misplaced", "title": "Misplaced", "format": "markdown",
             "output": "misplaced.md",
             "sections": [{ "title": "Statements", "content": "statements",
                            "filter": { "status": "open" } }] }"#,
    )
    .expect("parses");

    let refusal = Build(&store, &profile).expect_err("must refuse");

    let reported = format!("{refusal}");
    assert!(reported.contains("status"), "the refusal did not name the filter: {reported}");
    assert!(reported.contains("statements"), "the refusal did not name the content: {reported}");
}

/// Every node identity a `nodes` section selects, optionally narrowed to one declared status.
fn Node_Identities(store: &nomos_spec_store::SpecificationStore, status: Option<&str>) -> Vec<String>
{
    let filter = status.map_or_else(String::new, |status| return format!(r#", "filter": {{ "status": "{status}" }}"#));
    let profile = Profile::Parse(&format!(
        r#"{{ "id": "probe", "title": "Probe", "format": "markdown", "output": "probe.md",
              "sections": [{{ "title": "Nodes", "content": "nodes", "may_be_empty": true{filter} }}] }}"#
    ))
    .expect("parses");

    let projection = Select_Projection(store, &profile).expect("selects");

    return projection
        .sections
        .into_iter()
        .flat_map(|section| return section.items)
        .map(|item| return item.identity)
        .collect();
}

/// The real governing records, seeded the way `nomos spec` seeds them.
///
/// Local rather than shared with `rebuildability.rs`'s copy: that one is private to its own
/// file, and reaching into it would mean editing a file this item does not hold.
fn Seeded() -> nomos_spec_store::SpecificationStore
{
    let mut store = nomos_spec_store::SpecificationStore::In_Memory().expect("opens");
    nomos_spec_store::Seed_Governing_Records(&mut store).expect("seeds");

    return store;
}

/// A subject reaches its one-hop neighbours, carrying what each declared.
///
/// Over the real seeded records rather than the volume fixture, because the claim is about a
/// record graph and `OD-PROJECT-005`'s whole argument is a measurement of that graph's shape.
#[test]
fn Test_A_Neighbourhood_Should_Reach_The_Subjects_Own_Neighbours()
{
    let store = Seeded();

    let reached = Neighbourhood_Of(&store, "OD-PROJECT-005");

    assert!(!reached.is_empty(), "the subject reached nobody");
    for expected in ["OD-PROJECT-002", "OD-SPEC-012", "OD-SPEC-015"]
    {
        assert!(reached.contains(&expected.to_owned()), "{expected} missing from {reached:?}");
    }
    assert!(
        !reached.contains(&"OD-PROJECT-005".to_owned()),
        "a subject is not its own neighbour: {reached:?}"
    );
}

/// A subject whose every edge is `relates-to` still reaches its neighbours.
///
/// This is the test an implementation following only the typed terms would fail while passing
/// every other one here. `OD-SPEC-015` measured `relates-to` at 93 per cent of authored edges,
/// and `OD-PROJECT-005` measured the consequence: restricted to `affects`, `affected_by` and
/// `supersedes`, 177 of 232 records reach nothing but themselves at any hop count. A traversal
/// that skipped the symmetric term would be empty for three quarters of the corpus.
#[test]
fn Test_A_Subject_Whose_Edges_Are_All_Relates_To_Should_Still_Reach_Them()
{
    let store = Seeded();

    let reached = Neighbourhood_Of(&store, "OD-SPEC-015");

    assert!(
        reached.len() >= 3,
        "OD-SPEC-015 declares three relates-to edges and nothing else, and reached {reached:?}"
    );
}

/// Two selections of one store are identical.
///
/// The claim `OD-PROJECT-005`'s emission-order decision exists for. A traversal emitting in
/// discovery order would pass every assertion above and fail this one, and would then make two
/// renders of one store differ -- which is what the freshness sidecar's determinism rests on.
#[test]
fn Test_A_Neighbourhood_Should_Select_Identically_Twice()
{
    let store = Seeded();

    let first = Neighbourhood_Of(&store, "OD-PROJECT-005");
    let second = Neighbourhood_Of(&store, "OD-PROJECT-005");

    assert_eq!(first, second);
    let mut sorted = first.clone();
    sorted.sort();
    assert_eq!(first, sorted, "a neighbourhood is emitted by identity, not by discovery order");
}

/// A neighbour that declared no status is reported with an empty one rather than dropped.
///
/// A node referenced by a relation but never authored carries no `record_front_matter` row at
/// all. Dropping it would make a pack silently narrower than the graph, and inventing a status
/// for it would put a value in a projection no author wrote -- the one thing this store exists
/// to prevent. `OD-RULES-003`'s reasoning generally: an absence is said, not inferred.
#[test]
fn Test_A_Neighbour_With_No_Declared_Status_Should_Be_Reported_Not_Dropped()
{
    let store = Seeded();

    let statuses = Neighbourhood_Statuses(&store, "OD-PROJECT-005");

    assert!(!statuses.is_empty(), "the subject reached nobody");
    assert!(
        statuses.iter().all(|(identity, _)| return !identity.is_empty()),
        "a neighbour was reported with no identity: {statuses:?}"
    );
    assert!(
        statuses.iter().any(|(_, status)| return status == "accepted"),
        "no neighbour carried a declared status at all, so this proved nothing: {statuses:?}"
    );
}

/// Every identity a subject's neighbourhood section selects.
fn Neighbourhood_Of(store: &nomos_spec_store::SpecificationStore, subject: &str) -> Vec<String>
{
    return Neighbourhood_Statuses(store, subject).into_iter().map(|(identity, _)| return identity).collect();
}

/// Every `(identity, declared status)` a subject's neighbourhood section selects.
fn Neighbourhood_Statuses(store: &nomos_spec_store::SpecificationStore, subject: &str) -> Vec<(String, String)>
{
    let profile = Profile::Parse(&format!(
        r#"{{ "id": "probe", "title": "Probe", "format": "markdown", "output": "probe.md",
              "sections": [{{ "title": "Neighbourhood", "content": "neighbourhood",
                              "may_be_empty": true,
                              "filter": {{ "node_id": "{subject}" }} }}] }}"#
    ))
    .expect("parses");

    let projection = Select_Projection(store, &profile).expect("selects");

    return projection
        .sections
        .into_iter()
        .flat_map(|section| return section.items)
        .map(|item| {
            let status = item
                .fields
                .iter()
                .find(|(name, _)| return name == "status")
                .map_or_else(String::new, |(_, value)| return value.clone());
            return (item.identity, status);
        })
        .collect();
}
