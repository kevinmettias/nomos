//! The harness every declared domain is discharged through, and the child process half of
//! it.
//!
//! Held apart from the domains themselves because two modules reach it: [`domains`] runs it,
//! and [`declarations`] asks it whether a domain has a test registered at all.
//!
//! [`domains`]: crate::domains
//! [`declarations`]: crate::declarations

use nomos_contracts::Strategy;
use nomos_integration_tests::{
    Child_Variable, Cross_Environment_Owed, CrossEnvironment, Digest_In, Production, Report_Line,
    Verification, Verify,
};

/// Discharges the cross-environment half of a scope claim.
///
/// Spawns this same test binary, running this same test, with [`Child_Variable`] set. The
/// child prints its digest and returns; the parent compares. That is a second process on
/// the same machine, which is exactly what `CrossRun` names — and what an in-process
/// repetition cannot reach, because hash seeds, address layout and environment are fixed
/// for the life of a process and vary between two.
///
/// # Why the child's early return is not the defect this workspace hunts
///
/// A test that returns early prints `ok`, and `docs/records/OD-GATE-001` is about exactly
/// that. The branch below is not an instance of it: it is only ever taken in a process the
/// parent spawned, and the parent asserts on what it printed. No run of the suite reaches
/// the early return without a run of the suite having made the assertion. A `cargo test`
/// on a machine with nothing configured takes the full path.
fn Discharge_Scope<S: Strategy>(domain: &str, verification: &Verification, golden: &str)
{
    match Cross_Environment_Owed(S::SCOPE)
    {
        CrossEnvironment::Nothing =>
        {}
        CrossEnvironment::SecondProcess =>
        {
            Compare_Against_Child(domain, verification);
        }
        CrossEnvironment::SecondProcessAndGolden =>
        {
            Compare_Against_Child(domain, verification);
            assert_eq!(
                verification.digest, golden,
                "{domain} declares {} and its bytes do not match the digest captured for \
                 another platform. Either this platform produces different bytes — in which \
                 case the declaration is false and the implementation or the scope must \
                 change — or the encoding changed deliberately, in which case every fact \
                 ever keyed under the old bytes has just been re-addressed and this constant \
                 is the place that says so.",
                S::SCOPE
            );
        }
    }
}

fn Compare_Against_Child(domain: &str, verification: &Verification)
{
    let executable = std::env::current_exe().expect("a running test has an executable path");
    let output = std::process::Command::new(executable)
        .args(["--exact", "--nocapture", Test_Name_For(domain)])
        .env(Child_Variable(), domain)
        .output()
        .expect("the test binary must be runnable as a child; a scope claim it cannot verify is a scope claim nothing checks");
    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "the child process running {domain} failed: {printed}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reported = Digest_In(&printed, domain).unwrap_or_else(|| {
        // A child selected by `--exact` under a name that matches no test exits zero and
        // prints nothing, so a missing digest line is the only evidence that it ran nothing.
        // Falling back to the parent's own digest would compare a value against itself and
        // discharge the CrossRun claim without a second process ever having produced bytes.
        panic!("the child running {domain} printed no digest line; it printed: {printed}")
    });

    assert_eq!(
        reported, verification.digest,
        "{domain} produced different bytes in a second process. That is the failure a \
         CrossRun claim exists to catch: an unordered collection, a hash seed, or an \
         address reaching the output."
    );
}

/// The test function name a domain's child process should run.
///
/// Written down rather than derived from `std::any::type_name` or a macro, because the
/// child is selected by `--exact` and a name that drifted from the function would spawn a
/// child that ran *no* tests, exited zero, printed nothing, and failed only on the missing
/// digest line — a failure that names the wrong cause.
///
/// The module path is part of that name. `--exact` matches what `--list` prints, so a test
/// living in `domains` is `domains::Test_...` and a bare function name selects nothing —
/// which is the same silent-child failure by a different route.
pub(crate) fn Test_Name_For(domain: &str) -> &'static str
{
    return match domain
    {
        "syntax-fact-production" => "domains::Test_The_Parser_Should_Meet_Its_Declared_Strategy",
        "module-index-rollup" => "domains::Test_The_Rollup_Should_Meet_Its_Declared_Strategy",
        "scan-fact-production" => "domains::Test_The_Scanner_Should_Meet_Its_Declared_Strategy",
        "fact-reuse" => "domains::Test_The_Fact_Cache_Should_Meet_Its_Declared_Strategy",
        "snapshot-serialization" =>
        {
            "domains::Test_Snapshot_Serialization_Should_Meet_Its_Declared_Strategy"
        }
        "bundle-serialization" =>
        {
            "domains::Test_Bundle_Serialization_Should_Meet_Its_Declared_Strategy"
        }
        "projection-output" => "domains::Test_Projection_Output_Should_Meet_Its_Declared_Strategy",
        // The domain arrives as a string, so this arm is the only thing checking the mapping
        // is complete. Any fallback name would spawn a child that selected no test, exited
        // zero and printed nothing — the silent-child failure the doc above describes, then
        // reported at the missing digest line under a cause that is not the real one.
        other => panic!("no test is registered for the domain {other}"),
    };
}

/// Whether this process is a child spawned to report on one domain.
fn Child_For(domain: &str) -> bool
{
    return std::env::var(Child_Variable()).is_ok_and(|asked| return asked == domain);
}

/// The whole of one domain's obligation, discharged.
// `produce` is erased because `Verify` takes it erased. A type parameter here would be
// coerced to the same trait object one line into the body, and would monomorphize this whole
// function — child branch, golden comparison and all — once per domain closure to get there.
pub(crate) fn Check<S: Strategy>(domain: &str, produce: &dyn Fn() -> Vec<u8>, golden: &str)
{
    // A child reports and returns. It must not spawn a child of its own, which would
    // recurse until the machine ran out of processes.
    if Child_For(domain)
    {
        let production = Production { trace: produce() };
        println!(
            "{}",
            Report_Line(domain, &production.Digest_At(S::STRENGTH))
        );
        return;
    }

    let verification = Verify::<S>(domain, produce);
    Discharge_Scope::<S>(domain, &verification, golden);

    eprintln!(
        "{domain}: {} / {} / {} — discharged {}",
        S::STRENGTH,
        S::SCOPE,
        S::TRACE,
        verification.discharged.join(", ")
    );
}
