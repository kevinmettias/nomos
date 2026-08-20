//! Reading a v14 artifact: its declared identity, statement, and front matter.

use crate::archive::artifact::ArtifactFrontMatter;
use crate::Artifact;
use crate::Family;
use crate::IngestError;

/// Reads a v14 artifact's declared identity and statement.
///
/// A lighter reader than [`Parse_Record`](nomos_spec_model::Parse_Record) on purpose: an
/// artifact's heading is `# ID - title`, which is not its title, so the record reader
/// refuses it for naming itself twice. That check is right for records and wrong here.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if the front matter is absent or unreadable.
pub fn Parse_Artifact(markdown: &str, family: Family) -> Result<Artifact, IngestError>
{
    let yaml = Front_Matter(markdown)
        .ok_or_else(|| return IngestError::Parse("artifact has no front matter".to_owned()))?;

    let front: ArtifactFrontMatter = serde_yaml_ng::from_str(yaml)
        .map_err(|error| IngestError::Parse(format!("artifact front matter: {error}")))?;
    let statement = Statement_Of(&front)?;

    return Ok(Artifact {
        id: front.id,
        family,
        criteria: front.criteria.len(),
        statement,
    });
}

/// What an artifact says, from its statement or from the criteria standing in for one.
///
/// v14 let an artifact carry criteria and no statement, and the criteria are what it says
/// in that case. An artifact carrying neither is refused rather than reconciled as empty,
/// because empty text matches empty text and would report itself preserved.
fn Statement_Of(front: &ArtifactFrontMatter) -> Result<String, IngestError>
{
    let mut statement = front.statement.clone();
    if statement.trim().is_empty()
    {
        statement = Criteria_As_Statement(front);
    }

    if statement.trim().is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{} declares neither a statement nor any criteria, so there is nothing to reconcile",
            front.id
        )));
    }

    return Ok(statement);
}

/// The criteria run together, for an artifact that carries them instead of a statement.
fn Criteria_As_Statement(front: &ArtifactFrontMatter) -> String
{
    return front
        .criteria
        .iter()
        .map(|criterion| return format!("{} {}", criterion.id, criterion.statement))
        .collect::<Vec<String>>()
        .join("\n");
}

/// The front matter body, tolerating a byte order mark exactly as v14's readers did.
///
/// See D-131. The mark belongs to the opening fence and leaves with it.
fn Front_Matter(markdown: &str) -> Option<&str>
{
    let text = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let after = text.strip_prefix("---\n").or_else(|| return text.strip_prefix("---\r\n"))?;

    let mut offset = 0_usize;
    for line in after.split_inclusive('\n')
    {
        if line.trim_end() == "---"
        {
            return after.get(..offset);
        }
        offset = offset.checked_add(line.len())?;
    }

    return None;
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ARTIFACT: &str = "\u{feff}---\nid: MODEL-001\nkind: requirement\n\
                            statement: MODEL-001 Artifact represents persisted objects.\n\
                            ---\n\n# MODEL-001 - Artifact\n";

    #[test]
    fn Test_An_Artifact_Should_Read_Through_Its_Byte_Order_Mark()
    {
        let artifact = Parse_Artifact(ARTIFACT, Family::Requirement).expect("reads");

        assert_eq!(artifact.id, "MODEL-001");
        assert_eq!(artifact.statement, "MODEL-001 Artifact represents persisted objects.");
    }

    #[test]
    fn Test_An_Artifact_With_No_Statement_Should_Be_Refused()
    {
        let empty = ARTIFACT.replace("statement: MODEL-001 Artifact represents persisted objects.\n", "");

        assert!(Parse_Artifact(&empty, Family::Requirement).is_err());
    }

    /// One acceptance artifact declaring two criteria.
    const ACCEPTANCE: &str = concat!(
        "---
",
        "id: US-A-001-AC
",
        "kind: acceptance_criteria
",
        "criteria:
",
        "- id: US-A-001-AC-01
",
        "  statement: One holds
",
        "- id: US-A-001-AC-02
",
        "  statement: Two holds
",
        "---

# X
"
    );

    /// Acceptance artifacts declare a list, not a statement, and joining it in declared
    /// order is what makes a silently dropped criterion change the hash.
    #[test]
    fn Test_An_Acceptance_Artifact_Should_Reconcile_Through_Its_Criteria()
    {
        let artifact = Parse_Artifact(ACCEPTANCE, Family::Acceptance).expect("reads");
        let dropped = ACCEPTANCE.replace("- id: US-A-001-AC-02
  statement: Two holds
", "");
        let shorter = Parse_Artifact(&dropped, Family::Acceptance).expect("reads");

        assert_eq!(artifact.criteria, 2);
        assert!(artifact.statement.contains("US-A-001-AC-01 One holds"));
        assert!(artifact.statement.contains("US-A-001-AC-02 Two holds"));
        assert_ne!(
            shorter.statement, artifact.statement,
            "dropping a criterion did not change the text reconciliation hashes"
        );
    }
}
