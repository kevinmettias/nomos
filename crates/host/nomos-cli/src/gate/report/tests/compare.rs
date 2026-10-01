//! The `compare` verb's rendering, exercised.

use super::super::{ExitCode, Render_Compare};
use super::Judged_With;
use nomos_contracts::Digest128;
use nomos_gate_orchestration::GateRunProvenance;
use nomos_platform::Timestamp;

/// The two runs' ids: distinct, so a report that named one side where it meant the other
/// would be visible.
const BASELINE_FILL: u8 = 1;
const CANDIDATE_FILL: u8 = 2;

/// The policies the two sides declare: the same one when they were judged alike, and two
/// different ones when a test is about the difference between them.
const POLICY: u8 = 7;
const OTHER_POLICY: u8 = 8;

/// A provenance whose policy is `policy` and whose every other component is shared, so
/// that two of them differ in the policy and in nothing else.
fn Provenance_With(policy: u8) -> GateRunProvenance
{
    let shared = Digest128::From_Bytes([1; Digest128::BYTE_LENGTH]);

    return GateRunProvenance {
        source: shared,
        policy: Digest128::From_Bytes([policy; Digest128::BYTE_LENGTH]),
        selection: shared,
        instrument: shared,
        at: Timestamp::From_Unix_Seconds(0),
    };
}

/// What a comparison produced: the code it reduced to and the text it wrote.
///
/// Named rather than returned as a pair, so that an assertion reading the code cannot be
/// mistaken for one reading the rendering when the two are the same width.
struct Comparison
{
    code: ExitCode,
    rendered: String,
}

/// Two sides compared -- the baseline judged under `POLICY`, the candidate under
/// `candidate_policy`, `None` for a run that cannot say what judged it -- and the rendering
/// read back with the code it reduced to.
///
/// The tests below differ in that one thing and in nothing else, so the pair is spelled once
/// here rather than once per test.
fn Compared(candidate_policy: Option<u8>) -> Comparison
{
    let baseline = Judged_With(BASELINE_FILL, Some(Provenance_With(POLICY)));
    let candidate = Judged_With(CANDIDATE_FILL, candidate_policy.map(Provenance_With));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Compare(&baseline, &candidate, &mut stdout, &mut stderr);

    return Comparison { code, rendered: String::from_utf8_lossy(&stdout).into_owned() };
}

/// Two runs judged alike say nothing about it.
///
/// A caveat printed on every comparison is how a reader learns to skip the line that
/// matters, which would cost more than the caveat saves.
#[test]
fn Test_Render_Compare_Should_Say_Nothing_When_The_Two_Runs_Were_Judged_Alike()
{
    let compared = Compared(Some(POLICY));

    assert_eq!(compared.code, ExitCode::Ok, "{}", compared.rendered);
    assert!(!compared.rendered.contains("not judged alike"), "{}", compared.rendered);
    assert!(!compared.rendered.contains("does not record what judged it"), "{}", compared.rendered);
}

/// A policy difference is stated, and stated *before* the diff.
///
/// Order is the assertion, not decoration: a reader who takes the diff at face value has
/// already been misled by the time they reach a footnote, so a correct sentence in the
/// wrong place does not satisfy `OD-GATE-031`.
#[test]
fn Test_Render_Compare_Should_State_A_Policy_Difference_Before_The_Difference()
{
    let compared = Compared(Some(OTHER_POLICY));

    assert_eq!(compared.code, ExitCode::Ok, "a comparability is not a verdict: {}", compared.rendered);
    assert!(compared.rendered.contains("different declared policies"), "{}", compared.rendered);
    let stated = compared.rendered.find("not judged alike").unwrap_or(usize::MAX);
    let counted = compared.rendered.find(" added, ").unwrap_or(0);
    assert!(stated < counted, "the caveat has to arrive first: {}", compared.rendered);
}

/// A run that does not say what judged it is named, and the diff is still printed.
///
/// `P106` settled the same trade one verb over: refusing the verdict is not refusing the
/// answer, and throwing away what the caller asked for in order to say something about it
/// is the worse of the two.
#[test]
fn Test_Render_Compare_Should_Name_A_Run_That_Cannot_Say_What_Judged_It()
{
    let compared = Compared(None);

    assert_eq!(compared.code, ExitCode::Ok, "{}", compared.rendered);
    assert!(compared.rendered.contains("does not record what judged it"), "{}", compared.rendered);
    assert!(
        compared.rendered.contains("0 added, 0 removed"),
        "the difference is still reported: {}",
        compared.rendered
    );
}
