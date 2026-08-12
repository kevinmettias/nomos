//! What changed between two adjacent revisions.

use core::fmt::Write as _;

/// How many members a summary line names before it falls back to the count alone. Naming a
/// few is what makes a count checkable by eye; naming all of them makes the summary into the
/// report it is supposed to introduce.
const NAMED_IN_A_SUMMARY: usize = 3;
/// What became of every path between two adjacent revisions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PairChange
{
    pub from: String,
    pub to: String,
    pub appeared: Vec<String>,
    pub disappeared: Vec<String>,
    pub changed: Vec<String>,
    /// Absent in `from`, present in `to`, and present in some revision before `from`.
    pub reappeared: Vec<String>,
}

impl PairChange
{
    /// Names the pair and the size of each set, then a few members of each.
    ///
    /// Never a bare total: "38 disappeared" is the number that starts an argument, and the
    /// paths are what ends it.
    #[must_use]
    pub fn Summary(&self) -> String
    {
        let mut line = format!("{} -> {}", self.from, self.to);
        for (label, set) in [
            ("appeared", &self.appeared),
            ("disappeared", &self.disappeared),
            ("changed in place", &self.changed),
            ("reappeared", &self.reappeared),
        ]
        {
            if set.is_empty()
            {
                continue;
            }
            let named: Vec<&str> = set.iter().take(NAMED_IN_A_SUMMARY).map(String::as_str).collect();
            let _ = write!(
                line,
                "\n  {label}: {} ({}{})",
                set.len(),
                named.join(", "),
                if set.len() > named.len() { ", …" } else { "" }
            );
        }

        return line;
    }
}
