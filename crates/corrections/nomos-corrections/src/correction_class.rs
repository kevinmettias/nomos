//! How a correction was, or will be, carried out.

/// `COR-001`: "Fix actions shall distinguish mechanical correction, proposed correction,
/// agent correction, and interactive correction."
///
/// A caller declares this when constructing a [`crate::CorrectionCandidate`] -- this
/// crate does not decide it. `OD-CORRECTIONS-001`'s own module doc is explicit that "no
/// agent and no model backend appears anywhere in this crate... what a candidate would do
/// is decided before this crate exists to run it," and this type is exactly that
/// decision, carried rather than computed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CorrectionClass
{
    /// A deterministic, judgment-free edit -- the same edit every time, for the same
    /// input.
    Mechanical,
    /// A suggested change a human or a rule proposed without staging it.
    Proposed,
    /// A change an agent generated.
    Agent,
    /// A change made through direct, real-time interaction with an operator.
    Interactive,
}

impl CorrectionClass
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Mechanical => "mechanical",
            Self::Proposed => "proposed",
            Self::Agent => "agent",
            Self::Interactive => "interactive",
        };
    }
}

impl core::fmt::Display for CorrectionClass
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Be_Distinct_Per_Variant()
    {
        let mut labels = vec![
            CorrectionClass::Mechanical.Label(),
            CorrectionClass::Proposed.Label(),
            CorrectionClass::Agent.Label(),
            CorrectionClass::Interactive.Label(),
        ];
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two classes share a wire spelling");
    }
}
