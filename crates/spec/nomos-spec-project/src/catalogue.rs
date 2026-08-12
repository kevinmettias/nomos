use crate::Profile;
use crate::ProjectError;
use std::collections::{BTreeMap, BTreeSet};

pub const SHIPPED: &[(&str, &str)] = &[
    // Subject-addressed: one node, projected four ways. They share one selection and
    // differ only in format, which is the claim being made -- four artefacts about a
    // subject are four renderings of one record graph rather than four authorities.
    (
        "subject-dossier",
        include_str!("../profiles/subject-dossier.json"),
    ),
    (
        "subject-contract",
        include_str!("../profiles/subject-contract.json"),
    ),
    (
        "subject-model",
        include_str!("../profiles/subject-model.json"),
    ),
    (
        "subject-report",
        include_str!("../profiles/subject-report.json"),
    ),
    (
        "domain-specification",
        include_str!("../profiles/domain-specification.json"),
    ),
    (
        "architecture-document",
        include_str!("../profiles/architecture-document.json"),
    ),
    (
        "feature-design",
        include_str!("../profiles/feature-design.json"),
    ),
    (
        "api-documentation",
        include_str!("../profiles/api-documentation.json"),
    ),
    (
        "requirement-catalog",
        include_str!("../profiles/requirement-catalog.json"),
    ),
    (
        "traceability-matrix",
        include_str!("../profiles/traceability-matrix.json"),
    ),
    (
        "release-specification",
        include_str!("../profiles/release-specification.json"),
    ),
    (
        "implementation-context-pack",
        include_str!("../profiles/implementation-context-pack.json"),
    ),
    ("mcp-resource", include_str!("../profiles/mcp-resource.json")),
    (
        "github-markdown",
        include_str!("../profiles/github-markdown.json"),
    ),
    ("html-site", include_str!("../profiles/html-site.json")),
    (
        "contract-yaml",
        include_str!("../profiles/contract-yaml.json"),
    ),
    ("diagram-set", include_str!("../profiles/diagram-set.json")),
    (
        "offline-bundle",
        include_str!("../profiles/offline-bundle.json"),
    ),
];

pub struct Catalogue
{
    profiles: Vec<Profile>,
}

impl Catalogue
{
    pub fn Shipped() -> Result<Self, ProjectError>
    {
        let mut profiles = Vec::new();
        for (name, text) in SHIPPED
        {
            let profile = Profile::Parse(text)?;
            if profile.id != *name
            {
                return Err(ProjectError::Malformed(format!(
                    "{name}.json declares the identifier {}, so a profile named on the command \
                     line would resolve to a file that says it is something else",
                    profile.id
                )));
            }
            profiles.push(profile);
        }

        return Self::Of(profiles);
    }

    pub fn Of(profiles: Vec<Profile>) -> Result<Self, ProjectError>
    {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut outputs: BTreeMap<&str, &str> = BTreeMap::new();

        for profile in &profiles
        {
            if !seen.insert(profile.id.as_str())
            {
                return Err(ProjectError::Duplicate {
                    profile: profile.id.clone(),
                });
            }
            if let Some(first) = outputs.insert(profile.output.as_str(), profile.id.as_str())
            {
                return Err(ProjectError::Colliding {
                    output: profile.output.clone(),
                    first: first.to_owned(),
                    second: profile.id.clone(),
                });
            }
        }

        return Ok(Self { profiles });
    }

    #[must_use]
    pub fn Profiles(&self) -> &[Profile]
    {
        return &self.profiles;
    }

    #[must_use]
    pub fn Named(&self, id: &str) -> Option<&Profile>
    {
        return self.profiles.iter().find(|profile| return profile.id == id);
    }
}

pub fn Shipped() -> Result<Catalogue, ProjectError>
{
    return Catalogue::Shipped();
}
