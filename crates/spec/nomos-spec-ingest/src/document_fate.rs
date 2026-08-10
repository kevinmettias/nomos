//! What became of one document.

use crate::relocation::Relocation;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocumentFate
{
    pub appeared: Vec<String>,
    pub disappeared: Vec<String>,
    pub changed: Vec<String>,
    pub relocated: Vec<Relocation>,
}
