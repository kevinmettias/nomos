use crate::ProjectError;
use nomos_spec_model::ContentHash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Format
{
    Markdown,
    Yaml,
    Json,
    Html,
    Mermaid,
    Contextpack,
}

impl Format
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Markdown => "markdown",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Html => "html",
            Self::Mermaid => "mermaid",
            Self::Contextpack => "contextpack",
        };
    }

    #[must_use]
    pub const fn Extension(self) -> &'static str
    {
        return match self
        {
            Self::Markdown => "md",
            Self::Yaml => "yaml",
            Self::Json | Self::Contextpack => "json",
            Self::Html => "html",
            Self::Mermaid => "mmd",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Markdown,
            Self::Yaml,
            Self::Json,
            Self::Html,
            Self::Mermaid,
            Self::Contextpack,
        ];
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Content
{
    Suites,
    Documents,
    Headings,
    Blocks,
    Rows,
    Nodes,
    Statements,
    Relations,
    Lineage,
    Omissions,
}

impl Content
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Suites => "suites",
            Self::Documents => "documents",
            Self::Headings => "headings",
            Self::Blocks => "blocks",
            Self::Rows => "rows",
            Self::Nodes => "nodes",
            Self::Statements => "statements",
            Self::Relations => "relations",
            Self::Lineage => "lineage",
            Self::Omissions => "omissions",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Suites,
            Self::Documents,
            Self::Headings,
            Self::Blocks,
            Self::Rows,
            Self::Nodes,
            Self::Statements,
            Self::Relations,
            Self::Lineage,
            Self::Omissions,
        ];
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Filter
{
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier_prefix: Option<String>,
    /// One node, by the identity it is addressed as.
    ///
    /// Apart from `identifier_prefix` because a prefix is not an identity. `FEAT-1` as a
    /// prefix also selects `FEAT-12`, and on relations it narrows only the end the edge
    /// starts from — so a subject asked for its relations would be shown the ones it
    /// declares and not the ones declared about it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Section
{
    pub title: String,
    pub content: Content,
    #[serde(default)]
    pub filter: Filter,
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub may_be_empty: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Profile
{
    pub id: String,
    pub title: String,
    pub format: Format,
    pub output: String,
    pub sections: Vec<Section>,
}

impl Profile
{
    pub fn Parse(text: &str) -> Result<Self, ProjectError>
    {
        let profile: Self = serde_json::from_str(text)
            .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
        profile.Validate()?;

        return Ok(profile);
    }

    pub fn Validate(&self) -> Result<(), ProjectError>
    {
        if self.id.trim().is_empty() || self.title.trim().is_empty()
        {
            return Err(ProjectError::Malformed(format!(
                "a profile needs an identifier and a title; {:?} has {:?}",
                self.id, self.title
            )));
        }

        if self.sections.is_empty()
        {
            return Err(ProjectError::Malformed(format!(
                "{} declares no section, so it would render a title and nothing under it",
                self.id
            )));
        }

        for section in &self.sections
        {
            if section.title.trim().is_empty()
            {
                return Err(ProjectError::Malformed(format!(
                    "{}: a section without a title is a heading nothing can cite",
                    self.id
                )));
            }
        }

        return Path_Is_Relative(&self.id, &self.output);
    }

    #[must_use]
    pub fn Digest(&self) -> String
    {
        let canonical = serde_json::to_string(self).unwrap_or_default();

        return ContentHash::Of(&canonical).As_Str().to_owned();
    }

    /// Whether this profile projects one subject rather than the whole store.
    ///
    /// Derived from the profile rather than declared beside it. A `subject: true` field
    /// would be a second place to be wrong: a profile could claim a subject and never use
    /// it, or use one and forget to say so, and the placeholder is the thing that actually
    /// decides what the run needs.
    #[must_use]
    pub fn Names_A_Subject(&self) -> bool
    {
        return self.output.contains(SUBJECT)
            || self.sections.iter().any(|section| {
                return section
                    .filter
                    .Named()
                    .iter()
                    .any(|(_, value)| return value.contains(SUBJECT));
            });
    }

    /// This profile, resolved against the subject a run was given or was not given.
    ///
    /// The two refusals are the point. Silently ignoring a subject a whole-store profile
    /// cannot use would write the same file once per subject and call it a per-subject
    /// build; silently rendering a subject profile without one would write a directory
    /// named `{subject}`. Both read as success.
    pub fn For(&self, subject: Option<&str>) -> Result<Self, ProjectError>
    {
        return match (self.Names_A_Subject(), subject)
        {
            (true, Some(subject)) => Ok(self.Resolved_For(subject)),
            (false, None) => Ok(self.clone()),
            (true, None) => Err(ProjectError::SubjectMissing {
                profile: self.id.clone(),
            }),
            (false, Some(subject)) => Err(ProjectError::SubjectUnexpected {
                profile: self.id.clone(),
                subject: subject.to_owned(),
            }),
        };
    }

    /// This profile with `{subject}` replaced throughout by the subject given.
    ///
    /// A copy rather than a mutation, because the catalogue is shared and a resolved
    /// profile is a different thing from the one that was shipped: its digest differs,
    /// which is what keeps two subjects' stamps from claiming to have been built by the
    /// same profile over different inputs.
    #[must_use]
    pub fn Resolved_For(&self, subject: &str) -> Self
    {
        let mut resolved = self.clone();
        resolved.output = resolved.output.replace(SUBJECT, subject);

        for section in &mut resolved.sections
        {
            section.filter.Substitute(subject);
        }

        return resolved;
    }
}

/// What a profile writes where a subject belongs.
pub const SUBJECT: &str = "{subject}";

fn Path_Is_Relative(profile: &str, output: &str) -> Result<(), ProjectError>
{
    let refused = output.trim().is_empty()
        || output.starts_with('/')
        || output.starts_with('\\')
        || output.contains(':')
        || output.contains('\\')
        || output.split('/').any(|segment| return segment == "..");

    if refused
    {
        return Err(ProjectError::NotRelative {
            profile: profile.to_owned(),
            output: output.to_owned(),
        });
    }

    return Ok(());
}

#[cfg(test)]
mod tests
{
    use super::*;

    const MINIMAL: &str = r#"{
        "id": "one", "title": "One", "format": "markdown", "output": "one.md",
        "sections": [{ "title": "Nodes", "content": "nodes" }]
    }"#;

    #[test]
    fn Test_A_Minimal_Profile_Should_Parse()
    {
        let profile = Profile::Parse(MINIMAL).expect("parses");

        assert_eq!(profile.format, Format::Markdown);
        assert_eq!(profile.sections.first().map(|section| section.content), Some(Content::Nodes));
        assert_eq!(profile.sections.first().map(|section| section.may_be_empty), Some(false));
    }

    #[test]
    fn Test_An_Unknown_Field_Should_Be_Refused()
    {
        let typo = MINIMAL.replace("\"content\": \"nodes\"", "\"content\": \"nodes\", \"fitler\": {}");

        let refusal = Profile::Parse(&typo).expect_err("must refuse");

        assert!(format!("{refusal}").contains("fitler"), "{refusal}");
    }

    #[test]
    fn Test_An_Unknown_Content_Kind_Should_Be_Refused()
    {
        let unknown = MINIMAL.replace("\"nodes\"", "\"everything\"");

        assert!(Profile::Parse(&unknown).is_err());
    }

    #[test]
    fn Test_A_Profile_With_No_Section_Should_Be_Refused()
    {
        let empty = MINIMAL.replace(
            "[{ \"title\": \"Nodes\", \"content\": \"nodes\" }]",
            "[]",
        );

        let refusal = Profile::Parse(&empty).expect_err("must refuse");

        assert!(format!("{refusal}").contains("declares no section"), "{refusal}");
    }

    #[test]
    fn Test_An_Absolute_Output_Should_Be_Refused()
    {
        for output in ["/etc/one.md", "C:/build/one.md", "..\\one.md", "../../one.md"]
        {
            let escaping = MINIMAL.replace("one.md", output);

            assert!(
                Profile::Parse(&escaping).is_err(),
                "{output} was accepted as a projection output"
            );
        }
    }

    #[test]
    fn Test_The_Digest_Should_Follow_The_Profile()
    {
        let profile = Profile::Parse(MINIMAL).expect("parses");
        let renamed = Profile::Parse(&MINIMAL.replace("\"One\"", "\"Two\"")).expect("parses");

        assert_eq!(profile.Digest(), Profile::Parse(MINIMAL).expect("parses").Digest());
        assert_ne!(profile.Digest(), renamed.Digest());
    }
}
