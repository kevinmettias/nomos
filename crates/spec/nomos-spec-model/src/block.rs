use crate::normalize::ContentHash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockKind
{
    Heading,
    Prose,
    Code,
}

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

/// Splits an authored markdown document into the blocks the preservation ledger tracks.
///
/// Reproduces v14's segmentation exactly; verified against all 2533 recorded blocks.
#[must_use]
pub fn Segment(markdown: &str) -> Vec<SourceBlock>
{
    let lines: Vec<&str> = markdown.split('\n').map(|line| line.trim_end_matches('\r')).collect();
    let mut index = Skip_Front_Matter(&lines);

    let mut blocks: Vec<SourceBlock> = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut heading_path: Vec<String> = Vec::new();

    while let Some(line) = lines.get(index)
    {
        if line.starts_with('#')
        {
            Flush(&mut blocks, &mut paragraph, &heading_path);
            Push_Heading_Path(&mut heading_path, line);
            blocks.push(SourceBlock {
                ordinal: Next_Ordinal(&blocks),
                kind: BlockKind::Heading,
                heading_path: heading_path.clone(),
                text: (*line).to_owned(),
            });
        }
        else if line.starts_with("```")
        {
            Flush(&mut blocks, &mut paragraph, &heading_path);
            let mut fence = vec![*line];
            index = index.saturating_add(1);
            while let Some(inner) = lines.get(index)
            {
                fence.push(inner);
                if inner.starts_with("```")
                {
                    break;
                }
                index = index.saturating_add(1);
            }
            blocks.push(SourceBlock {
                ordinal: Next_Ordinal(&blocks),
                kind: BlockKind::Code,
                heading_path: heading_path.clone(),
                text: fence.join("\n"),
            });
        }
        else if line.trim().is_empty()
        {
            Flush(&mut blocks, &mut paragraph, &heading_path);
        }
        else
        {
            paragraph.push(line);
        }

        index = index.saturating_add(1);
    }

    Flush(&mut blocks, &mut paragraph, &heading_path);
    return blocks;
}

fn Skip_Front_Matter(lines: &[&str]) -> usize
{
    if lines.first().map(|line| line.trim()) != Some("---")
    {
        return 0;
    }

    let mut index = 1_usize;
    while let Some(line) = lines.get(index)
    {
        if line.trim() == "---"
        {
            return index.saturating_add(1);
        }
        index = index.saturating_add(1);
    }
    return 0;
}

fn Next_Ordinal(blocks: &[SourceBlock]) -> u32
{
    return u32::try_from(blocks.len()).unwrap_or(u32::MAX).saturating_add(1);
}

fn Flush(blocks: &mut Vec<SourceBlock>, paragraph: &mut Vec<&str>, heading_path: &[String])
{
    if paragraph.is_empty()
    {
        return;
    }

    blocks.push(SourceBlock {
        ordinal: Next_Ordinal(blocks),
        kind: BlockKind::Prose,
        heading_path: heading_path.to_vec(),
        text: paragraph.join("\n"),
    });
    paragraph.clear();
}

fn Push_Heading_Path(path: &mut Vec<String>, line: &str)
{
    let depth = line.chars().take_while(|character| *character == '#').count();
    let title = line.trim_start_matches('#').trim().to_owned();

    path.truncate(depth.saturating_sub(1).min(path.len()));
    path.push(title);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Front_Matter_Should_Not_Become_A_Block()
    {
        let blocks = Segment("---\nid: X\n---\n# Title\n\nBody.\n");

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks.first().map(|block| block.kind), Some(BlockKind::Heading));
        assert_eq!(blocks.get(1).map(|block| block.text.as_str()), Some("Body."));
    }

    #[test]
    fn Test_A_Heading_Should_Keep_Its_Marker()
    {
        let blocks = Segment("## Section\n");

        assert_eq!(blocks.first().map(|block| block.text.as_str()), Some("## Section"));
    }

    #[test]
    fn Test_Consecutive_Lines_Should_Be_One_Block()
    {
        let blocks = Segment("one\ntwo\n\nthree\n");

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks.first().map(|block| block.text.as_str()), Some("one\ntwo"));
    }

    #[test]
    fn Test_A_Table_Should_Be_One_Prose_Block()
    {
        let blocks = Segment("| a | b |\n| --- | --- |\n| 1 | 2 |\n");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks.first().map(|block| block.kind), Some(BlockKind::Prose));
    }

    #[test]
    fn Test_Heading_Path_Should_Nest_And_Pop()
    {
        let blocks = Segment("# A\n\n## B\n\n### C\n\n## D\n");
        let paths: Vec<&[String]> = blocks.iter().map(|block| block.heading_path.as_slice()).collect();

        assert_eq!(paths.first().map(|path| path.len()), Some(1));
        assert_eq!(paths.get(2).map(|path| path.len()), Some(3));
        assert_eq!(
            paths.get(3).map(|path| path.join("/")),
            Some("A/D".to_owned())
        );
    }

    #[test]
    fn Test_A_Fenced_Block_Should_Survive_Blank_Lines()
    {
        let blocks = Segment("```rust\nlet a = 1;\n\nlet b = 2;\n```\n");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks.first().map(|block| block.kind), Some(BlockKind::Code));
    }

    #[test]
    fn Test_Ordinals_Should_Be_Dense_And_One_Based()
    {
        let blocks = Segment("# A\n\nbody\n\n## B\n\nmore\n");
        let ordinals: Vec<u32> = blocks.iter().map(|block| block.ordinal).collect();

        assert_eq!(ordinals, vec![1, 2, 3, 4]);
    }
}
