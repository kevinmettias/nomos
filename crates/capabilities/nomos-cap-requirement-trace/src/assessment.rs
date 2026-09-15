//! What an assessment is: the verdict, the places it is about, and the record behind it,
//! gathered into the one committed entry this crate reads.
//!
//! Ported from `tests/contract/tests/requirement_trace/assessment.rs`, which `OD-TRACE-001`
//! and `OD-TRACE-002` already settled the shape of — this module is that shape, moved to
//! where a real gate-composed rule can reach it, not a redesign of it. The vocabulary lives
//! apart from the parser ([`crate::registry`]) and the predicates ([`crate::predicates`])
//! for the identical reason the test suite already split them: every other module here
//! needs these types. The verdict and the site each carry their own file under this one,
//! since an entry is the thing a reader names, not the two types it is built from.

mod site;
mod verdict;

pub use site::Site;
pub use verdict::Verdict;

/// Where the committed entries live, relative to the workspace root.
pub const REGISTRY: &str = "tests/contract/requirements";

/// One committed assessment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment
{
    /// The corpus requirement identifier. The file stem, never a key inside the file.
    pub requirement: String,
    /// What the assessment says.
    pub verdict: Verdict,
    /// The governing record carrying the reasoning, when one is named.
    pub record: Option<String>,
    /// Every place the verdict is about. Never empty.
    pub sites: Vec<Site>,
    /// Where the requirement is not yet satisfied, when the verdict is `Partial`.
    ///
    /// Checked exactly like `sites` — the same `path#symbol` shape, resolved the same
    /// way — so a `Partial` entry decays visibly the way a `Met` one does instead of
    /// quietly becoming a description of nothing. Empty for every other verdict;
    /// `Partial` requires at least one, enforced by [`crate::registry::Parse`].
    pub gaps: Vec<Site>,
}
