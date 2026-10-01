//! One entry's own identity and text, before anything has been read out of it.

/// One entry before it has been read: the requirement its file is named for, and the text
/// that file holds.
///
/// Named rather than passed as two adjacent `&str`. A call site that read `Parse_Assessment(text, stem)`
/// would compile, and the transposition would be caught only because [`Is_Requirement_Id`]
/// refuses a file body -- a run-time refusal where a named field lets the compiler make one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntrySource<'a>
{
    /// The requirement identifier the entry's own file is named for.
    pub stem: &'a str,
    /// The entry's own text, exactly as its file holds it.
    pub text: &'a str,
}
