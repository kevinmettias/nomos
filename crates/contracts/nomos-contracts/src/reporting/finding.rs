//! What a rule says about one subject.
//!
//! The five honesty vocabularies each answer one question. A finding is where several
//! of those answers arrive together, about one subject, from one rule — and the reason
//! it is a type rather than a formatted line is that every one of those answers is lost
//! the moment it becomes prose. A printed `warning: X is unmirrored` cannot be asked
//! whether the rule actually reached its subject, how the claim was come by, or whether
//! anything would have failed a build over it. Those three questions are the difference
//! between a report and a wish, and the prototypes lost all three by printing.
//!
//! # Identity is not a location
//!
//! [`Finding::subject`] is a digest, and [`Finding::locations`] is where to look. The
//! two are not interchangeable and the ordering matters: a universe that moves to a
//! different file is the same universe, and a finding keyed on its path would read as a
//! finding closed and a new finding opened. `identity.rs` states the rule for the whole
//! system — nothing is identified by its path and line, ever — and a finding is the
//! type most likely to break it, because a location is the field a human wants first.

// What a finding says beyond its text: whether the rule applied, what class of evidence
// backs it, and how each of those is labelled for a reader.
mod applicability;
mod display_label;
mod evidence_class;

pub use applicability::Applicability;
pub use display_label::DisplayLabel;
pub use evidence_class::EvidenceClass;

use crate::GateCategory;
use crate::{RuleId, SubjectId};
use serde::{Deserialize, Serialize};

/// One rule's judgment about one subject.
///
/// Every field is load-bearing, and the ones that look redundant are the ones that are
/// not:
///
/// - `applicability` is not implied by the existence of a finding. A rule that reached
///   only part of its subject has found something real and has *not* cleared the rest,
///   and [`Applicability::PartiallySupported`] is the only way to say so. Dropping this
///   field makes a partial judgment indistinguishable from a complete one.
/// - `evidence` is not implied by the rule being mechanical. A rule may derive one
///   finding from source it read and another from a declaration it took on trust, and
///   those are not the same claim.
/// - `gate` is not implied by severity. What a violation *deserves* and what the wiring
///   would actually *do* about it are the two halves `enforcement.rs` exists to keep
///   apart, and a finding that only carries the first one teaches a reader that
///   something is enforced when nothing runs.
///
/// There is no severity field and no `is_error`. A consumer that wants a red build asks
/// [`Finding::Can_Fail_A_Build`], which consults the gate and the applicability
/// together, because a finding from a rule that never reached its subject must not fail
/// anything.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding
{
    /// The rule that produced this.
    pub rule: RuleId,
    /// Stable identity of what the finding is about.
    ///
    /// Derived from [`Finding::subject_name`], never from a path. Two findings about
    /// the same subject in different files carry the same identity, which is what makes
    /// them comparable across revisions.
    pub subject: SubjectId,
    /// The human-authored name the identity was derived from.
    ///
    /// Not a duplicate of anything: it is the preimage of `subject`, so the two cannot
    /// disagree without the digest being computed from something else. It is here
    /// because a digest alone cannot be read, and a finding nobody can read is a finding
    /// nobody acts on.
    pub subject_name: String,
    /// Whether the rule actually reached this subject, and if not, why not.
    pub applicability: Applicability,
    /// How this claim was come by.
    pub evidence: EvidenceClass,
    /// What the wiring would really do about a violation of this rule.
    pub gate: GateCategory,
    /// What is wrong, in terms the author can act on.
    pub summary: String,
    /// Where to look. Repo-relative, forward slashes. Never identity.
    pub locations: Vec<String>,
}

impl Finding
{
    /// Whether this finding may fail a build.
    ///
    /// Two conditions, and the second one is the one that gets forgotten. The gate must
    /// be able to fail a build, *and* the rule must have reached its subject: a finding
    /// raised by a rule that could not read what it binds is a report about the
    /// analysis, not about the code, and failing a build over it teaches everybody to
    /// pass `--no-verify`.
    #[must_use]
    pub const fn Can_Fail_A_Build(&self) -> bool
    {
        return self.gate.Can_Fail_A_Build() && self.applicability.Was_Evaluated();
    }

    /// Whether this finding rests on a mechanism rather than on somebody's word.
    #[must_use]
    pub const fn Is_Mechanical(&self) -> bool
    {
        return self.evidence.Is_Mechanical();
    }

    /// A one-line rendering: what, where, and under whose authority.
    ///
    /// The gate category is in the line on purpose. A reader scanning output must be
    /// able to see which lines can stop them and which are being reported at them, and
    /// the prototype's finding output could not say.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        let where_to_look = if self.locations.is_empty()
        {
            "no location".to_owned()
        }
        else
        {
            self.locations.join(", ")
        };

        return format!(
            "[{}] {}: {} ({}) — {where_to_look}",
            self.gate.Label(),
            self.rule,
            self.subject_name,
            self.summary
        );
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Digest128;

    #[test]
    fn Test_Can_Fail_A_Build_Should_Be_True_For_An_Evaluated_Blocking_Finding()
    {
        assert!(Example_Finding(Applicability::Supported, GateCategory::Blocking).Can_Fail_A_Build());
    }

    /// The one that gets forgotten. A rule that could not read what it binds has
    /// reported on the analysis, not on the code, and a build failed over that is a
    /// build everybody learns to bypass.
    #[test]
    fn Test_A_Finding_From_A_Rule_That_Never_Ran_Should_Not_Fail_A_Build()
    {
        for unreached in Unreached_Applicability_States()
        {
            assert!(
                !Example_Finding(unreached, GateCategory::Blocking).Can_Fail_A_Build(),
                "{} reached nothing and must not fail a build",
                unreached.Label()
            );
        }
    }

    /// The applicability states that mean a rule never reached its subject.
    fn Unreached_Applicability_States() -> [Applicability; 5]
    {
        return [
            Applicability::MissingCapability,
            Applicability::ProviderUnavailable,
            Applicability::DependencyUnavailable,
            Applicability::Unparseable,
            Applicability::AnalysisFailed,
        ];
    }

    /// A partial judgment is still a judgment. What it did see, it saw.
    #[test]
    fn Test_A_Partial_Judgment_Should_Still_Fail_A_Build()
    {
        assert!(
            Example_Finding(Applicability::PartiallySupported, GateCategory::Blocking).Can_Fail_A_Build()
        );
    }

    /// The whole point of `enforcement.rs`, restated at the finding level: a real defect
    /// reported through wiring that cannot fail anything must not read as enforcement.
    #[test]
    fn Test_A_Real_Finding_Behind_A_Gate_That_Cannot_Fail_Should_Not_Fail_A_Build()
    {
        for toothless in Toothless_Gate_Categories()
        {
            assert!(!Example_Finding(Applicability::Supported, toothless).Can_Fail_A_Build());
        }
    }

    /// The gate categories that cannot fail a build no matter what a rule found.
    fn Toothless_Gate_Categories() -> [GateCategory; 3]
    {
        return [
            GateCategory::Advisory,
            GateCategory::Unreachable,
            GateCategory::Review,
        ];
    }

    /// A reader scanning output must be able to tell which lines can stop them.
    #[test]
    fn Test_Describe_Should_Name_The_Gate_The_Rule_And_The_Place()
    {
        let described = Example_Finding(Applicability::Supported, GateCategory::Blocking).Describe();

        assert!(described.contains("Blocking"), "{described}");
        assert!(described.contains("completeness-mirror"), "{described}");
        assert!(described.contains("Table::All"), "{described}");
        assert!(described.contains("store.rs"), "{described}");
    }

    /// A finding with nowhere to look says so rather than rendering an empty field, which
    /// reads as a location of "".
    #[test]
    fn Test_Describe_Should_Say_No_Location_When_None_Is_Recorded()
    {
        let mut nowhere = Example_Finding(Applicability::Supported, GateCategory::Blocking);
        nowhere.locations.clear();

        assert!(nowhere.Describe().contains("no location"));
    }

    /// `Finding::Is_Mechanical` is a pure delegation to `self.evidence.Is_Mechanical`,
    /// which `evidence_class.rs` tests across every variant; this pins the delegation
    /// itself rather than the axis it reads.
    #[test]
    fn Test_Is_Mechanical_Should_Reflect_The_Finding_s_Evidence_Class()
    {
        assert!(Example_Finding(Applicability::Supported, GateCategory::Blocking).Is_Mechanical());
    }

    fn Example_Finding(applicability: Applicability, gate: GateCategory) -> Finding
    {
        return Finding {
            rule: RuleId::New("completeness-mirror"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([7; Digest128::BYTE_LENGTH])),
            subject_name: "Table::All".to_owned(),
            applicability,
            evidence: EvidenceClass::Derived,
            gate,
            summary: "declares no mirror".to_owned(),
            locations: vec!["crates/spec/nomos-spec-store/src/store.rs".to_owned()],
        };
    }
}
