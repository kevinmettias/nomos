//! The identity of what judged a run, as `OD-GATE-031` decided it.

use nomos_contracts::Digest128;
use nomos_platform::Timestamp;

/// The identity of what judged a run, as `OD-GATE-031` decided it.
///
/// # Why a run needs one
///
/// `crate::Compare_Gate_Runs` reports which findings moved between two runs, and a caller
/// reads that as a fact about the repository, because that is what a gate is for. It is only
/// that when every judgment input other than the source was compatible between the two sides.
/// Before this type, every one of those inputs reached [`crate::Run_Gate`] and was discarded
/// by it, so a finished run could not say what judged it even though the function that
/// produced it held all five.
///
/// # Why these five and not more
///
/// `OD-GATE-031` sized this to what can actually differ between two sides *today*, rather
/// than to the eventual shape. A provider binary's version, the host operating system and the
/// identity of the nomos build are constant across a same-process comparison, which is the
/// only kind `OD-GATE-022-A` has compare perform, so a field for any of them would be one no
/// test could make differ. They arrive with the run history that record defers.
///
/// Which rules could actually look is also absent, and derivable rather than missing: a rule
/// whose provider could not run reports `Applicability::MissingCapability` or
/// `ProviderUnavailable`, `nomos_check_orchestration::Claim_Of` already reads exactly those,
/// and a comparison holds both sides' findings. A field would be a second encoding of what
/// the findings carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GateRunProvenance
{
    /// Every file this run judged, by path and content.
    ///
    /// The thing a difference is *allowed* to be attributed to. A comparison needs it in
    /// order to establish that the repository changed, rather than reaching that conclusion
    /// because nothing else explained the difference.
    pub source: Digest128,
    /// The policy this run judged under, after the declared `nomos-gate.json` was resolved
    /// over the command.
    ///
    /// The input most likely to differ between two sides and least likely to be noticed:
    /// `Run_Gate` reads the policy from `command.root`, and a comparison judges two roots, so
    /// comparing two checkouts compares two policies without anybody having asked for that.
    pub policy: Digest128,
    /// Which rules were allowed to count and which paths were in scope.
    ///
    /// A side that was told to look at less has fewer findings for that reason, and must not
    /// read as a side that looked at everything and found less.
    pub selection: Digest128,
    /// What did the judging: the build variant, and the rule set this build carries with each
    /// rule's contract record and version.
    ///
    /// Two different instruments measuring one tree is exactly the case the domain table in
    /// `nomos-contracts` declines to claim reproducibility for -- the analysis kernel is
    /// declared `CrossPlatform`, which is strictly weaker than `CrossBinary`.
    pub instrument: Digest128,
    /// The moment this run was judged against.
    ///
    /// The domain table declares the analysis kernel `StateTemporal`, which makes time a
    /// judgment input rather than a label, and it genuinely is one: a temporary waiver stops
    /// applying against this moment, so two runs over an identical tree under an identical
    /// policy can still disagree because one of them happened later.
    ///
    /// The moment itself rather than a digest of it, because unlike the other four this is
    /// not an identity to be matched -- a reader comparing two runs wants to know which was
    /// later, and a digest would destroy exactly that.
    pub at: Timestamp,
}
