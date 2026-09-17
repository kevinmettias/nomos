//! The four verdicts a built file can carry, and which of them owns which case.
//!
//! The verdicts overlap by construction — a hand edit and a moved store both make a body
//! disagree with what the profile now renders — so each test asserts the verdicts it must
//! *not* get as well as the one it must. A report where two verdicts claim one case is a
//! report where neither names anything in particular.

use crate::store::{Populated, Profile_Named};
use nomos_spec_project::{Build, Check, Freshness, Output, Profile, Stamp, SIDECAR_SUFFIX};
use nomos_spec_store::SpecificationStore;

#[test]
fn Test_The_Sidecar_Should_Round_Trip()
{
    let store = Populated();
    let output = Build(&store, &Profile_Named("domain-specification"))
        .expect("the fixture renders the shipped domain-specification profile");

    let parsed = Stamp::Parse(
        &output.Sidecar().expect("every built output carries a rendered stamp"),
    )
    .expect("the sidecar text came from Stamp::Render in the line above");

    assert_eq!(parsed, output.stamp);
    assert_eq!(output.sidecar_path, format!("{}{SIDECAR_SUFFIX}", output.path));
}

/// The stamp text that sits beside a body, wrapped so a caller cannot transpose it with the body.
///
/// `Check_Body_And_Sidecar` hands both to `Check` in a fixed order, and as two bare `&str` the
/// compiler would accept either arrangement.
struct SidecarText<'a>(&'a str);

/// The freshness of a body and the stamp beside it, against the store they came from.
fn Check_Body_And_Sidecar(
    store: &SpecificationStore,
    profile: &Profile,
    body: &str,
    sidecar: SidecarText<'_>,
) -> Freshness
{
    return Check(store, profile, Some(body), Some(sidecar.0))
        .expect("the store, the profile and the body all came from one build");
}

#[test]
fn Test_An_Unchanged_Store_Should_Be_Fresh()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("the fixture renders this shipped profile");

    let freshness = Check_Body_And_Sidecar(
        &store,
        &profile,
        &built.body,
        SidecarText(&built.Sidecar().expect("every built output carries a rendered stamp")),
    );

    assert!(freshness.Is_Fresh(), "{}", freshness.Report(&built.path));
}

#[test]
fn Test_A_Changed_Store_Should_Be_Stale()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("the fixture renders this shipped profile");
    let stamp = built.Sidecar().expect("every built output carries a rendered stamp");

    Rename_The_Fixtures_Node(&store);
    let freshness = Check_Body_And_Sidecar(&store, &profile, &built.body, SidecarText(&stamp));

    assert!(freshness.stale.is_some(), "a changed store reads as current");
    assert!(freshness.Report(&built.path).contains("stale"));
    // The store moving is not the file being written by hand, and the third verdict must not
    // swallow the first. This body does differ from what the store now renders, so a
    // divergence check asked unconditionally would report both and `diverged` would stop
    // naming anything in particular.
    assert!(
        freshness.diverged.is_none(),
        "a store that moved under an untouched file reads as hand-written: {}",
        freshness.Report(&built.path)
    );
}

/// Moves the store out from under a stamp by retitling the node every profile renders.
fn Rename_The_Fixtures_Node(store: &SpecificationStore)
{
    store
        .Connection()
        .execute("UPDATE nodes SET title = 'Renamed' WHERE node_id = 'AGT-EXEC-001'", [])
        .expect("changes the store");
}

#[test]
fn Test_A_Changed_Profile_Should_Be_Stale()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("the fixture renders this shipped profile");
    let mut retitled = profile.clone();
    retitled.title = "Renamed resource".to_owned();

    let freshness = Check_Body_And_Sidecar(
        &store,
        &retitled,
        &built.body,
        SidecarText(&built.Sidecar().expect("every built output carries a rendered stamp")),
    );

    assert!(freshness.stale.is_some(), "a rewritten profile reads as current");
}

#[test]
fn Test_An_Edited_Output_Should_Be_Reported_As_Edited()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("the fixture renders this shipped profile");
    let tampered = format!("{}\nhand written\n", built.body);

    let freshness = Check_Body_And_Sidecar(
        &store,
        &profile,
        &tampered,
        SidecarText(&built.Sidecar().expect("every built output carries a rendered stamp")),
    );

    assert!(freshness.edited.is_some(), "a hand-edited output reads as generated");
    assert!(freshness.stale.is_none(), "the store did not change");
    assert!(
        freshness.diverged.is_none(),
        "an edit its own stamp already contradicts is reported twice: {}",
        freshness.Report(&built.path)
    );
}

/// The case the other three cannot see.
///
/// `stale` compares the stamp's inputs against the store and `edited` compares the stamp's
/// digest against the file, so a body and the `content_digest` describing it, rewritten
/// together, agree with each other and satisfy both. Before this, that reported "is
/// current" and exited 0 — which is what the gate step added by P10-REQUIRED-PROJECTIONS
/// was resting on, since the commit that ships a projection can rewrite the stamp beside it.
///
/// `Test_An_Edited_Output_Should_Be_Reported_As_Edited` leaves the stamp alone, so it is
/// caught by the digest comparison and never reaches the bytes.
#[test]
fn Test_A_Stamp_Rewritten_To_Agree_With_An_Edited_Body_Should_Still_Be_Refused()
{
    let store = Populated();
    let profile = Profile_Named("mcp-resource");
    let built = Build(&store, &profile).expect("the fixture renders this shipped profile");
    let tampered = format!("{}\nhand written\n", built.body);
    let agreeing = Stamp_Agreeing_With(&built, &tampered);
    let freshness = Check_Body_And_Sidecar(&store, &profile, &tampered, SidecarText(&agreeing));

    assert!(
        !freshness.Is_Fresh(),
        "a body and a stamp rewritten together read as current: {}",
        freshness.Report(&built.path)
    );
    assert!(
        freshness.diverged.is_some(),
        "the refusal did not come from the comparison against what the store renders"
    );
    Assert_Divergence_Is_The_Only_Verdict(&freshness, &built.path);
}

/// `built`'s own stamp, re-rendered with a digest that agrees with `tampered`.
///
/// The edit is then invisible to the digest comparison, which is the only way to reach the
/// verdict this test exists for.
fn Stamp_Agreeing_With(built: &Output, tampered: &str) -> String
{
    use nomos_spec_model::ContentHash;

    let mut agreeing = built.stamp.clone();
    agreeing.content_digest = ContentHash::Of(tampered).As_String_Slice().to_owned();

    return agreeing.Render().expect("every stamp renders its own fields back as text");
}

/// Neither of the old two verdicts may claim the rewritten-stamp case. The stamp is internally
/// consistent, so saying "the stamp declares X and the file hashes to Y" would be false — they
/// are the same value — and the store never moved. The report needs words of its own too.
fn Assert_Divergence_Is_The_Only_Verdict(freshness: &Freshness, path: &str)
{
    assert!(freshness.edited.is_none(), "the stamp agrees with the file it describes");
    assert!(freshness.stale.is_none(), "the store did not change");

    let said = freshness.Report(path);

    assert!(said.contains("diverged"), "the new case has no words of its own: {said}");
    assert!(!said.contains("edited"), "the new case reuses the edited sentence: {said}");
}

#[test]
fn Test_An_Output_That_Was_Never_Built_Should_Be_Absent()
{
    let store = Populated();

    let freshness = Check(&store, &Profile_Named("mcp-resource"), None, None)
        .expect("Check accepts an absent body and sidecar and reports the file absent");

    assert!(freshness.absent);
    assert!(!freshness.Is_Fresh());
    assert!(freshness.Report("mcp/specification.json").contains("never been built"));
}
