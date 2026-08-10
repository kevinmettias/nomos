use crate::profile::{Format, Profile};
use crate::projection::{Input, Projection};
use crate::render::Render;
use crate::select::Select;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output
{
    pub path: String,
    pub body: String,
    pub sidecar_path: String,
    pub stamp: Stamp,
}

impl Output
{
    pub fn Sidecar(&self) -> Result<String, ProjectError>
    {
        return self.stamp.Render();
    }
}

pub fn Build(store: &SpecificationStore, profile: &Profile) -> Result<Output, ProjectError>
{
    profile.Validate()?;

    // The last place a template can be caught before it becomes a directory. `Resolved_For`
    // is the only thing that removes the placeholder, so a profile arriving here with one
    // still in it was never resolved -- and rendering it would quietly create a path named
    // after the placeholder rather than after any subject.
    if profile.Names_A_Subject()
    {
        return Err(ProjectError::SubjectUnresolved {
            profile: profile.id.clone(),
            output: profile.output.clone(),
        });
    }

    let projection = Select(store, profile)?;
    let body = Render(&projection)?;

    return Ok(Output {
        path: profile.output.clone(),
        sidecar_path: format!("{}{SIDECAR_SUFFIX}", profile.output),
        stamp: Stamped(profile, &projection, &body),
        body,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Freshness
{
    pub absent: bool,
    pub stale: Option<(String, String)>,
    pub edited: Option<(String, String)>,
    /// The file and its stamp agree with each other and not with the store.
    ///
    /// `stale` reads the stamp's inputs against the store and `edited` reads the stamp's
    /// digest against the file. Both are satisfied by a body and a `content_digest`
    /// rewritten together: the pair is internally consistent, so neither comparison has
    /// anything to say, and neither of them ever looks at what the store renders. This is
    /// the residue — what is left over once the store has been ruled out as the cause and
    /// the stamp has been ruled out as out of date.
    pub diverged: Option<(String, String)>,
}

impl Freshness
{
    #[must_use]
    pub const fn Is_Fresh(&self) -> bool
    {
        return !self.absent
            && self.stale.is_none()
            && self.edited.is_none()
            && self.diverged.is_none();
    }

    #[must_use]
    pub fn Report(&self, output: &str) -> String
    {
        if self.absent
        {
            return format!("{output} has never been built");
        }

        let mut said = Vec::new();
        if let Some((declared, current)) = &self.stale
        {
            said.push(format!(
                "stale: built over inputs {declared} and the store now holds {current}"
            ));
        }
        if let Some((declared, found)) = &self.edited
        {
            said.push(format!(
                "edited: the stamp declares {declared} and the file hashes to {found}"
            ));
        }
        if let Some((agreed, rendered)) = &self.diverged
        {
            said.push(format!(
                "diverged: the file and its stamp agree on {agreed} and the store renders \
                 {rendered}"
            ));
        }
        if said.is_empty()
        {
            return format!("{output} is current");
        }

        return format!("{output} {}", said.join("; "));
    }
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
    let mut freshness = Freshness::default();

    if recorded.inputs_digest != rebuilt.stamp.inputs_digest
        || recorded.profile_digest != rebuilt.stamp.profile_digest
    {
        freshness.stale = Some((
            recorded.inputs_digest.clone(),
            rebuilt.stamp.inputs_digest.clone(),
        ));
    }

    let found = ContentHash::Of(body).As_Str().to_owned();
    if recorded.content_digest != found
    {
        freshness.edited = Some((recorded.content_digest.clone(), found.clone()));
    }

    // The third comparison, and the only one that reads the store's own bytes. `rebuilt` has
    // been sitting here since the stale check and its body was never looked at, so a body
    // edited together with the `content_digest` describing it satisfied both tests above and
    // reported current.
    //
    // Only asked once the other two have come back clean, because it cannot distinguish a
    // cause. A stale output differs from `rebuilt.body` too, and so does an edited one, so
    // reporting this whenever the bytes differ would say `diverged` alongside every other
    // verdict and stop being the name of anything. Rendering is deterministic over the
    // inputs and the profile, and both digests have just been found to match the rebuild, so
    // an honest stamp guarantees these bytes are equal: reaching here means the pair was
    // written by something other than `Build`.
    if freshness.stale.is_none() && freshness.edited.is_none() && body != rebuilt.body
    {
        freshness.diverged = Some((found, rebuilt.stamp.content_digest));
    }

    return Ok(freshness);
}
