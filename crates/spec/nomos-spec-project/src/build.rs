// Whether what was built is still current, which only a build can answer.
mod freshness;
mod stamp;

pub use freshness::Freshness;
pub use stamp::Stamp;

use crate::Output;
use crate::Profile;
use crate::Projection;
use crate::ProjectError;
use nomos_spec_model::ContentHash;
use nomos_spec_store::SpecificationStore;

pub const SIDECAR_SUFFIX: &str = ".nomos-projection.json";

pub fn Build_Projection(store: &SpecificationStore, profile: &Profile) -> Result<Output, ProjectError>
{
    use crate::Render_Projection;
    use crate::Select_Projection;

    profile.Validate()?;
    Refuse_Unresolved(profile)?;

    let projection = Select_Projection(store, profile)?;
    let body = Render_Projection(&projection)?;

    return Ok(Output {
        path: profile.output.clone(),
        sidecar_path: format!("{}{SIDECAR_SUFFIX}", profile.output),
        stamp: Stamped_Projection(profile, &projection, &body),
        body,
    });
}

// Kept alongside `Build_Projection` because the real name (chosen for check-naming-clarity)
// is called from `crates/orchestration/nomos-spec-orchestration`, from
// `tests/integration/tests/determinism/spec_productions.rs`, and is checked by literal
// name in `tests/contract/tests/public_surface/scanner.rs` — none of which sit inside this
// campaign's `crates/spec` territory.
pub use self::Build_Projection as Build;

/// The last place a template can be caught before it becomes a directory.
///
/// `Resolved_For` is the only thing that removes the placeholder, so a profile arriving
/// here with one still in it was never resolved — and rendering it would quietly create a
/// path named after the placeholder rather than after any subject.
fn Refuse_Unresolved(profile: &Profile) -> Result<(), ProjectError>
{
    if !profile.Is_Per_Subject()
    {
        return Ok(());
    }

    return Err(ProjectError::SubjectUnresolved {
        profile: profile.id.clone(),
        output: profile.output.clone(),
    });
}

fn Stamped_Projection(profile: &Profile, projection: &Projection, body: &str) -> Stamp
{
    return Stamp {
        profile: profile.id.clone(),
        profile_digest: profile.Digest(),
        format: profile.format,
        output: profile.output.clone(),
        content_digest: ContentHash::Of(body).As_String_Slice().to_owned(),
        inputs_digest: projection.Inputs_Digest(),
        sections: projection
            .sections
            .iter()
            .map(|section| {
                return (
                    section.title.clone(),
                    u32::try_from(section.items.len()).unwrap_or(u32::MAX),
                );
            })
            .collect(),
        inputs: projection.inputs.clone(),
    };
}

pub fn Check_Freshness(
    store: &SpecificationStore,
    profile: &Profile,
    body: Option<&str>,
    sidecar: Option<&str>,
) -> Result<Freshness, ProjectError>
{
    let (Some(body), Some(sidecar)) = (body, sidecar)
    else
    {
        return Ok(Freshness {
            absent: true,
            ..Freshness::default()
        });
    };

    let recorded = Stamp::Parse(sidecar)?;
    let rebuilt = Build_Projection(store, profile)?;
    let found = ContentHash::Of(body).As_String_Slice().to_owned();

    let mut freshness = Freshness {
        stale: Stale_Inputs(&recorded, &rebuilt),
        edited: Edited_Content(&recorded, &found),
        ..Freshness::default()
    };
    freshness.diverged = Diverged_Bytes(&freshness, body, found, rebuilt);

    return Ok(freshness);
}

// Kept alongside `Check_Freshness` because the real name (chosen for check-naming-clarity)
// is called from `crates/orchestration/nomos-spec-orchestration`, outside this campaign's
// `crates/spec` territory.
pub use self::Check_Freshness as Check;

/// The stamp's inputs against what the store now holds.
///
/// The profile digest counts as an input: a projection built from the same rows under a
/// changed profile is a different document, and reporting it current would be a lie about
/// the only thing that changed.
fn Stale_Inputs(recorded: &Stamp, rebuilt: &Output) -> Option<(String, String)>
{
    if recorded.inputs_digest == rebuilt.stamp.inputs_digest
        && recorded.profile_digest == rebuilt.stamp.profile_digest
    {
        return None;
    }

    return Some((
        recorded.inputs_digest.clone(),
        rebuilt.stamp.inputs_digest.clone(),
    ));
}

/// The stamp's digest against the file it describes.
fn Edited_Content(recorded: &Stamp, found: &str) -> Option<(String, String)>
{
    if recorded.content_digest == found
    {
        return None;
    }

    return Some((recorded.content_digest.clone(), found.to_owned()));
}

/// The third comparison, and the only one that reads the store's own bytes.
///
/// `rebuilt` has been sitting there since the stale check with its body never looked at, so
/// a body edited together with the `content_digest` describing it satisfied both tests above
/// and reported current.
///
/// Only asked once the other two have come back clean, because it cannot distinguish a
/// cause. A stale output differs from `rebuilt.body` too, and so does an edited one, so
/// reporting this whenever the bytes differ would say `diverged` alongside every other
/// verdict and stop being the name of anything. Rendering is deterministic over the inputs
/// and the profile, and both digests have just been found to match the rebuild, so an honest
/// stamp guarantees these bytes are equal: reaching here means the pair was written by
/// something other than `Build`.
fn Diverged_Bytes(
    freshness: &Freshness,
    body: &str,
    found: String,
    rebuilt: Output,
) -> Option<(String, String)>
{
    if Is_Already_Explained(freshness, body, &rebuilt)
    {
        return None;
    }

    return Some((found, rebuilt.stamp.content_digest));
}

/// Whether another verdict already accounts for these bytes.
///
/// A stale pair and an edited one both differ from the rebuild too, so divergence is only
/// the answer once neither of those is what happened and the bytes still match.
fn Is_Already_Explained(freshness: &Freshness, body: &str, rebuilt: &Output) -> bool
{
    return freshness.stale.is_some() || freshness.edited.is_some() || body == rebuilt.body;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

    fn Populated_Store() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO suites (suite_id, title, authority_root) \
                 VALUES ('nomos', 'The Nomos specification', 1);",
            )
            .expect("seeds a suite");

        return store;
    }

    fn One_Section_Profile() -> Profile
    {
        return Profile::Parse(
            r#"{
                "id": "one", "title": "One", "format": "markdown", "output": "one.md",
                "sections": [{ "title": "Suites", "content": "suites" }]
            }"#,
        )
        .expect("parses");
    }

    #[test]
    fn Test_Build_Projection_Should_Render_And_Stamp_A_Whole_Store_Profile()
    {
        let store = Populated_Store();
        let profile = One_Section_Profile();

        let output = Build_Projection(&store, &profile).expect("builds");

        assert_eq!(output.path, "one.md");
        assert!(output.body.contains("nomos"), "{}", output.body);
        assert_eq!(output.stamp.profile, "one");
    }

    #[test]
    fn Test_Build_Projection_Should_Refuse_A_Profile_Whose_Subject_Was_Never_Resolved()
    {
        let store = Populated_Store();
        let mut profile = One_Section_Profile();
        profile.output = "{subject}.md".to_owned();

        let refusal = Build_Projection(&store, &profile).expect_err("must refuse");

        assert!(matches!(refusal, ProjectError::SubjectUnresolved { .. }), "{refusal:?}");
    }

    #[test]
    fn Test_Check_Freshness_Should_Report_Absent_When_Nothing_Was_Built_Yet()
    {
        let store = Populated_Store();
        let profile = One_Section_Profile();

        let freshness = Check_Freshness(&store, &profile, None, None).expect("checks");

        assert!(freshness.absent);
    }

    #[test]
    fn Test_Check_Freshness_Should_Report_Current_When_Nothing_Changed_Since_The_Build()
    {
        let store = Populated_Store();
        let profile = One_Section_Profile();
        let output = Build_Projection(&store, &profile).expect("builds");
        let sidecar = output.Sidecar().expect("renders");

        let freshness =
            Check_Freshness(&store, &profile, Some(&output.body), Some(&sidecar)).expect("checks");

        assert!(!freshness.absent);
        assert!(freshness.stale.is_none());
        assert!(freshness.edited.is_none());
        assert!(freshness.diverged.is_none());
    }
}
