//! Whether a declared rule judges test material, as the declaration states it.
//!
//! Its own file rather than a `bool` field on [`super::DeclaredTextRule`]: a boolean would
//! leave every declaration site spelling `true` or `false` against a field name that reads
//! both ways, and this is the one input of the eight whose wrong value is invisible --
//! `OD-RULES-034` names the exemption as belonging to `every-allow-carries-a-justification`
//! and deliberately *not* to its two siblings, since a disabled test lives in a test source
//! by construction and exempting one would delete the rule rather than narrow it.

/// Whether a declared rule's subject includes this repository's own test material.
///
/// [`Self::Excluded`] is what `OD-RULES-011` made a repository-declared answer rather than
/// a fixed one: `checks::Is_Test_Or_Example_Source` extends its fixed clauses with the
/// locations `nomos.cap.test.material.policy` declares, so a declaration choosing this is
/// choosing the resolved criterion and not a path list of its own.
///
/// It is also what decides the descriptor a declaration derives. A rule that excludes test
/// material reads a fact to find out what that means, so it is `SubjectKind::SourceFacts`
/// requiring `RequiredFact::TestMaterialPolicy`; one that judges everything reads nothing at
/// all and is `SubjectKind::SourceText` requiring nothing --
/// [`super::DeclaredTextRule::Subject`] and [`super::DeclaredTextRule::Requires`] are the
/// whole of that derivation, and `Test_A_Source_Text_Rule_Should_Require_No_Fact` is what
/// keeps it honest against the rest of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TestMaterialSensitivity
{
    /// Test material is judged like any other source.
    Judged,
    /// A test or example source is not this rule's subject.
    Excluded,
}
