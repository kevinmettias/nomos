//! [`ComparabilityResponse`], what a comparison says a reader may conclude from it, as
//! [`super::gate_compare_response::GateCompareResponse`] carries it.

use nomos_contracts::RunId;
use nomos_gate_orchestration::{Comparability, JudgmentDifference};
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::Comparability`].
///
/// A twin rather than a re-export for the reason [`super`]'s own doc gives for every other one
/// in this module.
///
/// # Why a headless caller needs this
///
/// `OD-GATE-031`: a difference between two runs may be attributed to repository state only
/// when the non-source judgment inputs were compatible, or their differences are explicitly
/// represented. A wire response reporting `added`, `removed` and `changed` with no word about
/// what judged either side is read as a fact about the repository, and a continuous
/// enforcement loop is exactly the caller that will act on that reading without a person in
/// between.
#[derive(Debug, Serialize)]
#[serde(tag = "comparability", rename_all = "snake_case")]
pub enum ComparabilityResponse
{
    /// Both runs recorded what judged them, and their policy, selection and instrument all
    /// agree. The difference reported alongside is a difference in the repository.
    Compatible,
    /// Both runs recorded what judged them, and something other than the source differed too.
    ///
    /// Not a warning that anything is wrong: comparing one tree under two policies is a real
    /// thing to want, and this is the fact that makes the difference readable rather than
    /// misleading.
    CompatibleWith
    {
        /// What differed, never empty — an empty list is [`Self::Compatible`].
        differences: Vec<JudgmentDifferenceResponse>,
    },
    /// At least one run does not say what judged it, so nothing can be attributed either way.
    Incomparable
    {
        /// The runs that did not say.
        unknown: Vec<RunId>,
    },
}

/// One judgment input, other than the source, that differed between two runs.
///
/// In this file rather than one of its own, which the module's rule about twins is read
/// against deliberately: that rule is about a type serializing one field of a *response*, and
/// this serializes a field of another twin. Splitting it would put a three-variant enum with
/// no independent meaning in a file that nothing but its sibling would ever open.
///
/// The source is absent, and that is the point of the whole type: a difference in source is
/// the licensed cause a comparison exists to attribute a change to, not a caveat on doing so.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgmentDifferenceResponse
{
    /// The two runs judged under different policies. The likeliest of the three and the least
    /// visible: a run resolves its policy from its own root, and a comparison judges two.
    Policy,
    /// The two runs were allowed to look at different things. A run told to look at less has
    /// fewer findings for that reason, and its absent findings otherwise read as the other
    /// run's additions.
    Selection,
    /// The two runs were judged by different builds or different rule sets. This workspace
    /// declares its analysis kernel reproducible `CrossPlatform`, which is strictly weaker
    /// than `CrossBinary`, so agreement across two instruments is not something it claims.
    Instrument,
}

impl ComparabilityResponse
{
    pub(crate) fn From(comparability: &Comparability) -> Self
    {
        return match comparability
        {
            Comparability::Compatible => Self::Compatible,
            Comparability::CompatibleWith(differences) => Self::CompatibleWith {
                differences: differences.iter().map(|difference| return JudgmentDifferenceResponse::From(*difference)).collect(),
            },
            Comparability::Incomparable(unknown) => Self::Incomparable { unknown: unknown.clone() },
        };
    }
}

impl JudgmentDifferenceResponse
{
    pub(crate) const fn From(difference: JudgmentDifference) -> Self
    {
        return match difference
        {
            JudgmentDifference::Policy => Self::Policy,
            JudgmentDifference::Selection => Self::Selection,
            JudgmentDifference::Instrument => Self::Instrument,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// One field of a serialized value, by path.
    ///
    /// `serde_json::Value`'s own `Index` panics on a missing key, which is the failure
    /// `clippy::indexing_slicing` is denied in this workspace to prevent.
    fn At(value: &serde_json::Value, path: &[&str]) -> serde_json::Value
    {
        const NOTHING: serde_json::Value = serde_json::Value::Null;

        let mut current = value;
        for step in path
        {
            current = current.get(step).unwrap_or(&NOTHING);
        }

        return current.clone();
    }

    fn Rendered(comparability: &Comparability) -> serde_json::Value
    {
        return serde_json::to_value(ComparabilityResponse::From(comparability)).expect("always serializes");
    }

    #[test]
    fn Test_Every_State_Should_Serialize_Under_A_Name_Of_Its_Own()
    {
        assert_eq!(At(&Rendered(&Comparability::Compatible), &["comparability"]), "compatible");
        assert_eq!(At(&Rendered(&Comparability::Incomparable(Vec::new())), &["comparability"]), "incomparable");
        let stated = Rendered(&Comparability::CompatibleWith(vec![JudgmentDifference::Policy]));
        assert_eq!(At(&stated, &["comparability"]), "compatible_with");
    }

    /// The differences are what a caller acts on, so they have to cross as discriminable
    /// values rather than as prose.
    #[test]
    fn Test_A_Stated_Difference_Should_Name_Each_Input_That_Differed()
    {
        let comparability = Comparability::CompatibleWith(vec![JudgmentDifference::Policy, JudgmentDifference::Instrument]);

        let rendered = Rendered(&comparability);

        let differences = At(&rendered, &["differences"]);
        assert_eq!(differences.as_array().map(Vec::len), Some(2), "{rendered}");
        assert_eq!(differences.to_string(), "[\"policy\",\"instrument\"]", "{rendered}");
    }

    /// A compatible comparison carries no difference list at all, rather than an empty one: a
    /// reader checking for differences must not have to tell `[]` from absent.
    #[test]
    fn Test_A_Compatible_Comparison_Should_Carry_No_Difference_List()
    {
        let rendered = Rendered(&Comparability::Compatible);

        assert!(rendered.get("differences").is_none(), "{rendered}");
    }
}
