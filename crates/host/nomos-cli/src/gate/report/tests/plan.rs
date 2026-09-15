//! The `plan` verb's rendering, exercised.

use super::super::{ExitCode, Render_Plan};

/// A real, composed registry -- `nomos_gate_orchestration::Run` never touches a
/// filesystem or a subprocess for `plan`, so this drives `Render_Plan` against a
/// genuine `GateOutcome` rather than a hand-built one.
#[test]
fn Test_Render_Plan_Should_Report_Ok_And_List_Every_Registered_Rule()
{
    let outcome = nomos_gate_orchestration::Run(&nomos_gate_orchestration::GateCommand::default());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Plan(&outcome, &mut stdout, &mut stderr);

    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    assert_eq!(code, ExitCode::Ok, "{rendered}{}", String::from_utf8_lossy(&stderr));
    assert!(rendered.starts_with("rules: "), "{rendered}");
}
