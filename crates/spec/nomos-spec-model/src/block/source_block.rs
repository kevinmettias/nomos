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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Content_Hash_Should_Hash_The_Blocks_Text_Verbatim()
    {
        let block = Block_With_Text("Body.");

        assert_eq!(block.Content_Hash(), ContentHash::Of("Body."));
    }

    #[test]
    fn Test_Normalized_Hash_Should_Hash_The_Blocks_Normalized_Text()
    {
        let block = Block_With_Text("a   b");

        assert_eq!(block.Normalized_Hash(), ContentHash::Of_Normalized("a   b"));
        assert_ne!(block.Normalized_Hash(), block.Content_Hash(), "the whitespace run must collapse");
    }

    fn Block_With_Text(text: &str) -> SourceBlock
    {
        return SourceBlock {
            ordinal: 1,
            kind: BlockKind::Prose,
            heading_path: Vec::new(),
            text: text.to_owned(),
        };
    }
}
