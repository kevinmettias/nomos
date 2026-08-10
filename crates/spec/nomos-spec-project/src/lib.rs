#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

mod build;
mod catalogue;
mod determinism;
mod profile;
mod projection;
mod render;
mod select;

pub use build::{Build, Check, Freshness, Output, Stamp, SIDECAR_SUFFIX};
pub use catalogue::{Catalogue, Shipped, SHIPPED};
pub use determinism::ProjectionOutput;
pub use profile::{Content, Filter, Format, Profile, Section as ProfileSection, SUBJECT};
pub use projection::{Input, Item, Projection, Section, DO_NOT_EDIT};
pub use render::Render;
pub use select::Select;

#[derive(Debug)]
pub enum ProjectError
{
    Store(nomos_spec_store::StoreError),
    Sql(String),
    Malformed(String),
    NotRelative
    {
        profile: String,
        output: String,
    },
    Empty
    {
        profile: String,
        section: String,
        content: &'static str,
    },
    UnsupportedFilter
    {
        profile: String,
        content: &'static str,
        filter: &'static str,
    },
    Duplicate
    {
        profile: String,
    },
    Colliding
    {
        output: String,
        first: String,
        second: String,
    },
    /// A profile that projects one subject, run without being told which.
    SubjectMissing
    {
        profile: String,
    },
    /// A subject given to a profile that projects the whole store.
    SubjectUnexpected
    {
        profile: String,
        subject: String,
    },
    /// A profile that reached the renderer with its placeholder still in it.
    SubjectUnresolved
    {
        profile: String,
        output: String,
    },
}

impl core::fmt::Display for ProjectError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Sql(cause) => write!(formatter, "projection sql error: {cause}"),
            Self::Malformed(cause) => write!(formatter, "malformed profile: {cause}"),
            Self::NotRelative { profile, output } => write!(
                formatter,
                "{profile} writes to {output}. A projection output is a relative path inside \
                 the build root: an absolute path makes the output depend on the machine that \
                 built it"
            ),
            Self::Empty {
                profile,
                section,
                content,
            } => write!(
                formatter,
                "{profile}: section \"{section}\" selected no {content}. Refusing to render a \
                 section that says nothing, because a projection over an empty selection reads \
                 exactly like a projection over an empty store. Declare `may_be_empty` if \
                 nothing is the honest answer"
            ),
            Self::UnsupportedFilter {
                profile,
                content,
                filter,
            } => write!(
                formatter,
                "{profile}: a {content} section filters on `{filter}`, which {content} does not \
                 honour. Refusing to ignore it: a filter that narrows nothing renders the whole \
                 table under a heading that claims otherwise"
            ),
            Self::Duplicate { profile } => {
                write!(formatter, "{profile} is declared more than once")
            }
            Self::Colliding {
                output,
                first,
                second,
            } => write!(
                formatter,
                "{first} and {second} both write {output}, so one would overwrite the other and \
                 the build would depend on profile order"
            ),
            Self::SubjectMissing { profile } => write!(
                formatter,
                "{profile} projects one subject and was not told which. It writes a path \
                 holding {SUBJECT} and selects on it, so without a subject there is no output \
                 to write and no rows to write into it — name one with --subject"
            ),
            Self::SubjectUnexpected { profile, subject } => write!(
                formatter,
                "{profile} projects the whole store, so it has nowhere to put the subject \
                 {subject}. Its output path is fixed and none of its filters names a subject: \
                 running it per subject would write the same file every time"
            ),
            Self::SubjectUnresolved { profile, output } => write!(
                formatter,
                "{profile} reached the renderer still writing to {output}. A path holding \
                 {SUBJECT} is a template rather than a destination, and rendering it would \
                 create a directory named after the placeholder"
            ),
        };
    }
}

impl std::error::Error for ProjectError {}

impl From<nomos_spec_store::StoreError> for ProjectError
{
    fn from(error: nomos_spec_store::StoreError) -> Self
    {
        return Self::Store(error);
    }
}

impl From<rusqlite::Error> for ProjectError
{
    fn from(error: rusqlite::Error) -> Self
    {
        return Self::Sql(error.to_string());
    }
}
