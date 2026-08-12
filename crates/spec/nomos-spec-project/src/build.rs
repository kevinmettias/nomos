// Whether what was built is still current, which only a build can answer.
mod freshness;

pub use freshness::Freshness;

use crate::Output;
use crate::Format;
use crate::Profile;
use crate::Input;
use crate::Projection;
use crate::Render;
use crate::ProjectError;
use nomos_spec_model::ContentHash;
use nomos_spec_store::SpecificationStore;
use serde::{Deserialize, Serialize};

pub const SIDECAR_SUFFIX: &str = ".nomos-projection.json";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp
{
    pub profile: String,
    pub profile_digest: String,
    pub format: Format,
    pub output: String,
    pub content_digest: String,
    pub inputs_digest: String,
    pub sections: Vec<(String, u32)>,
    pub inputs: Vec<Input>,
}

impl Stamp
{
    pub fn Parse(text: &str) -> Result<Self, ProjectError>
    {
        return serde_json::from_str(text)
            .map_err(|error| return ProjectError::Malformed(error.to_string()));
    }

    pub fn Render(&self) -> Result<String, ProjectError>
    {
        let mut rendered = serde_json::to_string_pretty(self)
            .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
        rendered.push('\n');

        return Ok(rendered);
    }
}

pub fn Build(store: &SpecificationStore, profile: &Profile) -> Result<Output, ProjectError>
{
    use crate::Select;

    profile.Validate()?;
    Refuse_Unresolved(profile)?;

    let projection = Select(store, profile)?;
    let body = Render(&projection)?;

    return Ok(Output {
        path: profile.output.clone(),
        sidecar_path: format!("{}{SIDECAR_SUFFIX}", profile.output),
        stamp: Stamped(profile, &projection, &body),
        body,
    });
}

/// The last place a template can be caught before it becomes a directory.
///
/// `Resolved_For` is the only thing that removes the placeholder, so a profile arriving
/// here with one still in it was never resolved — and rendering it would quietly create a
/// path named after the placeholder rather than after any subject.
fn Refuse_Unresolved(profile: &Profile) -> Result<(), ProjectError>
{
    if !profile.Names_A_Subject()
    {
        return Ok(());
    }

    return Err(ProjectError::SubjectUnresolved {
        profile: profile.id.clone(),
        output: profile.output.clone(),
    });
}

fn Stamped(profile: &Profile, projection: &Projection, body: &str) -> Stamp
{
    return Stamp {
        profile: profile.id.clone(),
        profile_digest: profile.Digest(),
        format: profile.format,
        output: profile.output.clone(),
        content_digest: ContentHash::Of(body).As_Str().to_owned(),
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

pub fn Check(
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
    let rebuilt = Build(store, profile)?;
    let found = ContentHash::Of(body).As_Str().to_owned();

    let mut freshness = Freshness {
        stale: Stale(&recorded, &rebuilt),
        edited: Edited(&recorded, &found),
        ..Freshness::default()
    };
    freshness.diverged = Diverged(&freshness, body, found, rebuilt);

    return Ok(freshness);
}

/// The stamp's inputs against what the store now holds.
///
/// The profile digest counts as an input: a projection built from the same rows under a
/// changed profile is a different document, and reporting it current would be a lie about
/// the only thing that changed.
fn Stale(recorded: &Stamp, rebuilt: &Output) -> Option<(String, String)>
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
fn Edited(recorded: &Stamp, found: &str) -> Option<(String, String)>
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
/// Whether another verdict already accounts for these bytes.
///
/// A stale pair and an edited one both differ from the rebuild too, so divergence is only
/// the answer once neither of those is what happened and the bytes still match.
fn Already_Explained(freshness: &Freshness, body: &str, rebuilt: &Output) -> bool
{
    return freshness.stale.is_some() || freshness.edited.is_some() || body == rebuilt.body;
}

fn Diverged(
    freshness: &Freshness,
    body: &str,
    found: String,
    rebuilt: Output,
) -> Option<(String, String)>
{
    if Already_Explained(freshness, body, &rebuilt)
    {
        return None;
    }

    return Some((found, rebuilt.stamp.content_digest));
}
