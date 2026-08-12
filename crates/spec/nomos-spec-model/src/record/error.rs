//! Every way a record refuses to be read.

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

impl std::error::Error for RecordError
{}
