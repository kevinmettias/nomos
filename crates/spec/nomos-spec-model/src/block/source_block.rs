use super::BlockKind;
use crate::ContentHash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBlock
{
    pub ordinal: u32,
    pub kind: BlockKind,
    pub heading_path: Vec<String>,
    pub text: String,
}

impl SourceBlock
{
    #[must_use]
    pub fn Content_Hash(&self) -> ContentHash
    {
        return ContentHash::Of(&self.text);
    }

    #[must_use]
    pub fn Normalized_Hash(&self) -> ContentHash
    {
        return ContentHash::Of_Normalized(&self.text);
    }
}
