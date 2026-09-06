//! The wire shape of a `nomos.requirement.trace.v1` payload, and its canonical encoding.
//!
//! Unlike `nomos.cap.goals.policy` and its `OD-RULES-011` siblings, this payload is not a
//! repository's raw declaration — it is the *already-judged* result of comparing that
//! declaration against the real tree, because only [`crate::provider::Materialize_Workspace`]
//! has the [`nomos_platform::FileSystem`] access a rule (in Rules zone, with no filesystem
//! port of its own) needs to tell a resolved citation from a stale one. So a [`Problem`] is
//! close in shape to `nomos_cap_lint::LintDiagnostic`: a fact whose own judgment is already
//! made, relayed by the rule that reads it rather than reached a second time.

mod refusal;

pub use refusal::Refusal;

/// One of the five ways `crate::predicates` already reports a stale or incomplete
/// assessment — kept as five variants, not collapsed to one "stale" tag, because a rule
/// reading this payload reconstructs one `Finding` per [`Problem`] and a reader comparing
/// two runs needs to tell which of the five kinds moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProblemKind
{
    /// A `site` line names a path or symbol that no longer resolves in this workspace.
    UnresolvedSite,
    /// A `gap` line (a `Partial` entry's own unsatisfied part) no longer resolves.
    UnresolvedGap,
    /// A `record` line names a governing record with no registration, or a registration
    /// with no document.
    UnresolvedRecord,
    /// A `Diverges` or `NotBinding` entry names no governing record at all.
    DivergenceWithNoRecord,
    /// A `Partial` entry names no gap at all.
    PartialWithNoGap,
}

impl ProblemKind
{
    const SITE_TAG: &'static str = "site";
    const GAP_TAG: &'static str = "gap";
    const RECORD_TAG: &'static str = "record";
    const DIVERGES_TAG: &'static str = "diverges";
    const PARTIAL_TAG: &'static str = "partial";

    /// The tag this kind is written with in [`Encode_Payload`]'s own tab-separated lines.
    const fn Tag(self) -> &'static str
    {
        return match self
        {
            Self::UnresolvedSite => Self::SITE_TAG,
            Self::UnresolvedGap => Self::GAP_TAG,
            Self::UnresolvedRecord => Self::RECORD_TAG,
            Self::DivergenceWithNoRecord => Self::DIVERGES_TAG,
            Self::PartialWithNoGap => Self::PARTIAL_TAG,
        };
    }

    /// The kind a tag names, or `None` if it names none of the five.
    fn Of_Tag(tag: &str) -> Option<Self>
    {
        return match tag
        {
            Self::SITE_TAG => Some(Self::UnresolvedSite),
            Self::GAP_TAG => Some(Self::UnresolvedGap),
            Self::RECORD_TAG => Some(Self::UnresolvedRecord),
            Self::DIVERGES_TAG => Some(Self::DivergenceWithNoRecord),
            Self::PARTIAL_TAG => Some(Self::PartialWithNoGap),
            _ => None,
        };
    }
}

/// One already-judged disagreement between a committed assessment and the workspace (or the
/// assessment's own completeness) it was checked against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem
{
    /// Which of the five checks this is.
    pub kind: ProblemKind,
    /// The requirement the stale assessment names -- the file stem, so a rule can point a
    /// `Finding` at `tests/contract/requirements/{requirement}.assessment`.
    pub requirement: String,
    /// The full, human-readable explanation -- composed here, by the provider that has the
    /// filesystem access to tell a missing file from a renamed symbol, not by the rule.
    pub message: String,
}

/// A repository's whole requirement-trace judgment: every stale or incomplete assessment
/// [`crate::predicates`] found, in a fixed, deterministic order (every unresolved site, then
/// every unresolved gap, then every unresolved record, then every divergence with no record,
/// then every partial with no gap -- each group itself ordered by requirement, since
/// [`crate::registry::Entries`] sorts the assessments it reads before any predicate runs).
///
/// Empty for a repository with no `tests/contract/requirements/` directory at all, or one
/// whose committed assessments all resolve -- the two cases `crate::provider`'s own module
/// doc says are the same fact to a caller that only reads this payload: nothing to report.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RequirementTracePayload
{
    pub problems: Vec<Problem>,
}

const PROBLEM_TAG: &str = "problem";

/// Encodes a payload as tab-separated lines, the same shape every other capability payload
/// in this workspace uses: diffable by a person, written in one place with no derive between
/// the data and the bytes.
#[must_use]
pub fn Encode_Payload(payload: &RequirementTracePayload) -> Vec<u8>
{
    let mut encoded = String::new();

    for problem in &payload.problems
    {
        encoded.push_str(PROBLEM_TAG);
        encoded.push('\t');
        encoded.push_str(problem.kind.Tag());
        encoded.push('\t');
        encoded.push_str(&problem.requirement);
        encoded.push('\t');
        encoded.push_str(&problem.message);
        encoded.push('\n');
    }

    return encoded.into_bytes();
}

/// Reads a payload back out of its canonical encoding.
///
/// # Errors
///
/// [`Refusal`] if the bytes are not valid UTF-8, a line does not open with [`PROBLEM_TAG`],
/// a line's kind field names none of [`ProblemKind`]'s five tags, or a line does not carry
/// all four fields (tag, kind, requirement, message).
pub fn Parse_Payload(bytes: &[u8]) -> Result<RequirementTracePayload, Refusal>
{
    let text = core::str::from_utf8(bytes).map_err(|error| return Refusal { reason: format!("not UTF-8: {error}") })?;

    let mut payload = RequirementTracePayload::default();
    for line in text.lines()
    {
        payload.problems.push(Problem_Of_Line(line)?);
    }

    return Ok(payload);
}

/// One line, decoded -- `PROBLEM_TAG`, a kind tag, a requirement and a message, in that
/// order. `splitn` on the fourth field so a message that happened to contain a tab is
/// carried whole rather than truncated at it.
fn Problem_Of_Line(line: &str) -> Result<Problem, Refusal>
{
    let mut fields = line.splitn(4, '\t');
    let (Some(tag), Some(kind), Some(requirement), Some(message)) = (fields.next(), fields.next(), fields.next(), fields.next())
    else
    {
        return Err(Refusal {
            reason: format!("line {line:?} does not carry all four fields"),
        });
    };

    if tag != PROBLEM_TAG
    {
        return Err(Refusal {
            reason: format!("line {line:?} does not open with `{PROBLEM_TAG}`"),
        });
    }
    let Some(kind) = ProblemKind::Of_Tag(kind)
    else
    {
        return Err(Refusal {
            reason: format!("line {line:?} names an unrecognized problem kind `{kind}`"),
        });
    };
    if requirement.is_empty()
    {
        return Err(Refusal {
            reason: format!("line {line:?} has an empty requirement"),
        });
    }

    return Ok(Problem {
        kind,
        requirement: requirement.to_owned(),
        message: message.to_owned(),
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Parse_Payload_Should_Round_Trip_A_Payload_Through_Its_Own_Encoding()
    {
        let payload = Sample();
        let encoded = Encode_Payload(&payload);
        let decoded = Parse_Payload(&encoded).expect("this crate's own encoding");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_An_Empty_Byte_String_Should_Decode_To_A_Payload_Reporting_Nothing()
    {
        let decoded = Parse_Payload(&[]).expect("no problems makes an empty payload unambiguous");

        assert_eq!(decoded, RequirementTracePayload::default());
    }

    #[test]
    fn Test_Encode_Payload_Should_Produce_Stable_Diffable_Bytes()
    {
        let rendered = String::from_utf8(Encode_Payload(&Sample())).expect("ASCII and tabs");

        assert_eq!(
            rendered,
            "problem\tsite\tCHK-003\tCHK-003: site crates/x.rs is not a file in this workspace\n"
        );
        assert!(!rendered.contains('\r'), "line endings must not be local");
    }

    #[test]
    fn Test_A_Line_With_Too_Few_Fields_Should_Be_Refused()
    {
        let error = Parse_Payload(b"problem\tsite\tCHK-003\n").expect_err("a message field is required");

        assert!(error.reason.contains("does not carry all four fields"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Line_Not_Opening_With_The_Problem_Tag_Should_Be_Refused()
    {
        let error = Parse_Payload(b"row\tsite\tCHK-003\tmessage\n").expect_err("the tag must be problem");

        assert!(error.reason.contains("does not open with"), "{}", error.reason);
    }

    #[test]
    fn Test_An_Unrecognized_Kind_Should_Be_Refused()
    {
        let error = Parse_Payload(b"problem\tmystery\tCHK-003\tmessage\n").expect_err("only five kinds exist");

        assert!(error.reason.contains("unrecognized problem kind"), "{}", error.reason);
    }

    #[test]
    fn Test_An_Empty_Requirement_Should_Be_Refused()
    {
        let error = Parse_Payload(b"problem\tsite\t\tmessage\n").expect_err("a problem is about some requirement");

        assert!(error.reason.contains("empty requirement"), "{}", error.reason);
    }

    #[test]
    fn Test_A_Message_Containing_A_Tab_Should_Survive_The_Round_Trip()
    {
        let payload = RequirementTracePayload {
            problems: vec![Problem {
                kind: ProblemKind::UnresolvedRecord,
                requirement: "CAP-002".to_owned(),
                message: "CAP-002: a message\twith an embedded tab".to_owned(),
            }],
        };

        let decoded = Parse_Payload(&Encode_Payload(&payload)).expect("a message field captures the rest of the line");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn Test_Every_Problem_Kind_Should_Round_Trip_Through_Its_Own_Tag()
    {
        for kind in [
            ProblemKind::UnresolvedSite,
            ProblemKind::UnresolvedGap,
            ProblemKind::UnresolvedRecord,
            ProblemKind::DivergenceWithNoRecord,
            ProblemKind::PartialWithNoGap,
        ]
        {
            let payload = RequirementTracePayload {
                problems: vec![Problem {
                    kind,
                    requirement: "CHK-003".to_owned(),
                    message: "a message".to_owned(),
                }],
            };
            let decoded = Parse_Payload(&Encode_Payload(&payload)).expect("this crate's own encoding");

            assert_eq!(decoded, payload, "{kind:?}");
        }
    }

    fn Sample() -> RequirementTracePayload
    {
        return RequirementTracePayload {
            problems: vec![Problem {
                kind: ProblemKind::UnresolvedSite,
                requirement: "CHK-003".to_owned(),
                message: "CHK-003: site crates/x.rs is not a file in this workspace".to_owned(),
            }],
        };
    }
}
