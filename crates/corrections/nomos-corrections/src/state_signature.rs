//! The state an affected scope was observed in, reduced to one comparable value.

use nomos_contracts::Digest128;

/// A digest over everything one observation of an affected scope reported.
///
/// `COR-005` asks that, after staging, the system "rerun affected rules, compare state
/// signatures, and commit or roll back". This is the value those comparisons are made
/// over: two observations of the same scope are the same state exactly when their
/// signatures are equal.
///
/// Order-insensitive. The observations are sorted before they are digested, because a
/// rule runner's output order is not part of what the scope is *in* -- two runs that
/// reported the same set of findings in two orders would otherwise look like two
/// different states, and a convergence check built on that would never see a repeat.
///
/// What an observation *is* is deliberately not decided here. This crate stays generic
/// over what a correction is (`OD-CORRECTIONS-001`), so it stays generic over what a
/// finding is too: a caller renders each observation to a string it considers stable --
/// rule, subject and location, say -- and this compares what it was handed. Digesting
/// rather than keeping the strings is what makes a state cheap to remember and cheap to
/// compare, which is the whole point of a signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateSignature
{
    digest: Digest128,
}

impl StateSignature
{
    /// The signature of a scope observed to be in exactly `observations`.
    ///
    /// Multiplicity is preserved: two identical observations are not one. Nothing here
    /// judges whether a repeated observation is meaningful, and silently collapsing it
    /// would be deciding that it is not.
    #[must_use]
    pub fn Of(observations: &[String]) -> Self
    {
        let mut sorted: Vec<&str> = observations.iter().map(String::as_str).collect();
        sorted.sort_unstable();

        let parts: Vec<&[u8]> = sorted.iter().map(|observation| return observation.as_bytes()).collect();

        return Self {
            digest: nomos_model::Digest_Of_Parts(&parts),
        };
    }

    #[must_use]
    pub const fn Digest(&self) -> Digest128
    {
        return self.digest;
    }
}

impl core::fmt::Display for StateSignature
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.digest.fmt(formatter);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Observations(observations: &[&str]) -> Vec<String>
    {
        return observations.iter().map(|observation| return (*observation).to_owned()).collect();
    }

    #[test]
    fn Test_Of_Should_Give_The_Same_Signature_To_The_Same_Observations()
    {
        let first = StateSignature::Of(&Observations(&["naming::a.rs:1", "mirror::b.rs"]));
        let second = StateSignature::Of(&Observations(&["naming::a.rs:1", "mirror::b.rs"]));

        assert_eq!(first, second);
    }

    /// The property the sort exists for: a rule runner's output order is not part of the
    /// state, so two orders of one set are one signature.
    #[test]
    fn Test_Of_Should_Ignore_The_Order_Observations_Arrive_In()
    {
        let forward = StateSignature::Of(&Observations(&["naming::a.rs:1", "mirror::b.rs"]));
        let reversed = StateSignature::Of(&Observations(&["mirror::b.rs", "naming::a.rs:1"]));

        assert_eq!(forward, reversed);
    }

    #[test]
    fn Test_Of_Should_Separate_A_Scope_That_Lost_An_Observation()
    {
        let before = StateSignature::Of(&Observations(&["naming::a.rs:1", "mirror::b.rs"]));
        let after = StateSignature::Of(&Observations(&["mirror::b.rs"]));

        assert_ne!(before, after);
    }

    /// Two observations that only differ in where one ends and the next begins are two
    /// states, which is what `Digest_Of_Parts` frames boundaries for.
    #[test]
    fn Test_Of_Should_Treat_Observation_Boundaries_As_Significant()
    {
        let split_early = StateSignature::Of(&Observations(&["ab", "c"]));
        let split_late = StateSignature::Of(&Observations(&["a", "bc"]));

        assert_ne!(split_early, split_late);
    }

    #[test]
    fn Test_Of_Should_Keep_A_Repeated_Observation_Rather_Than_Collapse_It()
    {
        let once = StateSignature::Of(&Observations(&["mirror::b.rs"]));
        let twice = StateSignature::Of(&Observations(&["mirror::b.rs", "mirror::b.rs"]));

        assert_ne!(once, twice);
    }

    #[test]
    fn Test_Of_Should_Give_A_Clean_Scope_A_Signature_Of_Its_Own()
    {
        let clean = StateSignature::Of(&[]);
        let dirty = StateSignature::Of(&Observations(&["mirror::b.rs"]));

        assert_ne!(clean, dirty);
    }

    #[test]
    fn Test_Display_Should_Render_The_Digest()
    {
        let signature = StateSignature::Of(&Observations(&["mirror::b.rs"]));

        assert_eq!(signature.to_string(), signature.Digest().to_string());
    }
}
