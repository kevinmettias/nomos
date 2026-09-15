//! Which of `crate::GateFindings`'s own buckets a finding fell into.

/// Which of `crate::GateFindings`'s own buckets a finding fell into.
///
/// One variant per bucket, and that correspondence is load-bearing rather than tidy:
/// `Population_Of` walks this list to build the population a comparison is computed over, so
/// a bucket with no variant here is a bucket whose findings are invisible to `compare`. They
/// would not be reported as unchanged -- they would be missing from one side, which reads as
/// *removed*, which is the false causal story this crate's comparison work exists to stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingDisposition
{
    Blocking,
    Calibrated,
    Suppressed,
    Baselined,
    /// A baseline entry matched, and its scope held more occurrences than the entry accepted.
    ///
    /// Distinct from [`Self::Blocking`] because the two answer differently when a reader asks
    /// why: this one is a tolerance that ran out of room, and moving between the two is a real
    /// transition a comparison should show rather than absorb. Distinct from
    /// [`Self::Baselined`] because it did not hold.
    BaselineExceeded,
}
