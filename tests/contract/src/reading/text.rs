//! Line arithmetic over a bounded slice of a file.
//!
//! Every offset here is absolute into the whole file and every range is bounded by the item
//! being read, so a scan cannot walk out of the block it was given.

/// The end of the line starting at `from`, bounded by `end`.
pub(crate) fn Line_End(text: &str, from: usize, end: usize) -> usize
{
    return text
        .get(from..end)
        .and_then(|rest| return rest.find('\n'))
        .map_or(end, |at| return from.saturating_add(at));
}

/// The start of the line after `from`, bounded by `end`.
pub(crate) fn Next_Line(text: &str, from: usize, end: usize) -> usize
{
    let line_end = Line_End(text, from.min(end), end);

    return line_end.saturating_add(1).max(from.saturating_add(1));
}

/// Collapses every run of whitespace to a single space.
pub(crate) fn Collapsed(text: &str) -> String
{
    return text.split_whitespace().collect::<Vec<&str>>().join(" ");
}
