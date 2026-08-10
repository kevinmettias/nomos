//! What became of one declared identifier.

use crate::disposition::Disposition;
use crate::family::Family;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentifierOutcome
{
    pub id: String,
    pub family: Family,
    pub disposition: Disposition,
}
