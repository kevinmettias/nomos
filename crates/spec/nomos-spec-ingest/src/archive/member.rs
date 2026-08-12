//! One member of a restored family.

use crate::Origin;
use crate::Restored;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member
{
    pub id: String,
    pub family: Restored,
    /// The name as the document gives it.
    pub name: String,
    pub document: String,
    pub origin: Origin,
    /// The name an alias resolves, where the document names the member rather than
    /// numbering it. `None` for a heading whose title is a sentence.
    pub alias: Option<String>,
}
