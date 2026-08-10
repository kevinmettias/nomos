//! What became of trying to read one file.

use crate::parse_failure::ParseFailure;
use crate::syntax_facts::SyntaxFacts;
/// The result of reading one recognized file.
///
/// Two variants and no third. There is deliberately no `Reading::Empty` and no
/// `impl Default`: a caller that wants to know whether a file declared nothing must ask
/// [`SyntaxFacts::Declares_Nothing`], which is only reachable through
/// [`Reading::Parsed`] — so "the file has no items" is a sentence that can only be said
/// about a file that was successfully read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    Parsed(SyntaxFacts),
    Unparseable(ParseFailure),
}
