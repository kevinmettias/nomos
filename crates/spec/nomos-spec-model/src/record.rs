//! An authored record: its declared identity and its body.

// A record's front matter, the relations it declares, and the refusals parsing one raises.
pub(crate) mod error;
pub(crate) mod front_matter;
mod relation;

use error::Error as RecordError;
use front_matter::FrontMatter as RecordFrontMatter;
pub use relation::Relation as RecordRelation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record
{
    pub front_matter: RecordFrontMatter,
    /// Everything after the front matter, including the first heading.
    pub body: String,
}

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

    Assert_The_Heading_Corroborates(&front_matter, body)?;

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

/// A record that names itself two different things is refused rather than reconciled.
fn Assert_The_Heading_Corroborates(
    front_matter: &RecordFrontMatter,
    body: &str,
) -> Result<(), RecordError>
{
    let Some(heading) = First_Heading(body)
    else
    {
        return Err(RecordError::NoHeading {
            id: front_matter.id.clone(),
        });
    };
    if Is_Heading_Corroborated(Heading(&heading), Id(&front_matter.id), Title(&front_matter.title))
    {
        return Ok(());
    }

    return Err(RecordError::TitleDiverges {
        id: front_matter.id.clone(),
        declared: front_matter.title.clone(),
        heading,
    });
}

fn First_Heading(body: &str) -> Option<String>
{
    return body
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line.trim_start_matches("# ").trim().to_owned());
}

/// Whether a heading says the same thing the front matter does.
///
/// Two spellings are accepted: the title alone, and the record's own identifier followed
/// by a separator and the title. The v15.0 archives write the second — `# ARC-FACT-001 —
/// Shared analysis` — and this repository's records write the first.
///
/// This is still corroboration, not tolerance. Only the record's *declared* identifier is
/// allowed as the prefix, so a heading naming a different concept still diverges, which
/// is what the refusal exists for. Widening it here rather than adding a second reader is
/// deliberate: two readers for one format is how the two come to disagree.
/// The heading a document opens with, kept distinct from [`Id`] and [`Title`] so the three
/// positions of [`Is_Heading_Corroborated`] cannot be swapped at its call site: all three are
/// plain strings and nothing else would tell them apart.
struct Heading<'a>(&'a str);

/// A record's declared identifier, kept distinct from [`Heading`] and [`Title`].
struct Id<'a>(&'a str);

/// A record's declared title, kept distinct from [`Heading`] and [`Id`].
struct Title<'a>(&'a str);

fn Is_Heading_Corroborated(heading: Heading<'_>, id: Id<'_>, title: Title<'_>) -> bool
{
    let heading = heading.0;
    let id = id.0;
    let title = title.0;

    if heading == title
    {
        return true;
    }

    let Some(rest) = heading.strip_prefix(id)
    else
    {
        return false;
    };
    let separated = rest.trim_start_matches([' ', '-', '\u{2013}', '\u{2014}', ':']);

    return !separated.eq(rest) && separated == title;
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
    fn Test_Parse_Record_Should_Read_Its_Declared_Identity()
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

    /// v15's `spec-governance` records spell the relation key `relation`, not `type`.
    #[test]
    fn Test_Either_Spelling_Of_The_Relation_Key_Should_Read()
    {
        let spelled_relation = RECORD.replace("    type: supersedes", "    relation: supersedes");
        assert_ne!(spelled_relation, RECORD, "the negative control changed nothing");

        assert_eq!(
            Parse_Record(&spelled_relation).expect("reads").front_matter.relations,
            Parse_Record(RECORD).expect("reads").front_matter.relations
        );
    }

    /// The v15.0 archives prefix a record's heading with its own identifier.
    #[test]
    fn Test_A_Heading_May_Name_The_Record_Before_Its_Title()
    {
        for separator in Heading_Prefix_Separators()
        {
            let prefixed = RECORD.replace("# A title", &format!("# D-129{separator}A title"));
            assert_ne!(prefixed, RECORD, "the {separator:?} case changed nothing");

            let record =
                // Unreachable while a heading may name the record before its title, for all
                // four separators the v15.0 archives use. It is a panic rather than `expect`
                // because the loop covers four spellings and only the separator says which
                // one stopped being accepted.
                Parse_Record(&prefixed).unwrap_or_else(|error| panic!("{separator:?}: {error}"));
            assert_eq!(record.front_matter.title, "A title");
        }
    }

    /// The separators the v15.0 archives prefix a record's heading with its own identifier
    /// using.
    fn Heading_Prefix_Separators() -> [&'static str; 4]
    {
        return [" \u{2014} ", " - ", ": ", " \u{2013} "];
    }

    #[test]
    fn Test_A_Heading_Naming_A_Different_Identity_Should_Not_Corroborate()
    {
        for case in Headings_That_Should_Not_Corroborate()
        {
            let wrong = RECORD.replace("# A title", case.replacement);
            assert_ne!(wrong, RECORD, "{}: the negative control changed nothing", case.description);

            let refusal = Parse_Record(&wrong)
                .expect_err(&format!("{} must be refused", case.description));

            assert!(matches!(refusal, RecordError::TitleDiverges { .. }), "{}: {refusal}", case.description);
        }
    }

    /// One heading that diverges from the front matter, and why it must be refused.
    struct HeadingDivergenceCase
    {
        /// What `# A title` becomes.
        replacement: &'static str,
        /// Why this is not corroboration.
        description: &'static str,
    }

    /// Widening the heading check must not let a different concept through, whether the
    /// heading names a foreign identifier or the record's own identifier with a different
    /// title.
    fn Headings_That_Should_Not_Corroborate() -> [HeadingDivergenceCase; 2]
    {
        return [
            HeadingDivergenceCase {
                replacement: "# D-130 \u{2014} A title",
                description: "a foreign identifier prefixing the declared title",
            },
            HeadingDivergenceCase {
                replacement: "# D-129 \u{2014} Another title",
                description: "the record's own identifier prefixing a different title",
            },
        ];
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
