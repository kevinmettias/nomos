//! What a real `nomos gate run` produced, including the check facts behind it.

use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Digest128, RunId};
use nomos_platform::Timestamp;
use std::path::PathBuf;

use crate::GateRunOutcome;
use super::GateFindings;

/// What a real `nomos gate run` produced.
///
/// `check_outcome` is carried in full -- including `Claim`, for information only, the same
/// choice `OD-COMPLETENESS-004` already made for `nomos check`'s own exit code -- so a
/// caller that wants the finer detail behind `disposition` does not have to re-walk or
/// re-judge anything to get it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateRunResult
{
    /// The identity of this execution -- `OD-WORKFLOW-001`'s first real consumer for
    /// `RunId`. Supplied by the caller to [`crate::Run_Gate`], not derived from anything
    /// else in this struct: two runs over the same `root` with the same findings are still
    /// two different executions.
    pub run: RunId,
    /// The tree this run judged.
    pub root: PathBuf,
    /// What [`nomos_check_orchestration::Run`] (or the walk decision made before it was
    /// ever called) produced.
    pub check_outcome: CheckOutcome,
    /// Every finding this run reduced, grouped by why it does or does not block.
    pub findings: GateFindings,
    /// The reduced verdict. When [`crate::GateCommand::phases`] declares a phase policy,
    /// this is [`crate::Phased_Disposition`]'s own answer rather than the flat
    /// [`crate::Disposition_Of_Findings`] every earlier increment computed alone -- a phase
    /// policy can turn a run that would otherwise fail into one that passes, when every
    /// blocking finding is named by some phase and no phase failed unapproved. `findings`
    /// still carries every finding this run reduced, in full, regardless of what any phase
    /// decided about them.
    pub disposition: GateRunOutcome,
    /// Every declared policy entry that matched no finding in this run, described as a
    /// reader would need to find it in the file that declares it.
    ///
    /// Not a failure. A policy legitimately outlives the finding it was written for, and a
    /// repository whose debt was paid must not fail its own gate for having paid it. But it
    /// is never silent either: `OD-GATE-024` was filed because an author who writes an entry
    /// that matches nothing gets no error, no warning and no effect, and cannot tell a
    /// mis-spelling from a finding that has since been fixed.
    ///
    /// Text rather than a typed entry, deliberately. This is a report line — the identity a
    /// caller would match on is already in the policy the entry came from, and duplicating
    /// it here would be a second addressing scheme for the same question.
    pub unmatched_policy: Vec<String>,
    /// Why this run judged its tree and still reached no verdict.
    ///
    /// `None` whenever `disposition` is a real verdict. `None` as well when nothing was
    /// judged at all: `check_outcome` is the reason in that case, in more detail than a
    /// second vocabulary here could add, and restating it would be two encodings of one
    /// fact. So this is `Some` for exactly the runs that got all the way through judging and
    /// came out without a verdict, which is the case nothing could previously explain.
    pub no_verdict: Option<NoVerdict>,
    /// What judged this run, so that a comparison against another run can say whether the
    /// two were judged alike.
    ///
    /// `None` means unknown, and `OD-GATE-031` decided what that means: a comparison treats
    /// it as incomparable rather than assuming it matches. [`crate::Run_Gate`] always fills
    /// `Some`, so `None` reaches a comparison only from a result built by hand or read back
    /// from a store that predates this field -- and in both cases the honest answer is that
    /// nobody knows what judged it, which must not read as "the same thing that judged the
    /// other side".
    pub provenance: Option<GateRunProvenance>,
}

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

/// Why a run that judged its tree still reached no verdict.
///
/// [`GateRunOutcome::Indeterminate`] is the disposition; this is the cause. Both producers
/// computed one and threw it away until this type existed, so `nomos-cli` could report that
/// a run had reached no verdict and could not say which of them had happened -- the result
/// did not know, and the serde message naming the offending key had already been dropped.
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
