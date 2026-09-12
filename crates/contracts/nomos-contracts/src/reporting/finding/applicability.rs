//! Whether a rule was evaluated against a subject, and if not, why not.

use serde::{Deserialize, Serialize};

use crate::DisplayLabel;

const SUPPORTED_LABEL: &str = "Supported";
const SUPPORTED_WITH_FALLBACK_LABEL: &str = "SupportedWithFallback";
const PARTIALLY_SUPPORTED_LABEL: &str = "PartiallySupported";
const NOT_APPLICABLE_LABEL: &str = "NotApplicable";
const MISSING_CAPABILITY_LABEL: &str = "MissingCapability";
const PROVIDER_UNAVAILABLE_LABEL: &str = "ProviderUnavailable";
const DEPENDENCY_UNAVAILABLE_LABEL: &str = "DependencyUnavailable";
const CONFIGURATION_DISABLED_LABEL: &str = "ConfigurationDisabled";
const UNPARSEABLE_LABEL: &str = "Unparseable";
const ANALYSIS_FAILED_LABEL: &str = "AnalysisFailed";
const AGENT_REQUIRED_LABEL: &str = "AgentRequired";

/// Why a rule did or did not produce a judgment about a subject.
///
/// This enum is the load-bearing expression of the product's first principle:
/// **unknown is not pass**. A region no analyzer could read is not healthy, it is
/// unknown, and the two must never render the same.
///
/// It is deliberately exhaustive and deliberately has no `Default`. A consumer's
/// `match` should break when the protocol grows a state, because a new way for
/// analysis to be incomplete is exactly the kind of change a consumer must not
/// silently absorb into an existing arm.
///
/// There is no `is_pass`. [`Applicability::Is_Evaluated`] is the closest thing, and
/// it is not a pass — it says only that a judgment was reached, not what it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Applicability
{
    /// The rule was evaluated with every capability it asked for, at the guarantee it
    /// asked for.
    Supported,
    /// The rule was evaluated, but a weaker provider than requested supplied at least
    /// one capability. The judgment stands; its guarantee is lower than nominal.
    SupportedWithFallback,
    /// The rule was evaluated over part of its subject only. What was not covered is
    /// recorded separately and is not implied to be clean.
    PartiallySupported,
    /// The rule does not bind this subject. This is the only variant that is a
    /// *positive* statement about the absence of a judgment.
    NotApplicable,
    /// No installed provider offers a capability the rule requires.
    MissingCapability,
    /// A provider that would satisfy the requirement is installed but could not run.
    ProviderUnavailable,
    /// A provider is present and runnable, but something it needs — an SDK, a
    /// toolchain, a license, a runtime — is not. Distinct from
    /// [`Applicability::ProviderUnavailable`] because the remedy is different and the
    /// user can act on it.
    DependencyUnavailable,
    /// Policy switched the rule off for this subject. A deliberate human choice, not a
    /// capability gap.
    ConfigurationDisabled,
    /// The subject could not be parsed. Nothing downstream of syntax was attempted.
    Unparseable,
    /// A provider ran and failed. Distinct from `Unparseable` because the input was
    /// well-formed and the fault is ours or the tool's.
    AnalysisFailed,
    /// The rule binds this subject and no mechanical provider can judge it; reaching a
    /// judgment needs a model.
    ///
    /// This is `CHK-003`'s seventh reporting category and it is deliberately not a
    /// capability gap. [`Applicability::MissingCapability`] tells a reader to install a
    /// provider, and no provider exists to install; leaving the subject out of coverage
    /// tells them nothing at all. The work is available and the executor for it is not a
    /// provider. `OD-CONTRACTS-002` records which of those two mis-filings this variant
    /// replaces.
    ///
    /// It says a model is *required*, never that one ran. What a model produced is
    /// [`crate::EvidenceClass::AgentJudged`], which
    /// [`crate::EvidenceClass::Is_Mechanical`] already refuses to report as a machine
    /// having checked something.
    AgentRequired,
}

impl Applicability
{
    /// The variant's stable `PascalCase` name, for display, diagnostics and wire form.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Supported => SUPPORTED_LABEL,
            Self::SupportedWithFallback => SUPPORTED_WITH_FALLBACK_LABEL,
            Self::PartiallySupported => PARTIALLY_SUPPORTED_LABEL,
            Self::NotApplicable => NOT_APPLICABLE_LABEL,
            Self::MissingCapability => MISSING_CAPABILITY_LABEL,
            Self::ProviderUnavailable => PROVIDER_UNAVAILABLE_LABEL,
            Self::DependencyUnavailable => DEPENDENCY_UNAVAILABLE_LABEL,
            Self::ConfigurationDisabled => CONFIGURATION_DISABLED_LABEL,
            Self::Unparseable => UNPARSEABLE_LABEL,
            Self::AnalysisFailed => ANALYSIS_FAILED_LABEL,
            Self::AgentRequired => AGENT_REQUIRED_LABEL,
        };
    }

    /// Whether a judgment was actually reached.
    ///
    /// This is **not** a pass. It says a rule ran and decided something; it says
    /// nothing about whether the subject conformed. A caller that treats this as a
    /// pass has reintroduced the defect this type exists to prevent.
    #[must_use]
    pub const fn Is_Evaluated(self) -> bool
    {
        return matches!(
            self,
            Self::Supported | Self::SupportedWithFallback | Self::PartiallySupported
        );
    }

    /// Whether this state represents analysis that was *wanted* and did not happen.
    ///
    /// [`Applicability::NotApplicable`] and [`Applicability::ConfigurationDisabled`]
    /// are excluded: the first is a positive statement that the rule does not bind, the
    /// second is a human decision. Everything else in this set is coverage debt, and a
    /// run that reports success while carrying it is lying.
    #[must_use]
    pub const fn Is_Coverage_Debt(self) -> bool
    {
        return matches!(
            self,
            Self::MissingCapability
                | Self::ProviderUnavailable
                | Self::DependencyUnavailable
                | Self::Unparseable
                | Self::AnalysisFailed
        );
    }

    /// Whether reaching a judgment about this subject needs a model.
    ///
    /// One place decides it, for the same reason [`Applicability::Is_Coverage_Debt`] is a
    /// method rather than a set each caller rebuilds. A consumer that wants the three
    /// answers apart asks the three predicates; none of them overlaps another.
    #[must_use]
    pub const fn Is_Agent_Required(self) -> bool
    {
        return matches!(self, Self::AgentRequired);
    }

    /// The presentation mapping clients use to render this state compactly.
    ///
    /// Clients localize the *label*; they never re-derive the *mapping*. Two clients
    /// disagreeing about whether `SupportedWithFallback` reads as "Native" is a defect
    /// this method exists to make impossible.
    #[must_use]
    pub const fn Display_Label(self) -> DisplayLabel
    {
        return match self
        {
            Self::Supported => DisplayLabel::Native,
            Self::SupportedWithFallback => DisplayLabel::Fallback,
            Self::PartiallySupported => DisplayLabel::Partial,
            Self::NotApplicable => DisplayLabel::NotApplicable,
            Self::MissingCapability
            | Self::ProviderUnavailable
            | Self::DependencyUnavailable
            | Self::ConfigurationDisabled => DisplayLabel::Unavailable,
            Self::Unparseable | Self::AnalysisFailed => DisplayLabel::Failed,
            Self::AgentRequired => DisplayLabel::AgentRequired,
        };
    }
}

impl core::fmt::Display for Applicability
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use alloc::vec::Vec;
    use super::*;

    /// Every state, once. A test that builds its own list checks the states it happened
    /// to remember, which is how a variant arrives unexamined.
    const ALL: [Applicability; 11] = [
        Applicability::Supported,
        Applicability::SupportedWithFallback,
        Applicability::PartiallySupported,
        Applicability::NotApplicable,
        Applicability::MissingCapability,
        Applicability::ProviderUnavailable,
        Applicability::DependencyUnavailable,
        Applicability::ConfigurationDisabled,
        Applicability::Unparseable,
        Applicability::AnalysisFailed,
        Applicability::AgentRequired,
    ];

    /// The whole point of the type. If this ever passes for a debt state, a run can
    /// report success having analyzed nothing.
    #[test]
    fn Test_Is_Coverage_Debt_Should_Never_Also_Read_As_Evaluated()
    {
        for state in Coverage_Debt_States()
        {
            assert!(state.Is_Coverage_Debt(), "{state} should be coverage debt");
            assert!(
                !state.Is_Evaluated(),
                "{state} must never read as evaluated"
            );
        }
    }

    /// The applicability states `Is_Coverage_Debt` must report `true` for.
    fn Coverage_Debt_States() -> [Applicability; 5]
    {
        return [
            Applicability::MissingCapability,
            Applicability::ProviderUnavailable,
            Applicability::DependencyUnavailable,
            Applicability::Unparseable,
            Applicability::AnalysisFailed,
        ];
    }

    /// A human switching a rule off, and a rule that does not bind, are decisions —
    /// not gaps. Counting them as debt would make every honest configuration look
    /// broken, which is how a real signal gets turned off.
    #[test]
    fn Test_Is_Coverage_Debt_Should_Exclude_Deliberate_Absences()
    {
        assert!(!Applicability::NotApplicable.Is_Coverage_Debt());
        assert!(!Applicability::ConfigurationDisabled.Is_Coverage_Debt());
    }

    /// `CHK-003`'s seventh category. Agent-required is work that is available and not
    /// done, so reading as evaluated would be a lie and reading as debt would send the
    /// reader to install a provider that does not exist.
    #[test]
    fn Test_Is_Agent_Required_Should_Be_True_Only_For_Agent_Required()
    {
        let state = Applicability::AgentRequired;

        assert!(state.Is_Agent_Required());
        assert!(!state.Is_Evaluated(), "a model has not run yet");
        assert!(!state.Is_Coverage_Debt(), "the work is available, not missing");

        for other in ALL
        {
            if other != Applicability::AgentRequired
            {
                assert!(!other.Is_Agent_Required(), "{other} should not require a model");
            }
        }
    }

    /// The three predicates partition nothing between them, and a state answering two of
    /// them would let one caller count a subject twice and another count it never.
    #[test]
    fn Test_No_State_Should_Answer_Two_Predicates()
    {
        for state in ALL
        {
            let answers = usize::from(state.Is_Evaluated())
                + usize::from(state.Is_Coverage_Debt())
                + usize::from(state.Is_Agent_Required());

            assert!(answers <= 1, "{state} answers {answers} predicates");
        }
    }

    /// Collapsing this into `Unavailable` is the mis-filing the variant exists to end,
    /// and the display layer is where it would come back unnoticed.
    #[test]
    fn Test_Display_Label_Should_Not_Collapse_Agent_Required_Into_Unavailable()
    {
        assert_eq!(
            Applicability::AgentRequired.Display_Label(),
            DisplayLabel::AgentRequired
        );

        for state in ALL
        {
            assert_eq!(
                state.Display_Label() == DisplayLabel::AgentRequired,
                state.Is_Agent_Required(),
                "{state} disagrees with its display label about needing a model"
            );
        }
    }

    #[test]
    fn Test_Is_Evaluated_Should_Be_True_For_Exactly_The_Three_Judged_States()
    {
        assert!(Applicability::Supported.Is_Evaluated());
        assert!(Applicability::SupportedWithFallback.Is_Evaluated());
        assert!(Applicability::PartiallySupported.Is_Evaluated());
        assert!(!Applicability::NotApplicable.Is_Evaluated());
        assert!(!Applicability::ConfigurationDisabled.Is_Evaluated());
    }

    /// A label is a wire value and a stable identity, not decoration. Renaming one is a
    /// protocol change, and this test is what makes that visible in review.
    #[test]
    fn Test_Label_Should_Be_Stable_And_Distinct_Per_State()
    {
        let mut seen: Vec<&str> = ALL.iter().map(|state| state.Label()).collect();
        let count = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), count, "two applicability states share a label");
    }

    /// `ALL` is a hand-written universe, and `OD-COMPLETENESS-001` records what one of
    /// those is worth on its own. The match below has no wildcard, so an eleventh state
    /// arriving stops the build here — beside the list it has to be added to. That is the
    /// guard; the assertion only catches a name written into the list twice.
    #[test]
    fn Test_Every_State_Should_Be_In_The_Tested_Universe()
    {
        for state in ALL
        {
            match state
            {
                Applicability::Supported
                | Applicability::SupportedWithFallback
                | Applicability::PartiallySupported
                | Applicability::NotApplicable
                | Applicability::MissingCapability
                | Applicability::ProviderUnavailable
                | Applicability::DependencyUnavailable
                | Applicability::ConfigurationDisabled
                | Applicability::Unparseable
                | Applicability::AnalysisFailed
                | Applicability::AgentRequired =>
                {}
            }
        }

        let mut distinct = ALL.to_vec();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), ALL.len(), "a state is listed twice in ALL");
    }
}
