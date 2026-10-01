//! [`ComparabilityResponse`], what a comparison says a reader may conclude from it, as
//! [`super::gate_compare_response::GateCompareResponse`] carries it.

use super::judgment_difference_response::JudgmentDifferenceResponse;
use nomos_contracts::RunId;
use nomos_gate_orchestration::Comparability;
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_gate_orchestration::JudgmentDifference;

    /// How many inputs the stated difference below names.
    const STATED_DIFFERENCES: usize = 2;

    #[test]
    fn Test_Every_State_Should_Serialize_Under_A_Name_Of_Its_Own()
    {
        assert_eq!(Field_At(&Rendered_Comparability(&Comparability::Compatible), &["comparability"]), "compatible");
        assert_eq!(Field_At(&Rendered_Comparability(&Comparability::Incomparable(Vec::new())), &["comparability"]), "incomparable");
        let stated = Rendered_Comparability(&Comparability::CompatibleWith(vec![JudgmentDifference::Policy]));
        assert_eq!(Field_At(&stated, &["comparability"]), "compatible_with");
    }

    /// The differences are what a caller acts on, so they have to cross as discriminable
    /// values rather than as prose.
    #[test]
    fn Test_A_Stated_Difference_Should_Name_Each_Input_That_Differed()
    {
        let comparability = Comparability::CompatibleWith(vec![JudgmentDifference::Policy, JudgmentDifference::Instrument]);

        let rendered = Rendered_Comparability(&comparability);

        let differences = Field_At(&rendered, &["differences"]);
        assert_eq!(differences.as_array().map(Vec::len), Some(STATED_DIFFERENCES), "{rendered}");
        assert_eq!(differences.to_string(), "[\"policy\",\"instrument\"]", "{rendered}");
    }

    /// One field of a serialized value, by path.
    ///
    /// `serde_json::Value`'s own `Index` panics on a missing key, which is the failure
    /// `clippy::indexing_slicing` is denied in this workspace to prevent.
    fn Field_At(value: &serde_json::Value, path: &[&str]) -> serde_json::Value
    {
        const NOTHING: serde_json::Value = serde_json::Value::Null;

        let mut current = value;
        for step in path
        {
            current = current.get(step).unwrap_or(&NOTHING);
        }

        return current.clone();
    }

    /// A compatible comparison carries no difference list at all, rather than an empty one: a
    /// reader checking for differences must not have to tell `[]` from absent.
    #[test]
    fn Test_A_Compatible_Comparison_Should_Carry_No_Difference_List()
    {
        let rendered = Rendered_Comparability(&Comparability::Compatible);

        assert!(rendered.get("differences").is_none(), "{rendered}");
    }

    /// A comparability as a caller receives it.
    fn Rendered_Comparability(comparability: &Comparability) -> serde_json::Value
    {
        return serde_json::to_value(ComparabilityResponse::From(comparability))
            .expect("a derived Serialize over owned data has nothing to refuse");
    }
}
