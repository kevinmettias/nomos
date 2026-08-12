#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

mod build;
mod catalogue;
mod determinism;
mod filter;
mod format;
mod profile;
mod projection;
mod render;
mod section;
mod select;

pub use build::{Build, Check, Freshness, SIDECAR_SUFFIX, Stamp};
pub use catalogue::{Catalogue, SHIPPED, Shipped};
pub use determinism::ProjectionOutput;
pub use filter::Filter;
pub use format::Format;
pub use profile::{Profile, SUBJECT};
pub use profile::Section as ProfileSection;
pub use projection::{Content, DO_NOT_EDIT, Input, Item, Output, Projection};
pub use section::Section;
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
            Self::Duplicate { profile } =>
            {
                write!(formatter, "{profile} is declared more than once")
            }
            Self::NotRelative { profile, output } => Not_Relative(formatter, profile, output),
            Self::Empty {
                profile,
                section,
                content,
            } => Empty_Section(formatter, profile, section, content),
            Self::UnsupportedFilter {
                profile,
                content,
                filter,
            } => Unsupported_Filter(formatter, profile, content, filter),
            Self::Colliding {
                output,
                first,
                second,
            } => Colliding(formatter, output, first, second),
            Self::SubjectMissing { profile } => Subject_Missing(formatter, profile),
            Self::SubjectUnexpected { profile, subject } =>
            {
                Subject_Unexpected(formatter, profile, subject)
            }
            Self::SubjectUnresolved { profile, output } =>
            {
                Subject_Unresolved(formatter, profile, output)
            }
        };
    }
}

/// An output path that would leave the build root.
fn Not_Relative(
    formatter: &mut core::fmt::Formatter<'_>,
    profile: &str,
    output: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{profile} writes to {output}. A projection output is a relative path inside the \
         build root: an absolute path makes the output depend on the machine that built it"
    );
}

/// A section that selected nothing and did not declare that it might.
fn Empty_Section(
    formatter: &mut core::fmt::Formatter<'_>,
    profile: &str,
    section: &str,
    content: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{profile}: section \"{section}\" selected no {content}. Refusing to render a section \
         that says nothing, because a projection over an empty selection reads exactly like a \
         projection over an empty store. Declare `may_be_empty` if nothing is the honest answer"
    );
}

/// A filter the content it narrows does not honour.
fn Unsupported_Filter(
    formatter: &mut core::fmt::Formatter<'_>,
    profile: &str,
    content: &str,
    filter: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{profile}: a {content} section filters on `{filter}`, which {content} does not \
         honour. Refusing to ignore it: a filter that narrows nothing renders the whole \
         table under a heading that claims otherwise"
    );
}

/// Two profiles writing one path.
fn Colliding(
    formatter: &mut core::fmt::Formatter<'_>,
    output: &str,
    first: &str,
    second: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{first} and {second} both write {output}, so one would overwrite the other and the \
         build would depend on profile order"
    );
}

/// A per-subject profile run without a subject.
fn Subject_Missing(formatter: &mut core::fmt::Formatter<'_>, profile: &str) -> core::fmt::Result
{
    return write!(
        formatter,
        "{profile} projects one subject and was not told which. It writes a path holding \
         {SUBJECT} and selects on it, so without a subject there is no output to write and no \
         rows to write into it — name one with --subject"
    );
}

/// A whole-store profile handed a subject.
fn Subject_Unexpected(
    formatter: &mut core::fmt::Formatter<'_>,
    profile: &str,
    subject: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{profile} projects the whole store, so it has nowhere to put the subject {subject}. \
         Its output path is fixed and none of its filters names a subject: running it per \
         subject would write the same file every time"
    );
}

/// A template that reached the renderer still holding its placeholder.
fn Subject_Unresolved(
    formatter: &mut core::fmt::Formatter<'_>,
    profile: &str,
    output: &str,
) -> core::fmt::Result
{
    return write!(
        formatter,
        "{profile} reached the renderer still writing to {output}. A path holding {SUBJECT} \
         is a template rather than a destination, and rendering it would create a directory \
         named after the placeholder"
    );
}

impl std::error::Error for ProjectError
{}

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
