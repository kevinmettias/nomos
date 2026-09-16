//! The real seams between `nomos_lang_rust_clippy` and `nomos_analysis` / `nomos_contracts` /
//! `nomos_model` / `nomos_platform`, driven through this crate's own public
//! [`Materialize_Workspace`] and [`Discover_Workspace`] rather than through a real `cargo
//! clippy` subprocess — the same substitution `src/clippy_error.rs`'s own inline `FakeLauncher`
//! already makes for the identical reason: a fake launcher is fast and deterministic where a
//! real one is neither.
//!
//! `nomos_contracts` and `nomos_model` were previously exercised only from inside
//! `src/fact_context.rs`'s own `#[cfg(test)]` block (`Test_A_Fact_Key_Should_Depend_On_The_
//! Guarantee`, which called the crate-private `Compute_Fact_Key` directly); this file proves
//! the same property — a fact's key depends on the build variant it was materialized under —
//! through `Materialize_Workspace` alone, since `Compute_Fact_Key` is not public. `nomos_
//! analysis` and `nomos_platform` had no suite anywhere; both are exercised here for the first
//! time.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_analysis::GuaranteeDigest;
use nomos_cap_lint::{Parse_Payload, Payload_Schema};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
use nomos_lang_rust_clippy::{Declared_Guarantee, DiagnosticsFact, FactContext, Materialize_Workspace};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use std::path::{Path, PathBuf};
use nomos_platform_std::StdEnvironment;

/// The source line the fabricated diagnostic attributes its own warning to -- any plausible
/// line inside the fixture's own `src/lib.rs` would do, named so the fixture's JSON carries
/// no bare literal.
const FABRICATED_DIAGNOSTIC_LINE: u32 = 5;

/// Fill bytes distinct enough that a context's three digests differ from one another; each
/// value carries no meaning beyond "not equal to the others" -- the same convention
/// `src/fact_context.rs`'s own inline tests already use.
const VARIANT_DIGEST_FILL: u8 = 2;
const CONFIGURATION_DIGEST_FILL: u8 = 3;

/// The build-variant fill the different-variant context below uses -- distinct from
/// [`VARIANT_DIGEST_FILL`] so the two contexts' own keys must file apart.
const DIFFERENT_VARIANT_DIGEST_FILL: u8 = 9;

/// The exit code `cargo clippy` reports for a real compile error -- `rustc`'s own
/// `EXIT_FAILURE`, the value a real failing run carries rather than a clean zero exit.
const COMPILE_FAILURE_EXIT_CODE: i32 = 101;

/// `Discover_Workspace` absolutizes `root` against the real process working directory
/// before relativizing any `package_id` against it (`Absolute_Path_Of`, added for
/// `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`) — an already-absolute root such as
/// this one, from a real (if never created) directory under the OS temp directory, passes
/// through that step unchanged, so this fixture still proves the same string arithmetic a
/// real repository root would without either this crate or the fake launcher below ever
/// touching a real filesystem at this path.
fn Root() -> PathBuf
{
    return std::env::temp_dir().join("nomos-lang-rust-clippy-materialize-workspace-seam");
}

/// A `path+file://` package id `cargo clippy` could plausibly report for `path` — the
/// same construction `src/clippy_error.rs`'s own `local_tests::Package_Id_Uri` uses,
/// reproduced here because that one is private to the crate's own test module.
fn Package_Id_Uri(path: &Path) -> String
{
    let forward = path.to_string_lossy().replace('\\', "/");
    let rooted = if forward.starts_with('/') { forward } else { format!("/{forward}") };

    return format!("path+file://{rooted}#0.1.0");
}

/// A launcher that hands `Materialize_Workspace` a fixed JSON-lines stream, or a failed
/// exit, instead of running a real `cargo clippy` — the same shape `src/clippy_error.rs`'s
/// own private `FakeLauncher` takes, reimplemented here because that one is not public.
struct FakeLauncher
{
    stdout: String,
    outcome: ExitOutcome,
    stderr: String,
}

impl FakeLauncher
{
    fn Reporting(stdout: String) -> Self
    {
        return Self { stdout, outcome: ExitOutcome::Exited { code: 0 }, stderr: String::new() };
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for FakeLauncher
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for FakeLauncher
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
    {
        return Ok(ProcessOutput {
            outcome: self.outcome,
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
        });
    }
}

/// One workspace member, `nomos-rules`, reporting one real-shaped diagnostic — the same
/// `compiler-artifact` / `compiler-message` pair `cargo clippy --message-format=json`
/// actually prints, in the same two-line form `src/clippy_error.rs`'s own fixtures use.
fn Single_Member_Clippy_Output() -> String
{
    let package_id = Package_Id_Uri(&Root().join("nomos-rules"));
    let artifact = serde_json::json!({
        "reason": "compiler-artifact",
        "package_id": package_id.clone(),
        "target": { "kind": ["lib"] }
    })
    .to_string();
    let message = serde_json::json!({
        "reason": "compiler-message",
        "package_id": package_id.clone(),
        "message": {
            "level": "warning",
            "message": "unneeded return statement",
            "code": { "code": "clippy::needless_return" },
            "spans": [{ "file_name": "src/lib.rs", "line_start": FABRICATED_DIAGNOSTIC_LINE, "is_primary": true }]
        }
    })
    .to_string();

    return format!("{artifact}\n{message}\n");
}

fn Context(generation: GenerationId) -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        generation,
    };
}

/// The happy path: a fake but real-shaped `cargo clippy` stream, materialized into a fact
/// `nomos_analysis` and `nomos_model` both recognize as their own.
#[test]
fn Test_Materialize_Workspace_Should_Produce_A_Fact_Nomos_Analysis_And_Nomos_Model_Both_Recognize()
{
    let launcher = FakeLauncher::Reporting(Single_Member_Clippy_Output());
    let context = Context(GenerationId::INITIAL);

    let facts = Materialize_Workspace(&Root(), context, &launcher, &StdEnvironment).expect("the fake launcher reports one clean member");

    assert_eq!(facts.len(), 1, "{facts:?}");
    let member = facts.first().expect("asserted len 1 above");

    assert_eq!(member.path, "nomos-rules");
    assert_eq!(
        member.subject,
        nomos_model::Subject_Of_Path("nomos-rules"),
        "the subject a real rule looks the fact up by must be the same one `nomos_model` computes for this path"
    );
    assert_eq!(member.fact.guarantee, Declared_Guarantee());
    assert_eq!(member.fact.Key().guarantee, GuaranteeDigest::Of(&Declared_Guarantee()));
    assert_eq!(member.fact.payload.schema, Payload_Schema());

    let decoded = Parse_Payload(&member.fact.payload.bytes).expect("this crate's own encoding");
    assert_eq!(decoded.package, "nomos-rules");
    let diagnostic = decoded.diagnostics.first().expect("the fixture stdout names one diagnostic");
    assert_eq!(diagnostic.lint.as_deref(), Some("clippy::needless_return"));
}

/// The lifecycle constraint the other side imposes: two runs at different generations over
/// the same inputs must file under the *same* key (a later generation re-asks the same
/// question), while two runs under a different build variant must file apart.
/// `src/fact_context.rs`'s own (now-removed) `Test_A_Fact_Key_Should_Depend_On_The_Guarantee`
/// proved the neighboring claim — that the key depends on the *guarantee* — through the
/// crate-private `Compute_Fact_Key`, which is not reachable from outside the crate; the
/// build-variant axis proved here is the closest equivalent `Materialize_Workspace`'s public
/// signature actually exposes, and it exercises the identical `FactKey` fields
/// (`nomos_contracts::BuildVariantId`, `nomos_contracts::GenerationId`) that made the
/// original claim true.
#[test]
fn Test_The_Facts_Key_Should_Depend_On_The_Build_Variant_But_Not_On_The_Generation()
{
    let launcher = FakeLauncher::Reporting(Single_Member_Clippy_Output());
    let base = Context(GenerationId::INITIAL);

    let at_base = Materialize_Workspace(&Root(), base, &launcher, &StdEnvironment).expect("base context");
    let at_later_generation = Materialize_Workspace(&Root(), Later_Generation(base), &launcher, &StdEnvironment).expect("later generation");
    let at_different_variant = Materialize_Workspace(&Root(), Different_Variant(base), &launcher, &StdEnvironment).expect("different variant");

    assert_eq!(
        Key_Of(&at_base),
        Key_Of(&at_later_generation),
        "a later generation re-asks the same question and must file under the same key"
    );
    assert_ne!(
        Key_Of(&at_base),
        Key_Of(&at_different_variant),
        "two offers of the same subject under a different build variant must file apart"
    );
    assert_ne!(
        Single_Member(&at_base).fact.Generation(),
        Single_Member(&at_later_generation).fact.Generation(),
        "the generation itself must still differ even though the key does not"
    );
}

/// `base`'s own context one generation later -- the same question, re-asked.
fn Later_Generation(base: FactContext) -> FactContext
{
    return FactContext { generation: GenerationId::INITIAL.Next(), ..base };
}

/// `base`'s own context under a different build variant -- the same subject and generation,
/// asked under a variant whose digest differs from [`VARIANT_DIGEST_FILL`]'s.
fn Different_Variant(base: FactContext) -> FactContext
{
    let variant = BuildVariantId::From_Digest(Digest128::From_Bytes([DIFFERENT_VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH]));

    return FactContext { variant, ..base };
}

/// The single fact a one-member stream materializes into -- the invariant the fixture's own
/// [`Single_Member_Clippy_Output`] makes true, and the reason this `first()` is not a bet.
fn Single_Member(facts: &[DiagnosticsFact]) -> &DiagnosticsFact
{
    return facts.first().expect("Single_Member_Clippy_Output names exactly one workspace member, so materializing it yields exactly one fact");
}

/// [`Single_Member`]'s own fact key, as the digest the store files it under.
fn Key_Of(facts: &[DiagnosticsFact]) -> nomos_contracts::Digest128
{
    return Single_Member(facts).fact.Key().Digest();
}

/// The error that crosses the boundary: a real compile failure — `cargo clippy` exiting
/// non-zero — must refuse rather than report a clean but empty result, and the refusal must
/// carry the real exit code and the real stderr rather than a generic message.
#[test]
fn Test_A_Non_Zero_Exit_Should_Refuse_Rather_Than_Report_A_Clean_Result()
{
    let launcher = FakeLauncher {
        stdout: String::new(),
        outcome: ExitOutcome::Exited { code: COMPILE_FAILURE_EXIT_CODE },
        stderr: "error[E0308]: mismatched types".to_owned(),
    };

    let error =
        Materialize_Workspace(&Root(), Context(GenerationId::INITIAL), &launcher, &StdEnvironment).expect_err("a non-zero exit must refuse");

    assert!(error.reason.contains("exit 101"), "{}", error.reason);
    assert!(error.reason.contains("mismatched types"), "{}", error.reason);
}
