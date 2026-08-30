//! What became of trying to read one file.

use crate::ParseFailure;
use crate::Facts;
/// The result of reading one recognized file.
///
/// Two variants and no third. There is deliberately no `Reading::Empty` and no
/// `impl Default`: a caller that wants to know whether a file declared nothing must ask
/// [`Facts::Has_No_Declarations`], which is only reachable through
/// [`Reading::Parsed`] — so "the file has no items" is a sentence that can only be said
/// about a file that was successfully read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    Parsed(Facts),
    Unparseable(ParseFailure),
}
