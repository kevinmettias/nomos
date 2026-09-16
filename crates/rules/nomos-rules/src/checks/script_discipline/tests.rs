//! The `SourceFile` fixture every test module under `script_discipline` builds its input from,
//! in one place because a fixture restated per test module is a fixture that drifts.

use crate::SourceFile;
use nomos_contracts::SubjectId;
use nomos_model::Content_Digest;

/// A fixture source's two halves, grouped so a call site names which string is the path
/// and which is the text, rather than counting two adjacent `&str` positions a caller
/// could transpose without the compiler objecting.
pub(super) struct SourceText<'text>
{
    pub(super) path: &'text str,
    pub(super) text: &'text str,
}

/// A source file at the fixture's `path` carrying the fixture's `text`, both taken from the
/// [`SourceText`] a call site writes so the two `&str` positions cannot be transposed.
pub(super) fn Source(source: SourceText<'_>) -> SourceFile
{
    let SourceText { path, text } = source;
    return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
}
