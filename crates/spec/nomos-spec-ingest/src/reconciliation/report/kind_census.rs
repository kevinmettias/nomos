//! How many of each kind a revision holds.

/// Counts of content kinds in one revision, each naming its unit.
///
/// Both readings of a table and both readings of a code block, because the plan states
/// the line counts and the store answers the block counts, and D-132 says a figure that
/// disagrees is recorded rather than replaced.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KindCensus
{
    pub documents: u32,
    pub documents_with_tables: u32,
    pub pipe_lines: u32,
    pub non_separator_rows: u32,
    pub content_rows: u32,
    pub fence_lines: u32,
    pub code_blocks: u32,
}
