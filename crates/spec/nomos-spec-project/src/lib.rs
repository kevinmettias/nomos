#![forbid(unsafe_code)]
// Every fallible function in this crate returns `ProjectError` and nothing else. Each of its
// variants already carries the failure and the remedy — in a doc comment above the variant and
// in the `Display` arm below, which are what a caller actually reads. An `# Errors` section per
// function would be a second, unchecked copy of that one enumeration, so the lint is declined
// for the whole crate rather than answered file by file.
#![allow(clippy::missing_errors_doc)]

mod build;
mod catalogue;
mod determinism;
mod profile;
mod projection;
mod render;
mod section;
mod select;

pub use build::{Build, Check, Freshness, SIDECAR_SUFFIX, Stamp};
pub use catalogue::{Catalogue, SHIPPED, Shipped};
pub use determinism::ProjectionOutput;
pub use profile::{Profile, SUBJECT};
// Named for the boundary rather than for its module, because two different types here are
// a Section and both are exported: `section::Section` is a section of a rendered projection,
// and this one is a section as a profile declares it. Aliasing at the facade would have made
// the exported name a thing no declaration says, so the declaration says it.
pub use profile::ProfileSection;
pub use projection::{Content, DO_NOT_EDIT, Filter, Format, Input, Item, Name, Output, Projection, Value};
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

/// A profile's own identifier, kept distinct from the other plain-string identifiers below
/// (an [`OutputPath`] above all) so adjacent parameters that mean different things cannot be
/// swapped at a call site without a type error.
pub(crate) struct ProfileName<'a>(pub(crate) &'a str);

/// A projection's output path, kept distinct from [`ProfileName`] for the same reason.
pub(crate) struct OutputPath<'a>(pub(crate) &'a str);

/// The section a refusal is about, kept distinct from [`ProfileName`] and [`ContentName`].
struct SectionName<'a>(&'a str);

/// The kind of content a section selects, kept distinct from [`ProfileName`] and
/// [`SectionName`] (or, in [`Unsupported_Filter`], from [`FilterName`]).
struct ContentName<'a>(&'a str);

/// A filter name a section declared, kept distinct from [`ContentName`].
struct FilterName<'a>(&'a str);

/// The subject a run was, or was not, given.
struct SubjectName<'a>(&'a str);

/// The profile that already claims a colliding output.
struct FirstProfile<'a>(&'a str);

/// The profile that would also write it.
struct SecondProfile<'a>(&'a str);

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
            Self::NotRelative { profile, output } =>
                Not_Relative(formatter, ProfileName(profile), OutputPath(output)),
            Self::Empty {
                profile,
                section,
                content,
            } => Empty_Section(formatter, ProfileName(profile), SectionName(section), ContentName(content)),
            Self::UnsupportedFilter {
                profile,
                content,
                filter,
            } => Unsupported_Filter(formatter, ProfileName(profile), ContentName(content), FilterName(filter)),
            Self::Colliding {
                output,
                first,
                second,
            } => Colliding(formatter, OutputPath(output), FirstProfile(first), SecondProfile(second)),
            Self::SubjectMissing { profile } => Subject_Missing(formatter, profile),
            Self::SubjectUnexpected { profile, subject } =>
            {
                Subject_Unexpected(formatter, ProfileName(profile), SubjectName(subject))
            }
            Self::SubjectUnresolved { profile, output } =>
            {
                Subject_Unresolved(formatter, ProfileName(profile), OutputPath(output))
            }
        };
    }
}

/// An output path that would leave the build root.
fn Not_Relative(
    formatter: &mut core::fmt::Formatter<'_>,
    profile: ProfileName<'_>,
    output: OutputPath<'_>,
) -> core::fmt::Result
{
    let profile = profile.0;
    let output = output.0;

    return write!(
        formatter,
        "{profile} writes to {output}. A projection output is a relative path inside the \
         build root: an absolute path makes the output depend on the machine that built it"
    );
}

/// A section that selected nothing and did not declare that it might.
fn Empty_Section(
    formatter: &mut core::fmt::Formatter<'_>,
    profile: ProfileName<'_>,
    section: SectionName<'_>,
    content: ContentName<'_>,
) -> core::fmt::Result
{
    let profile = profile.0;
    let section = section.0;
    let content = content.0;

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
    profile: ProfileName<'_>,
    content: ContentName<'_>,
    filter: FilterName<'_>,
) -> core::fmt::Result
{
    let profile = profile.0;
    let content = content.0;
    let filter = filter.0;

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
    output: OutputPath<'_>,
    first: FirstProfile<'_>,
    second: SecondProfile<'_>,
) -> core::fmt::Result
{
    let output = output.0;
    let first = first.0;
    let second = second.0;

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
    profile: ProfileName<'_>,
    subject: SubjectName<'_>,
) -> core::fmt::Result
{
    let profile = profile.0;
    let subject = subject.0;

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
    profile: ProfileName<'_>,
    output: OutputPath<'_>,
) -> core::fmt::Result
{
    let profile = profile.0;
    let output = output.0;

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
