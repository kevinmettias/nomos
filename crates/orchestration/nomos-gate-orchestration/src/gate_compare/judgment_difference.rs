//! One judgment input, other than the source, that differed between two runs.

/// One judgment input, other than the source, that differed between two runs.
///
/// The source is deliberately absent. A difference there is the licensed cause — the thing a
/// comparison exists to attribute a change to — rather than a caveat on attributing one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JudgmentDifference
{
    /// The two sides judged under different policies.
    ///
    /// The likeliest of the three by far, and the least visible: `Run_Gate` resolves
    /// `nomos-gate.json` from `command.root`, and a comparison judges two roots, so comparing
    /// two checkouts compares two policies whether or not anybody meant to.
    Policy,
    /// The two sides were allowed to look at different things — different rules counted, or
    /// different paths were in scope. A side told to look at less has fewer findings for that
    /// reason, and its absent findings otherwise read as the other side's additions.
    Selection,
    /// The two sides were judged by different instruments: a different build variant, or a
    /// different rule set. The domain table in `nomos-contracts` declares the analysis kernel
    /// `CrossPlatform`, which is strictly weaker than `CrossBinary`, so reproducibility across
    /// two instruments is not claimed by this workspace and must not be assumed by a caller.
    Instrument,
}
