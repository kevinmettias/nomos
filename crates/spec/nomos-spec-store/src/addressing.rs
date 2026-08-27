//! The two positions that recur at every boundary that writes or looks up a source
//! document: where it lives, and which revision of it was read.
//!
//! Named rather than left as two adjacent `&str` parameters. `Put_Record(revision, path,
//! markdown)` used to type-check — a position is not a name, and the compiler could not
//! tell the caller apart from the one who meant it the other way round. One pair, defined
//! once and reused everywhere the concept repeats, rather than a fresh wrapper per call
//! site: `nomos-spec-ingest` depends on this crate and reuses the same two types at its own
//! ingest boundary instead of inventing its own.
//!
//! Each type accepts both `&str` and `&String` at its call sites, so a caller passing an
//! owned field (`&record.path`) or a borrowed literal converts the same way a plain `&str`
//! parameter always did — the newtype costs the callers already writing `&str` nothing.

mod document_revision;

pub use document_revision::DocumentRevision;

/// Where a document lives, as a repository-relative path.
#[derive(Clone, Copy, Debug)]
pub struct DocumentPath<'a>(pub &'a str);

impl<'a> From<&'a str> for DocumentPath<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for DocumentPath<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
