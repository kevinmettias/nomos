//! A source the run could not read, and the findings that say so.

use super::{SourceFile, Finding, RuleId, COMPLETENESS_MIRROR, SubjectId, Content_Digest, Applicability, EvidenceClass, GateCategory};

/// A file the rule could not read.
///
/// Identified by the digest of its contents rather than by its name, because what could
/// not be parsed is this text — the same path holding different bytes is a different
/// fact, and a finding keyed on the name would look unchanged after an edit that fixed
/// it.
pub(super) fn Unreadable(source: &SourceFile, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(source.text.as_bytes())),
        subject_name: source.path.clone(),
        applicability: Applicability::Unparseable,
        evidence: EvidenceClass::Derived,
        // Advisory, and it would make no difference if it were not: `Can_Fail_A_Build`
        // consults the applicability too, and a rule that never read its subject must not
        // stop anybody. What this finding is for is being *counted* — a run that could not
        // read four files and reports clean is the defect, not the four files.
        gate: GateCategory::Advisory,
        summary: format!("could not be parsed, so no universe in it was judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

/// One subject whose check names could not be read, and why not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Unread<'source>
{
    /// Where to look, for the reader. Reporting only.
    pub(super) path: String,
    /// The subject's own text, borrowed from the sources the rule was handed.
    ///
    /// Not a second source of check names, and it must never become one — the floor
    /// [`crate::Syntax_Requirement`] states exists to stop exactly that, and
    /// [`Unread::Could_Have_Declared`] is the only thing that reads this field.
    pub(super) text: &'source str,
    /// The digest of the text the answer was asked about.
    ///
    /// The finding's identity. Not the [`SubjectId`] the fact was requested under: that is
    /// a digest of a path, and `identity.rs` states the rule for the whole system — a
    /// finding keyed on where something lives closes and reopens every time somebody moves
    /// a file. What could not be read is *these bytes*, which is what [`Unreadable`] keys
    /// on for the same reason one paragraph up.
    pub(super) inputs: SubjectId,
    /// What the reader said, in the vocabulary a run reports in.
    pub(super) applicability: Applicability,
    /// The specific reason, for the summary.
    pub(super) because: String,
}

impl Unread<'_>
{
    /// Whether any reading of this subject could have declared `name`.
    ///
    /// This is the whole of `OD-RULES-002`, so it is worth being exact about what it claims
    /// and in which direction it is allowed to be wrong.
    ///
    /// A check name in the index is an identifier the provider read out of a token stream,
    /// and an identifier in a token stream is spelled in the bytes the stream was lexed
    /// from. So a subject whose text does not contain `name` as a substring cannot have
    /// declared it, whatever the provider would have said about the rest of the file. That
    /// direction is sound, and it is the direction this function is used in: it decides when
    /// the index is short **of something that could have resolved this claim**.
    ///
    /// The converse is not claimed and is not needed. A substring match means only that the
    /// name is spelled somewhere in the file — in a comment, in a string literal, inside a
    /// `Test_X_And_More` — and every one of those is a false positive on "could have
    /// declared it". Each of those false positives *withholds* a block. That is why an
    /// unsound text test is admissible here and inadmissible thirty lines up:
    /// `Check_Index_Of` uses a fact for the check names because an unsound *positive* there
    /// resolves a claim and silences the rule, which is the defect this rule exists to
    /// find. Used only to add doubt, the same signal cannot manufacture a phantom — it can
    /// only fail to establish one, and `Test_A_Name_Spelled_In_A_Comment_Should_Still_\
    /// Withhold_The_Block` pins the shape.
    ///
    /// The one way this can be wrong in the *unsafe* direction is a name no reading of the
    /// bytes spells: an identifier a macro composed, or one behind an `include!`. That hole
    /// is not opened here. It is `Assurance::Unknown` completeness, which
    /// [`crate::Syntax_Requirement`] already declares and this rule already blocks under — a
    /// macro-generated `Test_X` in a *readable* file is equally absent from the index and
    /// equally reported as a phantom today. The guarantee is therefore exact with respect to
    /// what the source spells and no better, which is the same bound the rest of the rule
    /// carries rather than a new one.
    pub(super) fn Could_Have_Declared(&self, name: &str) -> bool
    {
        return self.text.contains(name);
    }
}

/// A subject whose check names are missing from the index.
///
/// Advisory, and it would make no difference if it were not: `Can_Fail_A_Build` consults
/// the applicability too, and every value this finding can carry says the rule did not
/// reach its subject. What this finding is for is being *counted* — a run that could not
/// read a fact for four files and reports clean is the defect, not the four files.
///
/// A file the text side also failed to parse produces two findings rather than one, and
/// that is deliberate. Two independent readings of the same file both refused, which is a
/// different observation from either one alone — and while `nomos-rules` keeps a `syn`
/// front end of its own, the case where exactly one of the two refuses is the case worth
/// being able to see.
pub(super) fn Unread_Subject(subject: &Unread<'_>) -> Finding
{
    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: subject.inputs,
        subject_name: subject.path.clone(),
        applicability: subject.applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "no syntax fact could be read for this file, so any check it declares is \
             missing from the index every mirror claim is resolved against: {}",
            subject.because
        ),
        locations: vec![subject.path.clone()],
    };
}
