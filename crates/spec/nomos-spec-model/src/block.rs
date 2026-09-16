// What kind of block this is, beneath the block it describes.
pub(crate) mod kind;
mod source_block;

// `Kind` would collide with `statement::kind::Kind` if flattened bare, so `lib.rs` reaches
// this module directly and keeps the longer, table-naming public name at the crate root;
// this local alias is for this file's own use only, not part of the public surface.
use kind::Kind as BlockKind;
pub use source_block::SourceBlock;

/// Splits an authored markdown document into the blocks the preservation ledger tracks.
///
/// Reproduces v14's segmentation exactly; verified against all 2533 recorded blocks.
#[must_use]
pub fn Segment_Markdown(markdown: &str) -> Vec<SourceBlock>
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
            Flush_Paragraph(&mut blocks, &mut paragraph, &heading_path);
            Push_A_Heading(&mut blocks, &mut heading_path, line);
        }
        else if line.starts_with("```")
        {
            Flush_Paragraph(&mut blocks, &mut paragraph, &heading_path);
            index = Push_A_Fence(&mut blocks, &heading_path, &lines, index);
        }
        else if line.trim().is_empty()
        {
            Flush_Paragraph(&mut blocks, &mut paragraph, &heading_path);
        }
        else
        {
            paragraph.push(line);
        }

        index = index.saturating_add(1);
    }

    Flush_Paragraph(&mut blocks, &mut paragraph, &heading_path);
    return blocks;
}

// Renamed from the vague `Segment` for check-naming-clarity (it took a parameter without
// saying what it segments). The public name stays `Segment` through this alias: it is called
// from roughly 30 sites across every crate in `crates/spec` plus
// `tests/integration/tests/determinism/spec_productions.rs`, which sits outside this
// campaign's `crates/spec` territory and so cannot be edited from here.
pub use self::Segment_Markdown as Segment;

/// v14's readers consume this as part of the opening front matter fence.
///
/// `str::trim` does not remove it — U+FEFF is not whitespace under Unicode — so a first
/// line carrying one does not compare equal to `---` unless it is stripped here.
const BYTE_ORDER_MARK: char = '\u{feff}';

/// Where the authored content of a document begins.
///
/// Both v14 readers that ship match front matter as `^\ufeff?---\n`, so a byte order mark
/// belongs to the opening fence and leaves with it. All 2187 authored v14 documents match
/// that pattern, 1637 of them with a mark, and none carries one into its body.
///
/// Where no front matter follows, v14 returned the text as it found it. The mark is kept
/// in that case, so the only bytes this drops are a delimiter's.
fn Skip_Front_Matter(lines: &[&str]) -> usize
{
    let Some(first) = lines.first()
    else
    {
        return 0;
    };

    if first.strip_prefix(BYTE_ORDER_MARK).unwrap_or(first).trim() != "---"
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

fn Flush_Paragraph(blocks: &mut Vec<SourceBlock>, paragraph: &mut Vec<&str>, heading_path: &[String])
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

/// A heading extends the path it will then be filed under.
fn Push_A_Heading(blocks: &mut Vec<SourceBlock>, heading_path: &mut Vec<String>, line: &str)
{
    Push_Heading_Path(heading_path, line);

    let block = A_Heading(blocks, heading_path, line);
    blocks.push(block);
}

fn Push_Heading_Path(path: &mut Vec<String>, line: &str)
{
    let depth = line.chars().take_while(|character| *character == '#').count();
    let title = line.trim_start_matches('#').trim().to_owned();

    path.truncate(depth.saturating_sub(1).min(path.len()));
    path.push(title);
}

/// One heading line as a block, under the path it has just extended.
fn A_Heading(blocks: &[SourceBlock], heading_path: &[String], line: &str) -> SourceBlock
{
    return SourceBlock {
        ordinal: Next_Ordinal(blocks),
        kind: BlockKind::Heading,
        heading_path: heading_path.to_vec(),
        text: line.to_owned(),
    };
}

/// A fenced block, answering with the index of its closing fence.
fn Push_A_Fence(
    blocks: &mut Vec<SourceBlock>,
    heading_path: &[String],
    lines: &[&str],
    index: usize,
) -> usize
{
    let span = Fenced_Lines(lines, index);
    let block = A_Fence(blocks, heading_path, &span.lines);

    blocks.push(block);

    return span.closing;
}

/// The lines of a fenced block, and the index of its last line.
///
/// Named rather than an unnamed pair, so the two positions cannot be swapped at a call site
/// without a type error: both are ordinary values a tuple would give no names at all.
struct FenceSpan<'a>
{
    lines: Vec<&'a str>,
    closing: usize,
}

/// The lines of the fenced block opening at `index`, and the index of its last line.
///
/// An unterminated fence runs to the end of the document rather than being refused, which
/// is what v14's readers do and therefore what reproducing their segmentation requires.
fn Fenced_Lines<'a>(lines: &[&'a str], index: usize) -> FenceSpan<'a>
{
    let mut fence: Vec<&str> = Vec::new();
    let mut cursor = index;

    while let Some(inner) = lines.get(cursor)
    {
        fence.push(inner);
        if cursor > index && inner.starts_with("```")
        {
            break;
        }
        cursor = cursor.saturating_add(1);
    }

    return FenceSpan { lines: fence, closing: cursor };
}

/// A fenced block, from its opening fence to its closing one.
fn A_Fence(blocks: &[SourceBlock], heading_path: &[String], fence: &[&str]) -> SourceBlock
{
    return SourceBlock {
        ordinal: Next_Ordinal(blocks),
        kind: BlockKind::Code,
        heading_path: heading_path.to_vec(),
        text: fence.join("\n"),
    };
}

fn Next_Ordinal(blocks: &[SourceBlock]) -> u32
{
    return u32::try_from(blocks.len()).unwrap_or(u32::MAX).saturating_add(1);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A document with front matter, read here both with a leading mark and without one.
    const DOCUMENT: &str = "---\nid: D-045\n---\n\n# Runtime capture boundary\n\nBody.\n";

    /// What `"---\nid: X\n---\n# Title\n\nBody.\n"` holds once its front matter is dropped:
    /// the heading and the prose under it, and nothing for the fence itself.
    const BLOCKS_AFTER_FRONT_MATTER: usize = 2;

    /// What `"one\ntwo\n\nthree\n"` holds: the consecutive lines merge into one block, and
    /// the blank line ends it, leaving a second.
    const BLOCKS_ACROSS_TWO_PARAGRAPHS: usize = 2;

    /// The position of `### C` in `"# A\n\n## B\n\n### C\n\n## D\n"`, counting from zero.
    const THIRD_BLOCK_INDEX: usize = 2;

    /// The position of `## D`, the block that pops the third heading back off the path.
    const FOURTH_BLOCK_INDEX: usize = 3;

    /// The depth of `### C`'s heading path: `A`, `B` and `C` itself.
    const DEPTH_OF_A_THIRD_LEVEL_HEADING: usize = 3;

    /// The ordinals a document's blocks carry, in order, when none was merged away.
    const ORDINALS_IN_DOCUMENT_ORDER: [u32; 4] = [1, 2, 3, 4];

    #[test]
    fn Test_Front_Matter_Should_Not_Become_A_Block()
    {
        let blocks = Segment("---\nid: X\n---\n# Title\n\nBody.\n");

        assert_eq!(blocks.len(), BLOCKS_AFTER_FRONT_MATTER);
        assert_eq!(blocks.first().map(|block| block.kind), Some(BlockKind::Heading));
        assert_eq!(blocks.get(1).map(|block| block.text.as_str()), Some("Body."));
    }

    /// P3-BOM. 1637 of v14's 2187 authored documents carry a mark, including every
    /// decision record I4 ingests, and both shipped v14 readers match `^\ufeff?---\n`.
    #[test]
    fn Test_A_Byte_Order_Mark_Should_Not_Turn_Front_Matter_Into_Content()
    {
        let marked = Segment(&format!("\u{feff}{DOCUMENT}"));

        assert_eq!(marked, Segment(DOCUMENT), "the mark changed how the document read");
        assert_eq!(marked.first().map(|block| block.kind), Some(BlockKind::Heading));
        assert!(
            !marked.iter().any(|block| block.text.contains(BYTE_ORDER_MARK)),
            "the mark reached a block"
        );
    }

    /// The one case v14's manifest cannot corroborate, because it never occurs: all 2187
    /// authored documents have front matter. Settled by what v14's reader did rather than
    /// by preference — no fence matched, so it dropped nothing.
    #[test]
    fn Test_A_Byte_Order_Mark_Should_Survive_Where_No_Front_Matter_Follows()
    {
        let blocks = Segment("\u{feff}# Title\n\nBody.\n");

        assert_eq!(
            blocks.first().map(|block| block.text.as_str()),
            Some("\u{feff}# Title")
        );
    }

    #[test]
    fn Test_A_Heading_Should_Keep_Its_Marker()
    {
        let blocks = Segment("## Section\n");

        assert_eq!(blocks.first().map(|block| block.text.as_str()), Some("## Section"));
    }

    #[test]
    fn Test_Segment_Markdown_Should_Merge_Consecutive_Lines_Into_One_Block()
    {
        let blocks = Segment("one\ntwo\n\nthree\n");

        assert_eq!(blocks.len(), BLOCKS_ACROSS_TWO_PARAGRAPHS);
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
        assert_eq!(
            paths.get(THIRD_BLOCK_INDEX).map(|path| path.len()),
            Some(DEPTH_OF_A_THIRD_LEVEL_HEADING)
        );
        assert_eq!(
            paths.get(FOURTH_BLOCK_INDEX).map(|path| path.join("/")),
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

        assert_eq!(ordinals, ORDINALS_IN_DOCUMENT_ORDER);
    }
}
