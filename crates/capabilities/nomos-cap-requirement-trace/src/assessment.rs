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

use nomos_contracts::RuleId;

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
    /// Every rule this build enforces the assessed part of the requirement with.
    ///
    /// `OD-HOST-015` decided the line and what it means: a finding from one of these rules
    /// bears on this requirement, because the rule is where this build enforces the part of
    /// the requirement [`Self::sites`] satisfy. It is evidence about the verdict at a site,
    /// never the verdict, and it is declared rather than derived -- that record refused
    /// reading a finding's location as a requirement link, because a site is where a
    /// requirement is *satisfied* and a finding's location is where a rule *fired*.
    ///
    /// **Empty means nobody declared a rule. It does not mean no rule bears**, and nothing
    /// may read it that way: absence is what absence means everywhere in this registry
    /// (`OD-TRACE-001`), and `OD-TRACE-002` refused a written `Unassessed` for the same
    /// reason a `rule: none` is refused here -- it would record that somebody looked, which
    /// is a claim nobody would have checked. `OD-HOST-015` measured that fifty-nine of this
    /// build's seventy-one rules are ported code-standards rules bearing on no corpus
    /// requirement at all, so a line per entry would answer a question nobody asked.
    ///
    /// The kernel's own [`RuleId`] rather than a second string spelling of the identifier a
    /// `Finding` already carries. Resolving one against the rules this build composes is
    /// `tests/contract`'s, not this crate's: `Capability Contract` may not name `Rules`.
    pub rules: Vec<RuleId>,
}
