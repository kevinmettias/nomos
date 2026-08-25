//! What became of trying to read one file.

use crate::ParseFailure;
use crate::SyntaxFacts;

/// The result of reading one recognized file.
///
/// Two variants and no third — see `nomos-lang-rust`'s own [`Reading`] for why: there is
/// deliberately no `Reading::Empty`, so "the file has no items" is a sentence that can only
/// be said about a file that was successfully read, through [`SyntaxFacts::Declares_Nothing`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    Parsed(SyntaxFacts),
    Unparseable(ParseFailure),
}
