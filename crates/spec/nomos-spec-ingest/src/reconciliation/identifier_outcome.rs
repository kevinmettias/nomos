//! What became of one declared identifier.

use crate::Disposition;
use crate::Family;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentifierOutcome
{
    pub id: String,
    pub family: Family,
    pub disposition: Disposition,
}
