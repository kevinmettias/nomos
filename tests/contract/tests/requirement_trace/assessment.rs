//! What an assessment is: a verdict, the places it is about, and the record behind it.
//!
//! The vocabulary rather than the reading or the comparing, because every other module in
//! this suite is one of those two and both need these three types.

/// Where the committed entries live, relative to the workspace root.
pub(crate) const REGISTRY: &str = "tests/contract/requirements";

/// What an assessment says about a requirement.
///
/// Four verdicts in `OD-TRACE-001` and three of them here. The fourth, `Unassessed`, is
/// held by the *absence* of an entry and is refused as a written word by
/// [`crate::registry::Parse`] — a file saying `Unassessed` would be somebody looking and
/// recording that they had not, which is the one thing this registry must not be able to
/// express.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict
{
    /// The site satisfies the requirement, and the entry names where.
    Met,
    /// The site is derived from the requirement and departs from it deliberately. The
    /// entry names the governing record carrying the reason; a divergence with no record
    /// is not a verdict, it is the state `OD-TRACE-001` exists to end.
    Diverges,
    /// The requirement is read as not reaching this build, with a record saying why a
    /// corpus requirement does not bind the thing built to enforce it.
    NotBinding,
}

impl Verdict
{
    /// The word an entry is written with.
    pub(crate) const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Met => "Met",
            Self::Diverges => "Diverges",
            Self::NotBinding => "NotBinding",
        };
    }

    /// Whether a verdict of this kind is owed a governing record.
    ///
    /// Both non-`Met` verdicts are. Departing from a normative requirement and declaring
    /// one out of scope are the same act from the corpus's side — somebody deciding this
    /// build will not do what the requirement says — and the reason is what makes either
    /// reviewable.
    pub(crate) const fn Owes_A_Record(self) -> bool
    {
        return matches!(self, Self::Diverges | Self::NotBinding);
    }
}

/// A place in the workspace a verdict is about.
///
/// A path alone would nearly never fire: files are renamed far less often than the symbols
/// inside them. The symbol is what makes the entry decay visibly when the thing it was
/// about is renamed out from under it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Site
{
    /// Repo-relative, forward slashes.
    pub(crate) path: String,
    /// Text that must occur in that file.
    pub(crate) symbol: String,
}

/// One committed assessment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Assessment
{
    /// The corpus requirement identifier. The file stem, never a key inside the file.
    pub(crate) requirement: String,
    /// What the assessment says.
    pub(crate) verdict: Verdict,
    /// The governing record carrying the reasoning, when one is named.
    pub(crate) record: Option<String>,
    /// Every place the verdict is about. Never empty.
    pub(crate) sites: Vec<Site>,
}
