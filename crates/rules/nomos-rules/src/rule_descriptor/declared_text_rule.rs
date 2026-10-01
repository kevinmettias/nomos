//! A rule of the dominant archetype, declared rather than written.
//!
//! Its own file rather than a second type beside [`super::RuleDescriptor`]: the
//! one-public-type rule wants one file-home type per file, and this one is the whole of
//! what a declared rule is.
//!
//! # What this is and what it is not
//!
//! `OD-RULES-034` censused all seventy-one composed rules and found one archetype worth a
//! form: a per-line predicate over one file's raw text, forty-four of the seventy-one. This
//! is that form. Its eight inputs are read off the engines this crate already has rather
//! than designed -- each one is a parameter some function here already takes -- and the
//! record's own test for the proposal was that a field with no existing caller would be a
//! field measured against nothing.
//!
//! It is deliberately not a rule intermediate representation, not an expression language and
//! not a manifest a repository ships. The first two are `OD-RULES-007`'s question, re-measured
//! at seventy-one and still unfired. The third cannot be `&'static`, so it would make the
//! composed rule set a run-time construction rather than a `const` table -- a real change to
//! `OD-RULES-027`'s decision, and one nothing in this population demands, since every rule
//! measured is one this workspace itself ships. The observation that would decide it is a
//! repository outside this workspace needing a rule this workspace does not ship.
//!
//! # What it may not express
//!
//! Each of these is a refusal rather than a case that renders empty, because a form that
//! silently cannot say something is worse than no form. Four are impossible by construction
//! and the fifth is checked:
//!
//! - **The detector itself.** [`DeclaredDetector`] is an enum of names holding no callable,
//!   no pattern and no data, so a declaration containing a detector does not compile.
//! - **A relation between subjects.** There is no field that could name another subject, and
//!   the interpreter's traversal is one source at a time: a mirror rule declared here would
//!   have to be written as a per-file loop that finds nothing, which is the exact failure
//!   this workspace ranks `Blocking` when a mirror claim resolves to nothing.
//! - **A judgment over a materialized payload.** [`Self::Requires`] is derived, not
//!   declared, and the only family it can name is the test-material policy that parameterizes
//!   the gate. No declaration can ask for a payload family, so none can judge one.
//! - **Block or scope state.** [`DeclaredDetector::As_Predicate`] sees one line's text and
//!   [`DeclaredJustification::As_Predicate`] sees the look-back the engine already does.
//!   Neither signature can carry brace state across lines.
//! - **Anything about which rules run.** There is no applicability field, and selection is
//!   `OD-HOST-004`'s question and `OD-RULES-022`'s resolution step. A form that declared its
//!   own applicability would be a second answer to it.
//!
//! The fifth refusal is [`Self::Is_Well_Formed`]: a declared parameter naming a detector that
//! takes none. That one cannot be made impossible by construction without deleting the field
//! `OD-RULES-034` names, so it is stated and checked instead -- twice, and the two catch
//! different things. `super::Descriptor_For_Declaration` asserts it in a `const` context, so
//! a malformed declaration composed into the table does not compile at all; and
//! `Test_Every_Declaration_Should_Be_Well_Formed` asserts it over every declaration this
//! crate states, which reaches one written but not yet composed.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

use super::{DeclaredDetector, DeclaredJustification, DeclaredParameter, RequiredFact, SubjectKind, TestMaterialSensitivity};

/// What a rule of the archetype states about itself, and the whole of it.
///
/// Public fields and no constructor, the same shape [`super::RuleDescriptor`] has and for the
/// same reason: a `const` table is written as literals, and a constructor taking eight inputs
/// would be this crate's own `parameter-count` rule broken by the type that declares rules.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeclaredTextRule
{
    /// The rule identifier, the same string a descriptor names.
    pub(crate) id: &'static str,
    /// The authority this rule's contract comes from, as [`super::RuleDescriptor`] means it.
    pub(crate) contract_record: &'static str,
    /// The version of [`Self::contract_record`], or `NO_VERSIONED_RECORD` for a prose one.
    pub(crate) contract_record_version: u32,
    /// The language `SourceFile::Is_Written_In` compares against, or `None` for every file.
    pub(crate) language: Option<&'static str>,
    /// Whether this rule's subject includes this repository's own test material.
    pub(crate) test_material: TestMaterialSensitivity,
    /// The implementation modules whose own text necessarily spells out what this rule looks
    /// for, and which it therefore does not judge.
    ///
    /// A field because it cannot be derived: a rule's detector constants spell the syntax the
    /// rule searches for, and `closure_bounds`' own doc says why no repository declaration
    /// could or should make that judgeable. Each entry is a repository-relative module path,
    /// matched the way `checks::rust_text`'s own exemption has always matched one -- the
    /// module's file or anything under its directory, never a bare suffix, so a stranger's
    /// repository ending the same way does not inherit the exemption.
    pub(crate) self_exemption: &'static [&'static str],
    /// The construct this rule judges, named from the closed vocabulary.
    pub(crate) detector: DeclaredDetector,
    /// What locally excuses the construct, or `None` where nothing does.
    pub(crate) justification: Option<DeclaredJustification>,
    /// The one sentence every finding of this rule carries.
    pub(crate) message: &'static str,
    /// The repository-declared value this rule's detector reads, where one does.
    pub(crate) parameter: Option<DeclaredParameter>,
}

/// What a declaration that reads no fact requires: nothing.
const NO_FACT: &[RequiredFact] = &[];

/// What a declaration that excludes test material requires, and the only family a
/// declaration can reach. The gate reads it; no declared judgment ever sees a payload.
const TEST_MATERIAL_POLICY_ONLY: &[RequiredFact] = &[RequiredFact::TestMaterialPolicy];

impl DeclaredTextRule
{
    /// Runs this declaration over `sources`, through the interpreter that owns the vocabulary.
    ///
    /// One line, and that is the seam: the declaration is data, the traversal and the finding
    /// shape are `checks::rust_text`'s, and the two meet here. `super::RuleJudgment::Judges`
    /// calls this and nothing else does, so a declared rule reaches a run through exactly the
    /// call path a linked one does.
    pub(crate) fn Judges(&self, sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
    {
        return crate::checks::Judged_By_Declaration(self, sources, facts);
    }

    /// What a run must have materialized before this rule can be judged.
    ///
    /// Derived from [`Self::test_material`] rather than declared, which is what keeps a
    /// declaration from asking for a payload family it could not read anyway.
    pub(crate) const fn Requires(&self) -> &'static [RequiredFact]
    {
        return match self.test_material
        {
            TestMaterialSensitivity::Judged => NO_FACT,
            TestMaterialSensitivity::Excluded => TEST_MATERIAL_POLICY_ONLY,
        };
    }

    /// What this rule reads, in the descriptor's own vocabulary.
    ///
    /// Derived from [`Self::Requires`] so the pair cannot disagree:
    /// `Test_A_Source_Text_Rule_Should_Require_No_Fact` and
    /// `Test_A_Fact_Reading_Rule_Should_Require_At_Least_One_Fact` judge a declared row
    /// exactly as they judge a linked one, and a derivation that could fail either would be
    /// a declaration wearing a descriptor it does not mean.
    pub(crate) const fn Subject(&self) -> SubjectKind
    {
        return match self.test_material
        {
            TestMaterialSensitivity::Judged => SubjectKind::SourceText,
            TestMaterialSensitivity::Excluded => SubjectKind::SourceFacts,
        };
    }

    /// Whether a test or example source is outside this rule's subject.
    pub(crate) const fn Is_Excluding_Test_Material(&self) -> bool
    {
        return matches!(self.test_material, TestMaterialSensitivity::Excluded);
    }

    /// Whether this declaration states something the vocabulary can mean.
    ///
    /// The one refusal that is not impossible by construction: a declared parameter is a
    /// policy key and a default, and no detector in the vocabulary reads one today, so a
    /// declaration naming a parameter would have its value resolved and dropped. That is the
    /// silent-failure shape `OD-RULES-034` forbids, so the declaration is refused instead --
    /// loudly, over the whole table, rather than at the one call site that would have been
    /// wrong.
    pub(crate) const fn Is_Well_Formed(&self) -> bool
    {
        if self.parameter.is_some()
        {
            return self.detector.Is_Taking_A_Parameter();
        }

        return true;
    }
}
