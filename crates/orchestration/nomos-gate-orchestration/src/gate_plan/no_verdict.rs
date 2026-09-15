//! Why a run that judged its tree still reached no verdict.

/// Why a run that judged its tree still reached no verdict.
///
/// [`crate::GateRunOutcome::Indeterminate`] is the disposition; this is the cause. Both
/// producers computed one and threw it away until this type existed, so `nomos-cli` could
/// report that a run had reached no verdict and could not say which of them had happened --
/// the result did not know, and the serde message naming the offending key had already been
/// dropped.
///
/// The two policy variants are kept apart for the reason `Applicability` keeps
/// `MissingCapability` and `ProviderUnavailable` apart rather than folding them into one
/// "could not look": the remedies differ. A file that cannot be read is a path or a
/// permission; a file that cannot be parsed is its content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoVerdict
{
    /// A `nomos-gate.json` is present under the run's root and could not be read at all, so
    /// there were no declared rules to reduce this run's findings by. Carries the path and
    /// the failure as the file system reported them.
    UnreadablePolicy(String),
    /// A `nomos-gate.json` is present and is not a policy this reader accepts, so there were
    /// again no declared rules to reduce by. Carries the reader's own message, which names
    /// the refused key for a mis-spelling and the position for a syntax error -- the detail
    /// a person actually needs, and the one this crate used to compute and discard.
    ///
    /// A present-but-broken file is deliberately not treated as an absent one, for the
    /// reason `Resolve_Gate_Policy`'s own doc gives: a repository that meant to suppress a
    /// finding and mis-spelled the file would otherwise get a build that passes for a reason
    /// nobody chose.
    MalformedPolicy(String),
    /// The declared coverage floor is `CoveragePolicy::RequireCompleteness` and this run's
    /// selected findings are an incomplete claim, so a run that would otherwise have passed
    /// is not reported as one.
    ///
    /// `OD-GATE-016`'s own decision, and the only member of this enum that is not a defect
    /// in anything: nothing is wrong with the tree or the configuration, and the repository
    /// asked for exactly this. A reader that cannot tell it from a broken policy file will
    /// go looking for a fault that is not there, which is why it is a variant of its own
    /// rather than a shared "no verdict".
    IncompleteCoverage,
}
