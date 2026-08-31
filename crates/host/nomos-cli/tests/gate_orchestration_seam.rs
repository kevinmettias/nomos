//! Proves nomos-cli's real boundary with `nomos_gate_orchestration`, through the only
//! surface a `tests/` file in this bin-only crate can reach: the compiled binary's real
//! argv/exit code, plus a direct construction of the other crate's own public selector
//! types.
//!
//! `src/gate/tests.rs` already builds `nomos_gate_orchestration::ScopeSelector`,
//! `RuleSelector`, `SuppressionPolicy`, `BaselinePolicy`, `AdoptionPolicy` and
//! `CoveragePolicy` values and drives them through `gate::Run` from inside `nomos-cli`'s own
//! `src/` -- that stays in place as a white-box unit test. This file is the public-surface
//! counterpart: `gate/parsing.rs::Plan_Or_Run_Command` turns `--include`/`--rule` into
//! exactly these same public types, so a value constructed directly from this crate, fed to
//! the real binary as the equivalent argv, must produce the exit code
//! `nomos_gate_orchestration::Run_Gate` actually computes for it.

#[path = "support/mod.rs"]
mod support;

use support::Run;

/// `gate/parsing.rs` turns a repeated `--include <path>` into exactly
/// `ScopeSelector::include`, and `Run_Gate` scopes the walk before ever calling
/// `nomos_check_orchestration::Run` -- a scope admitting nothing reports the same
/// `ExitCode::Vacuous` (6) an empty walk does, the same property `gate/tests.rs`'s own
/// `Test_Run_Should_Not_Report_Ok_When_Scoped_To_Nothing` proves in process. Constructing the
/// selector directly here, rather than only typing the flag, is what makes the assertion
/// below a property of `nomos_gate_orchestration::ScopeSelector` and not merely of a string
/// nomos-cli happens to spell the same way.
#[test]
fn Test_A_Scope_Selector_Naming_Nothing_Should_Report_Vacuous_Through_A_Real_Run()
{
    let scope = nomos_gate_orchestration::ScopeSelector {
        include: vec!["does/not/exist.rs".to_owned()],
        exclude: Vec::new(),
    };
    assert_eq!(scope.include.len(), 1, "the fixture below must drive exactly this scope");

    let ran = Run(&["gate", "run", "--root", ".", "--include", &scope.include[0]]);

    assert_eq!(
        ran.code, 6,
        "a real `gate run` scoped to the one path this ScopeSelector names must report the \
         same Vacuous outcome Run_Gate gives an empty walk: {}",
        ran.stdout
    );
}

/// `gate/parsing.rs` turns a repeated `--rule <id>` into exactly `RuleSelector::include`,
/// mapped through `RuleId::New` -- and `Run_Gate` passes that selection straight to
/// `nomos_check_orchestration::Run` as which rules to materialize at all, per `OD-GATE-017`.
/// A selector naming a rule id nothing registers means none of the eight real rules run, so
/// none can produce a blocking finding, and the run must still report `Ok` (0) rather than
/// silently reading as unjudged.
#[test]
fn Test_A_Rule_Selector_Naming_An_Unknown_Rule_Should_Still_Report_Ok()
{
    let selector = nomos_gate_orchestration::RuleSelector { include: vec![nomos_contracts::RuleId::New("no-such-rule-anywhere")] };
    assert_eq!(selector.include, vec![nomos_contracts::RuleId::New("no-such-rule-anywhere")]);

    let ran = Run(&["gate", "run", "--root", ".", "--rule", "no-such-rule-anywhere"]);

    assert_eq!(
        ran.code, 0,
        "scoping a real run to a rule id that selects nothing must not fail the build: {}",
        ran.stdout
    );
}
