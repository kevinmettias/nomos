//! The baseline-debt report `run` prints, exercised.

use super::super::{ExitCode, Render_Run};
use super::Judged_With;
use nomos_contracts::RuleId;
use nomos_gate_orchestration::{BaselineAllowance, BaselinePopulation};

/// The populations the tests below render, named once each so that a scope's allowance and
/// the occurrences held against it are read together rather than spelled `5` at four call
/// sites and meant differently at one of them.
///
/// `ACCEPTED`/`OVER_IT` is the exceeded case, by four; `ACCEPTED_WITH_ROOM`/`WITHIN_IT` is the
/// same allowance with nothing over it; `UNBOUNDED` is the observed count a scope with no
/// stated limit holds, chosen well above every other count here so that a report which
/// exceeded at any of them would exceed at this one too.
const ACCEPTED: u32 = 1;
const OVER_IT: u32 = 5;
const ACCEPTED_WITH_ROOM: u32 = 5;
const WITHIN_IT: u32 = 2;
const UNBOUNDED: u32 = 900;

/// A run whose baselined scope holds `observed` occurrences against an allowance of
/// `allowed`, rendered.
///
/// The declared path is spelled the way an author would plausibly write it rather than the
/// way `Subject_Of_Path` normalizes it, so that the output a test reads is the one the
/// failing behavior actually produced: a digest where the author wrote a path.
fn Rendered_Population(allowed: BaselineAllowance, observed: u32) -> String
{
    return Rendered_Population_Declared(Some("./src/lib.rs"), allowed, observed);
}

/// The same, for an entry no file declared -- a policy a caller built in code.
fn Rendered_Population_Declared(declared_path: Option<&str>, allowed: BaselineAllowance, observed: u32) -> String
{
    let population = BaselinePopulation {
        rule: RuleId::New("no-single-line-function-bodies"),
        subject: nomos_model::Subject_Of_Path("src/lib.rs"),
        declared_path: declared_path.map(str::to_owned),
        allowed,
        observed,
    };
    let mut result = Judged_With(1, None);
    result.findings.baseline_populations = vec![population];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    assert_eq!(
        code,
        ExitCode::Ok,
        "baseline debt is reported, not turned into a verdict: {stderr:?}"
    );
    return String::from_utf8_lossy(&stdout).into_owned();
}

/// The numbers a reader acts on, and the sentence that stops them acting on more.
///
/// `OD-GATE-030` refuses attribution inside an exceeded population. A report that gave the
/// excess and stopped would leave a reader to pick which occurrences are new, which is the
/// claim the run cannot support, so the disclaimer is asserted as part of the output rather
/// than left to a reviewer to notice going missing.
#[test]
fn Test_An_Exceeded_Scope_Should_Report_Its_Arithmetic_And_Refuse_To_Name_Which_Are_New()
{
    let rendered = Rendered_Population(BaselineAllowance::AtMost(ACCEPTED), OVER_IT);

    assert!(rendered.contains("baseline debt has grown past what was adopted"), "{rendered}");
    assert!(rendered.contains("5 occurrence(s) now"), "{rendered}");
    assert!(rendered.contains("1 accepted at adoption"), "{rendered}");
    assert!(rendered.contains("4 more than accepted"), "{rendered}");
    assert!(rendered.contains("is not known"), "the report must refuse to name which are new: {rendered}");
}

/// The scope is named the way its author wrote it, not by the identity it folds to.
///
/// The defect this closes, as a reader met it: `no-single-line-function-bodies at
/// e3f85dfb5b619eeb400b77bf17e6437c` for an entry whose author typed `./src/lib.rs`. Both
/// halves are asserted, because either alone is satisfied by the wrong thing -- a report
/// that printed the path *and* the digest would pass a containment check on the path, and
/// one that printed neither would pass a check on the digest.
#[test]
fn Test_An_Exceeded_Scope_Should_Be_Named_As_Its_Author_Wrote_It_Rather_Than_By_Its_Digest()
{
    let rendered = Rendered_Population(BaselineAllowance::AtMost(ACCEPTED), OVER_IT);
    let digest = nomos_model::Subject_Of_Path("src/lib.rs").to_string();

    assert!(rendered.contains("./src/lib.rs"), "the author's own spelling is what they can act on: {rendered}");
    assert!(!rendered.contains(&digest), "a digest is the identity and not something the author can search their own configuration for: {rendered}");
}

/// An entry no file declared has no authored spelling, so the digest is printed and is not
/// dressed up as a path.
///
/// The converse control. Without it, a report that always printed the subject would satisfy
/// the test above only by accident of the fixture, and a reader of a programmatically built
/// policy would be shown a path that no file contains.
#[test]
fn Test_An_Exceeded_Scope_No_File_Declared_Should_Be_Named_By_Its_Digest_Alone()
{
    let rendered = Rendered_Population_Declared(None, BaselineAllowance::AtMost(ACCEPTED), OVER_IT);
    let digest = nomos_model::Subject_Of_Path("src/lib.rs").to_string();

    assert!(rendered.contains(&digest), "{rendered}");
    assert!(!rendered.contains("src/lib.rs"), "no path may be invented for an entry that named none: {rendered}");
}

/// A scope inside its allowance says nothing at all.
///
/// A line on every clean run is how the exceptional line stops being read.
#[test]
fn Test_A_Scope_Within_Its_Allowance_Should_Report_Nothing()
{
    let rendered = Rendered_Population(BaselineAllowance::AtMost(ACCEPTED_WITH_ROOM), WITHIN_IT);

    assert!(!rendered.contains("baseline debt has grown"), "{rendered}");
}

/// An unbounded scope is never exceeded, however much it holds.
#[test]
fn Test_An_Unbounded_Scope_Should_Report_Nothing_However_Much_It_Holds()
{
    let rendered = Rendered_Population(BaselineAllowance::Unbounded, UNBOUNDED);

    assert!(!rendered.contains("baseline debt has grown"), "{rendered}");
}
