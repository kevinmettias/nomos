//! The three rules of this family that are declared rather than written.
//!
//! Split out of [`super`], which states the family's shared reasoning, and holding only
//! literals: each constant here is the whole of one rule. `OD-RULES-034` decided the form
//! and named these three's archetype as the one it serves — a per-line predicate over one
//! file's raw text, forty-four of this workspace's seventy-one composed rules.
//!
//! Read one against the function it replaced and the form's claim is visible: every line of
//! those functions was either the gate (`Is_Written_In`, `Is_Test_Or_Example_Source`, the
//! self-exemption), the traversal, or the finding — all of which
//! [`super::Judged_By_Declaration`] now does once — leaving the detector, the justification,
//! the message and the identity, which is exactly what a declaration states.
//!
//! What is *not* here is any detector. A declaration names one from
//! [`DeclaredDetector`]'s closed vocabulary; it cannot contain one, and a rule needing a
//! reading the vocabulary does not have is a Rust change. That refusal is the record's
//! important one: the archetype's forty-four detectors include a substring test, a matcher
//! that returns its match, a documentation-comment range parser and an integer-width
//! computation, and a form that let the last three be written but not meant would be worse
//! than no form.

use crate::rule_descriptor::{DeclaredDetector, DeclaredJustification, DeclaredTextRule, TestMaterialSensitivity};
use crate::rule_descriptor::{NO_VERSIONED_RECORD, PORTED_STANDARD};

use super::{A_DISABLED_TEST_STATES_WHY, EVERY_ALLOW_CARRIES_A_JUSTIFICATION, INLINE_ALWAYS_JUSTIFICATION, OWN_IMPLEMENTATION_MODULE};

/// This family's own implementation module, as a declaration states a self-exemption.
///
/// One list shared by all three, because it is one exemption: [`OWN_IMPLEMENTATION_MODULE`]'s
/// own doc says why every rule under it exempts it, and a declaration naming its own copy of
/// the same path would be that reasoning written twice.
const OWN_MODULE: &[&str] = &[OWN_IMPLEMENTATION_MODULE];

/// `every-allow-carries-a-justification`, declared.
///
/// The one of the three that excludes test material, and deliberately so: a test suppresses
/// a lint to construct the shape it is testing, and the justification the rule asks for is
/// the test's own name. `TestMaterialSensitivity::Excluded` is what carries that here, and
/// it is also what makes this row `SubjectKind::SourceFacts` requiring the test-material
/// policy — the same descriptor the linked function's row declared by hand.
pub(crate) const EVERY_ALLOW_DECLARATION: DeclaredTextRule = DeclaredTextRule {
    id: EVERY_ALLOW_CARRIES_A_JUSTIFICATION,
    contract_record: PORTED_STANDARD,
    contract_record_version: NO_VERSIONED_RECORD,
    language: Some(crate::RUST_LANGUAGE),
    test_material: TestMaterialSensitivity::Excluded,
    self_exemption: OWN_MODULE,
    detector: DeclaredDetector::AllowAttribute,
    justification: Some(DeclaredJustification::AnyAdjacentComment),
    message: "carries an #[allow(...)] with no adjacent comment explaining why",
    parameter: None,
};

/// `inline-always-requires-justification`, declared.
///
/// `TestMaterialSensitivity::Judged`, and that is the deliberate half of the pair above:
/// `#[inline(always)]` means the same thing wherever it is written.
pub(crate) const INLINE_ALWAYS_DECLARATION: DeclaredTextRule = DeclaredTextRule {
    id: INLINE_ALWAYS_JUSTIFICATION,
    contract_record: PORTED_STANDARD,
    contract_record_version: NO_VERSIONED_RECORD,
    language: Some(crate::RUST_LANGUAGE),
    test_material: TestMaterialSensitivity::Judged,
    self_exemption: OWN_MODULE,
    detector: DeclaredDetector::InlineAlwaysAttribute,
    justification: Some(DeclaredJustification::AnyAdjacentComment),
    message: "carries #[inline(always)] with no adjacent comment explaining why",
    parameter: None,
};

/// `a-disabled-test-states-why`, declared.
///
/// `TestMaterialSensitivity::Judged` for the sharper reason of the two: a disabled test is
/// in a test source by construction, so excluding test material here would delete the rule
/// rather than narrow it.
pub(crate) const A_DISABLED_TEST_DECLARATION: DeclaredTextRule = DeclaredTextRule {
    id: A_DISABLED_TEST_STATES_WHY,
    contract_record: PORTED_STANDARD,
    contract_record_version: NO_VERSIONED_RECORD,
    language: Some(crate::RUST_LANGUAGE),
    test_material: TestMaterialSensitivity::Judged,
    self_exemption: OWN_MODULE,
    detector: DeclaredDetector::BareIgnoreAttribute,
    justification: Some(DeclaredJustification::AnyAdjacentComment),
    message: "disables a test with a bare #[ignore] and no reason",
    parameter: None,
};

#[cfg(test)]
mod tests;
