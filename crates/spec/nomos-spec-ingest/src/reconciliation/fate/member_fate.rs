//! What became of one member of a family.

use crate::Fate;
use crate::Restored;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberFate
{
    pub id: String,
    pub family: Restored,
    pub name: String,
    pub was: String,
    pub fate: Fate,
}
