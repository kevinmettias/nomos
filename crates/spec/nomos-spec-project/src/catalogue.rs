use crate::Profile;
use crate::ProjectError;
use std::collections::{BTreeMap, BTreeSet};

/// Every profile this build ships, as a name and the file's contents at build time.
///
/// Mirrored by `Test_Every_Profile_File_Should_Be_Shipped`, which walks `profiles/` on disk
/// and compares its `.json` stems against these names in both directions: a file with no
/// entry here is a profile `nomos spec profiles` cannot see, and an entry with no file is a
/// name this list carries for nothing.
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Catalogue_Shipped_Should_Parse_Every_Bundled_Profile()
    {
        let catalogue = Catalogue::Shipped().expect("the shipped profiles parse");

        assert!(!catalogue.Profiles().is_empty(), "no profile shipped at all");
    }

    #[test]
    fn Test_Of_Should_Refuse_Two_Entries_Sharing_One_Identifier()
    {
        let mut entries = Two_Entries();
        let first_id = entries.first().expect("Two_Entries builds at least one entry").id.clone();
        entries.get_mut(1).expect("Two_Entries builds at least two entries").id = first_id;

        let refusal = match Catalogue::Of(entries)
        {
            // Test assertion, not a runtime escape hatch: an unrefused duplicate here is a
            // failing test, and panicking is how the test reports it.
            Ok(_) => panic!("must refuse"),
            Err(error) => error,
        };

        assert!(matches!(refusal, ProjectError::Duplicate { .. }), "{refusal:?}");
    }

    #[test]
    fn Test_Profiles_Should_Return_Every_Profile_It_Was_Built_From()
    {
        let entries = Two_Entries();
        let catalogue = Catalogue::Of(entries.clone()).expect("distinct");

        assert_eq!(catalogue.Profiles(), entries.as_slice());
    }

    #[test]
    fn Test_Named_Should_Find_A_Profile_By_Its_Identifier()
    {
        let catalogue = Catalogue::Of(Two_Entries()).expect("distinct");

        assert_eq!(catalogue.Named("two").map(|profile| return profile.id.as_str()), Some("two"));
        assert!(catalogue.Named("missing").is_none());
    }

    fn Two_Entries() -> Vec<Profile>
    {
        let one = Profile::Parse(
            r#"{
                "id": "one", "title": "One", "format": "markdown", "output": "one.md",
                "sections": [{ "title": "Nodes", "content": "nodes" }]
            }"#,
        )
        .expect("parses");
        let two = Profile::Parse(
            r#"{
                "id": "two", "title": "Two", "format": "markdown", "output": "two.md",
                "sections": [{ "title": "Nodes", "content": "nodes" }]
            }"#,
        )
        .expect("parses");

        return vec![one, two];
    }
}
