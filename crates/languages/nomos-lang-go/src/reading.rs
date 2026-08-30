//! What became of trying to read one file.

use crate::ParseFailure;
use crate::Facts;

/// The result of reading one recognized file.
///
/// Two variants and no third — see `nomos-lang-rust`'s own [`Reading`] for why: there is
/// deliberately no `Reading::Empty`, so "the file has no items" is a sentence that can only
/// be said about a file that was successfully read, through [`Facts::Has_No_Declarations`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    Parsed(Facts),
    Unparseable(ParseFailure),
}
