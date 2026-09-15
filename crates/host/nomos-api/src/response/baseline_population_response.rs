//! [`BaselinePopulationResponse`], what one baselined scope accepted against what a run found.

use super::baseline_allowance_response::BaselineAllowanceResponse;
use nomos_contracts::{RuleId, SubjectId};
use nomos_gate_orchestration::BaselinePopulation;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::BaselinePopulation`].
///
/// A twin rather than a re-export for the reason [`super`]'s own doc gives for every other one
/// in this module.
///
/// # Why a headless caller needs this
///
/// `OD-GATE-030`: a baseline may tolerate no more than the quantity it adopted, and a
/// population above that is a baseline expansion the baseline must not hide. A response that
/// reported only which findings blocked would leave a continuous-enforcement caller unable to
/// tell a rule nobody had addressed from a tolerance that ran out of room — and those call for
/// different actions, since the second means the repository's debt grew under a tolerance
/// somebody granted deliberately.
///
/// # What this is not evidence of
///
/// `observed` at or below `allowed` does **not** say the adopted occurrences persisted, and an
/// `excess` does **not** identify which occurrences are new. Both are counting facts about a
/// scope. A caller that reads either as occurrence history has read more than the run knows;
/// `OD-GATE-030` records why nothing here can answer that yet.
#[derive(Debug, Serialize)]
pub struct BaselinePopulationResponse
{
    /// The rule whose occurrences this scope counts.
    pub rule: RuleId,
    /// The subject whose occurrences this scope counts.
    pub subject: SubjectId,
    /// The path the declared entry was written with, or `null` when no file declared it.
    ///
    /// Display material for the caller's own message, and never identity: `subject` is what a
    /// run matched on. It is carried because the mapping from a path to a subject is one way,
    /// so a caller given only `subject` can print a digest its reader cannot trace back to the
    /// `nomos-gate.json` entry they wrote -- which is the one thing such a reader needs in
    /// order to repair it.
    ///
    /// `null` means there is no authored spelling, not that one was withheld. A caller
    /// rendering a message falls back to `subject` there, and the two cases are worth keeping
    /// apart: a path invented for an entry that named none would send a reader looking for a
    /// line that does not exist.
    pub declared_path: Option<String>,
    /// What the declared entry accepted.
    pub allowed: BaselineAllowanceResponse,
    /// How many occurrences this run found in the scope.
    pub observed: u32,
    /// How far past its allowance this scope is; zero when within one, and zero when the entry
    /// named no allowance.
    ///
    /// Carried as a field although the domain type derives it, because a wire consumer cannot
    /// call a method and the alternative is every caller reimplementing the subtraction —
    /// including the saturating case that keeps a scope inside its allowance from reporting a
    /// negative excess as an enormous positive one.
    pub excess: u32,
}

impl BaselinePopulationResponse
{
    pub(crate) fn From(population: BaselinePopulation) -> Self
    {
        let excess = population.Excess();

        return Self {
            rule: population.rule,
            subject: population.subject,
            declared_path: population.declared_path,
            allowed: BaselineAllowanceResponse::From(population.allowed),
            observed: population.observed,
            excess,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_gate_orchestration::BaselineAllowance;

    fn Population(allowed: BaselineAllowance, observed: u32) -> serde_json::Value
    {
        return Rendered(Some("./src/lib.rs"), allowed, observed);
    }

    fn Rendered(declared_path: Option<&str>, allowed: BaselineAllowance, observed: u32) -> serde_json::Value
    {
        let population = BaselinePopulation {
            rule: RuleId::New("no-single-line-function-bodies"),
            subject: nomos_model::Subject_Of_Path("src/lib.rs"),
            declared_path: declared_path.map(str::to_owned),
            allowed,
            observed,
        };

        return serde_json::to_value(BaselinePopulationResponse::From(population)).expect("always serializes");
    }

    /// The scope reaches a headless caller as its author wrote it, so that caller's message can
    /// name the entry the person reading it has to repair.
    ///
    /// Without this the wire carries a digest and nothing else, and the digest is the identity
    /// rather than a spelling anyone can look up -- which is the same defect the terminal
    /// report had, one surface over.
    #[test]
    fn Test_A_Declared_Scope_Should_Cross_The_Wire_As_Its_Author_Wrote_It()
    {
        let rendered = Rendered(Some("./src/lib.rs"), BaselineAllowance::AtMost(1), 5);

        assert_eq!(rendered.get("declared_path").and_then(serde_json::Value::as_str), Some("./src/lib.rs"), "{rendered}");
    }

    /// An entry no file declared crosses as an explicit `null` rather than as a path somebody
    /// invented for it.
    ///
    /// `null` is a claim about the entry -- there is no authored spelling -- and it is the one
    /// a caller needs in order to fall back to `subject` deliberately rather than by accident.
    #[test]
    fn Test_A_Scope_No_File_Declared_Should_Cross_The_Wire_With_No_Path_At_All()
    {
        let rendered = Rendered(None, BaselineAllowance::AtMost(1), 5);

        assert_eq!(rendered.get("declared_path"), Some(&serde_json::Value::Null), "{rendered}");
    }

    /// The defect this whole line of work is about, as a caller reads it: five present, one
    /// adopted, four that cannot be the adopted one.
    #[test]
    fn Test_An_Exceeded_Scope_Should_Carry_What_It_Accepted_What_It_Found_And_The_Difference()
    {
        let rendered = Population(BaselineAllowance::AtMost(1), 5);

        assert_eq!(rendered.pointer("/allowed/accepted_occurrence_count").and_then(serde_json::Value::as_u64), Some(1), "{rendered}");
        assert_eq!(rendered.get("observed").and_then(serde_json::Value::as_u64), Some(5), "{rendered}");
        assert_eq!(rendered.get("excess").and_then(serde_json::Value::as_u64), Some(4), "{rendered}");
    }

    /// A scope inside its allowance reports no excess rather than a wrapped one.
    #[test]
    fn Test_A_Scope_Within_Its_Allowance_Should_Report_No_Excess()
    {
        let rendered = Population(BaselineAllowance::AtMost(5), 2);

        assert_eq!(rendered.get("excess").and_then(serde_json::Value::as_u64), Some(0), "{rendered}");
    }

    /// An unbounded scope is never exceeded, however many occurrences it holds, and says so
    /// with a word rather than by omitting the field.
    #[test]
    fn Test_An_Unbounded_Scope_Should_Report_No_Excess_And_Name_Itself()
    {
        let rendered = Population(BaselineAllowance::Unbounded, 900);

        assert_eq!(rendered.pointer("/allowed/allowance").and_then(serde_json::Value::as_str), Some("unbounded"), "{rendered}");
        assert_eq!(rendered.get("observed").and_then(serde_json::Value::as_u64), Some(900), "{rendered}");
        assert_eq!(rendered.get("excess").and_then(serde_json::Value::as_u64), Some(0), "{rendered}");
    }
}
