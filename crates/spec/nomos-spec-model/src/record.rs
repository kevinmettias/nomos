use serde::Deserialize;

/// A typed edge declared in a record's front matter.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RecordRelation
{
    pub target: String,
    #[serde(rename = "type")]
    pub relation: String,
}

/// The declared identity of a record.
///
/// The field names are the v14 corpus's, so the same reader serves this repository's own
/// records and the records restored from the archives. Two readers for one format is how
/// the two disagree.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RecordFrontMatter
{
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: String,
    pub authority: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub relations: Vec<RecordRelation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record
{
    pub front_matter: RecordFrontMatter,
    /// Everything after the front matter, including the first heading.
    pub body: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordError
{
    NoFrontMatter,
    UnterminatedFrontMatter,
    Yaml(String),
    NoHeading
    {
        id: String,
    },
    /// The front matter and the first heading name the record differently.
    ///
    /// Refused rather than reconciled: two titles for one record is two homes for one
    /// concept, and picking a winner here would decide silently which one people read.
    TitleDiverges
    {
        id: String,
        declared: String,
        heading: String,
    },
}

impl core::fmt::Display for RecordError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::NoFrontMatter => write!(
                formatter,
                "a record begins with a `---` front matter block; this one does not"
            ),
            Self::UnterminatedFrontMatter => {
                write!(formatter, "the front matter block is never closed")
            }
            Self::Yaml(cause) => write!(formatter, "front matter is not readable: {cause}"),
            Self::NoHeading { id } => {
                write!(formatter, "{id} has no heading, so its title cannot be corroborated")
            }
            Self::TitleDiverges {
                id,
                declared,
                heading,
            } => write!(
                formatter,
                "{id} declares the title {declared:?} and its heading says {heading:?}"
            ),
        };
    }
}

impl std::error::Error for RecordError {}

/// Reads a record and refuses one that names itself two different things.
///
/// # Errors
///
/// Returns [`RecordError`] naming what was wrong with the document.
pub fn Parse_Record(markdown: &str) -> Result<Record, RecordError>
{
    let text = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);

    let Some(after_opening) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n"))
    else
    {
        return Err(RecordError::NoFrontMatter);
    };

    let Some((yaml, body)) = Split_At_Closing_Fence(after_opening)
    else
    {
        return Err(RecordError::UnterminatedFrontMatter);
    };

    let front_matter: RecordFrontMatter =
        serde_yaml_ng::from_str(yaml).map_err(|error| RecordError::Yaml(error.to_string()))?;

    let Some(heading) = First_Heading(body)
    else
    {
        return Err(RecordError::NoHeading {
            id: front_matter.id,
        });
    };

    if heading != front_matter.title
    {
        return Err(RecordError::TitleDiverges {
            id: front_matter.id,
            declared: front_matter.title,
            heading,
        });
    }

    return Ok(Record {
        front_matter,
        body: body.to_owned(),
    });
}

fn Split_At_Closing_Fence(text: &str) -> Option<(&str, &str)>
{
    let mut offset = 0_usize;

    for line in text.split_inclusive('\n')
    {
        if line.trim_end() == "---"
        {
            let yaml = text.get(..offset)?;
            let body = text.get(offset.checked_add(line.len())?..)?;
            return Some((yaml, body));
        }
        offset = offset.checked_add(line.len())?;
    }

    return None;
}

fn First_Heading(body: &str) -> Option<String>
{
    return body
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line.trim_start_matches("# ").trim().to_owned());
}

#[cfg(test)]
mod tests
{
    use super::*;

    const RECORD: &str = "---\nid: D-129\ntype: decision\ntitle: A title\nstatus: accepted\n\
                          version: 1\nauthority: canonical-normative-record\ntags:\n  - one\n\
                          relations:\n  - target: ADR-DOC-001\n    type: supersedes\n---\n\n\
                          # A title\n\nBody.\n";

    #[test]
    fn Test_A_Record_Should_Read_Its_Declared_Identity()
    {
        let record = Parse_Record(RECORD).expect("reads");

        assert_eq!(record.front_matter.id, "D-129");
        assert_eq!(record.front_matter.kind, "decision");
        assert_eq!(record.front_matter.status, "accepted");
        assert_eq!(record.front_matter.tags, vec!["one".to_owned()]);
        assert_eq!(record.front_matter.relations.len(), 1);
        assert_eq!(record.front_matter.relations.first().map(|r| r.relation.as_str()), Some("supersedes"));
        assert!(record.body.starts_with("\n# A title"));
    }

    /// The v14 records carry one, and a reader that chokes on it reads no record at all.
    #[test]
    fn Test_A_Byte_Order_Mark_Should_Not_Hide_The_Front_Matter()
    {
        let marked = format!("\u{feff}{RECORD}");

        assert_eq!(Parse_Record(&marked).expect("reads"), Parse_Record(RECORD).expect("reads"));
    }

    #[test]
    fn Test_A_Record_That_Names_Itself_Twice_Should_Be_Refused()
    {
        let divergent = RECORD.replace("# A title", "# Another title");
        assert_ne!(divergent, RECORD, "the negative control changed nothing");

        let refusal = Parse_Record(&divergent).expect_err("two titles must be refused");

        assert!(matches!(refusal, RecordError::TitleDiverges { .. }), "{refusal}");
    }

    #[test]
    fn Test_A_Document_Without_Front_Matter_Should_Be_Refused()
    {
        assert_eq!(Parse_Record("# Just a heading\n"), Err(RecordError::NoFrontMatter));
    }

    #[test]
    fn Test_Unterminated_Front_Matter_Should_Be_Refused()
    {
        assert_eq!(
            Parse_Record("---\nid: D-1\n"),
            Err(RecordError::UnterminatedFrontMatter)
        );
    }

    #[test]
    fn Test_Missing_Required_Identity_Should_Be_Refused()
    {
        let without_authority = RECORD.replace("authority: canonical-normative-record\n", "");

        let refusal = Parse_Record(&without_authority).expect_err("authority is not optional");

        assert!(matches!(refusal, RecordError::Yaml(_)), "{refusal}");
    }
}
