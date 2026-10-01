//! [`BaselineAllowanceResponse`], how much debt one
//! [`super::baseline_debt_response::BaselineDebtResponse`] accepted.

use nomos_gate_orchestration::BaselineAllowance;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::BaselineAllowance`].
///
/// A twin rather than a re-export for the reason [`super`]'s own doc gives for every other one
/// in this module.
///
/// # Why a tagged state rather than a nullable number
///
/// The obvious wire shape is `accepted_occurrence_count: number | null`, and it is the wrong
/// one. `OD-GATE-030` decides that an entry naming no count tolerates *without limit*, which is
/// the strongest reading of the three a reader might give a `null` — the other two being zero
/// and one, and both of them wrong in the direction that blocks a build over debt a repository
/// did adopt. A caller that has to infer which of those a missing field meant will eventually
/// infer the convenient one. Naming the state removes the inference: `unbounded` is a word on
/// the wire, and a bounded entry is the only shape that carries a number at all.
#[derive(Debug, Serialize)]
#[serde(tag = "allowance", rename_all = "snake_case")]
pub enum BaselineAllowanceResponse
{
    /// The entry names no count, so it tolerates however many occurrences its scope holds.
    Unbounded,
    /// The entry accepted at most `accepted_occurrence_count` occurrences.
    AtMost
    {
        /// The quantity adopted. Never zero -- the declared reader refuses that to its author.
        accepted_occurrence_count: u32,
    },
}

impl BaselineAllowanceResponse
{
    pub(crate) const fn From(allowance: BaselineAllowance) -> Self
    {
        return match allowance
        {
            BaselineAllowance::Unbounded => Self::Unbounded,
            BaselineAllowance::AtMost(count) => Self::AtMost { accepted_occurrence_count: count },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The occurrence count a bounded entry adopts in these tests.
    const ADOPTED_OCCURRENCES: u32 = 3;

    /// Each state crosses under a name of its own, so a caller branches on a word.
    #[test]
    fn Test_Each_State_Should_Serialize_Under_A_Name_Of_Its_Own()
    {
        let unbounded = Rendered_Allowance(BaselineAllowance::Unbounded);
        let bounded = Rendered_Allowance(BaselineAllowance::AtMost(ADOPTED_OCCURRENCES));

        assert_eq!(unbounded.get("allowance").and_then(serde_json::Value::as_str), Some("unbounded"), "{unbounded}");
        assert_eq!(bounded.get("allowance").and_then(serde_json::Value::as_str), Some("at_most"), "{bounded}");
        assert_eq!(bounded.get("accepted_occurrence_count").and_then(serde_json::Value::as_u64), Some(u64::from(ADOPTED_OCCURRENCES)), "{bounded}");
    }

    /// An unbounded entry carries no count at all, rather than a zero or a null.
    ///
    /// The whole reason this is a tagged state: a caller checking the quantity must not have to
    /// tell "no limit" from "a limit of nothing", which is the confusion a nullable number
    /// leaves in place.
    #[test]
    fn Test_An_Unbounded_Allowance_Should_Carry_No_Count()
    {
        let rendered = Rendered_Allowance(BaselineAllowance::Unbounded);

        assert!(rendered.get("accepted_occurrence_count").is_none(), "{rendered}");
    }

    /// An allowance as a caller receives it.
    fn Rendered_Allowance(allowance: BaselineAllowance) -> serde_json::Value
    {
        return serde_json::to_value(BaselineAllowanceResponse::From(allowance))
            .expect("a derived Serialize over owned data has nothing to refuse");
    }
}
