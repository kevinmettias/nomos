//! The two verbs that touch a governed output on disk: `Render`, which places one, and
//! `Freshness`, which reads one back.

use std::path::PathBuf;

use nomos_platform_std::StdFileSystem;

use crate::{Run, SpecCommand};
use crate::request::{FreshnessRequest, RenderRequest};
use crate::spec_outcome::{FreshnessRefusal, RenderAnswer, RenderRefusal, SpecOutcome, Verdict};

use super::{EMBEDDED_PROFILE, No_Corpus, Scratch_Root};

/// How much of a rendered body's generated header a failure message quotes back. Enough to
/// carry the `---` fence and `nomos_generated: true` in full.
const GENERATED_HEADER_PREVIEW_CHARS: usize = 40;

/// Runs `Run(Render, ..)` for the embedded profile into `into`, and hands back the projection
/// that was built and placed. Panics if the verb refused one, which no fixture here does.
fn Render_Embedded_Profile(into: &std::path::Path) -> RenderAnswer
{
    let request = SpecCommand::Render(RenderRequest {
        profile: EMBEDDED_PROFILE.to_owned(),
        into: into.to_path_buf(),
        subject: None,
    });

    let outcome = Run(&request, &No_Corpus(), &StdFileSystem);

    let SpecOutcome::Render(answer) = outcome
    else
    {
        panic!("Run(Render, ..) must answer SpecOutcome::Render");
    };

    return answer.expect("domain-specification builds from the embedded records alone");
}

#[test]
fn Test_Run_Of_Render_Should_Place_A_Projection_Built_With_No_Corpus()
{
    let answer = Render_Embedded_Profile(&Scratch_Root("render"));

    assert_eq!(answer.id, EMBEDDED_PROFILE);
    let body = std::fs::read_to_string(&answer.body).expect("the body was written");
    assert!(
        body.starts_with("---\nnomos_generated: true\n"),
        "{:?}",
        body.get(..GENERATED_HEADER_PREVIEW_CHARS)
    );
    let sidecar = std::fs::read_to_string(&answer.sidecar).expect("the sidecar was written");
    assert!(sidecar.contains("\"profile\": \"domain-specification\""), "{sidecar:.200}");
}

#[test]
fn Test_Run_Of_Render_Should_Refuse_An_Unknown_Profile()
{
    let into = Scratch_Root("render-unknown");
    let request = SpecCommand::Render(RenderRequest {
        profile: "no-such-profile".to_owned(),
        into,
        subject: None,
    });

    let outcome = Run(&request, &No_Corpus(), &StdFileSystem);

    let SpecOutcome::Render(Err(RenderRefusal::NoSuchProfile { requested, known })) = outcome
    else
    {
        panic!("an unknown profile must refuse NoSuchProfile");
    };
    assert_eq!(requested, "no-such-profile");
    assert!(known.iter().any(|id| return id == EMBEDDED_PROFILE), "{known:?}");
}

/// Runs `Run(Freshness, ..)` over `into` for the embedded profile alone, with no `--require`,
/// and hands back the one profile outcome `--profile` narrows this run to.
///
/// Shared by every `Test_Run_Of_Freshness_Should_Report_*` test below: each stages `into`
/// differently (freshly rendered, empty, ...) but asks the identical question of it, and a
/// second, independent reading of the same `Run`/destructure/narrow steps would drift from
/// this one silently the way `check-intrafile-duplication` caught it drifting the first time.
fn Single_Freshness_Outcome(into: PathBuf) -> Verdict
{
    let outcome = Run(
        &SpecCommand::Freshness(FreshnessRequest {
            into,
            profile: Some(EMBEDDED_PROFILE.to_owned()),
            require: Vec::new(),
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Freshness(answer) = outcome
    else
    {
        panic!("Run(Freshness, ..) must answer SpecOutcome::Freshness");
    };
    let answer = answer.expect("a resolvable single profile does not refuse");
    let examined = answer.examined.len();
    let [outcome] = answer.examined.try_into().unwrap_or_else(|_| {
        panic!("--profile narrows this run to exactly one profile: {examined:?}");
    });

    return outcome.verdict;
}

#[test]
fn Test_Run_Of_Freshness_Should_Report_A_Freshly_Rendered_Output_As_Current()
{
    let into = Scratch_Root("freshness-current");
    let render = Run(
        &SpecCommand::Render(RenderRequest {
            profile: EMBEDDED_PROFILE.to_owned(),
            into: into.clone(),
            subject: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );
    assert!(matches!(render, SpecOutcome::Render(Ok(_))), "the fixture did not render");

    let verdict = Single_Freshness_Outcome(into);
    assert!(
        matches!(&verdict, Verdict::Compared(Ok(freshness)) if freshness.Is_Fresh()),
        "{verdict:?}"
    );
}

#[test]
fn Test_Run_Of_Freshness_Should_Report_An_Empty_Build_Root_As_Absent()
{
    let into = Scratch_Root("freshness-absent");

    let verdict = Single_Freshness_Outcome(into);
    assert!(matches!(verdict, Verdict::Absent), "{verdict:?}");
}

#[test]
fn Test_Run_Of_Freshness_Should_Refuse_An_Unexamined_Requirement()
{
    let into = Scratch_Root("freshness-unexamined");

    let outcome = Run(
        &SpecCommand::Freshness(FreshnessRequest {
            into,
            profile: Some(EMBEDDED_PROFILE.to_owned()),
            require: vec!["diagram-set".to_owned()],
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Freshness(Err(FreshnessRefusal::RequirementUnexamined { requested, only })) = outcome
    else
    {
        panic!("a requirement outside --profile's narrowing must refuse RequirementUnexamined");
    };
    assert_eq!(requested, "diagram-set");
    assert_eq!(only.as_deref(), Some(EMBEDDED_PROFILE));
}
