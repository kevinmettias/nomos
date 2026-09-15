//! The real seam between `nomos_lang_rust_clippy` and `nomos_platform_std`, proven from
//! outside the crate through a *separate*, cheap real `cargo clippy` pass over a scratch
//! one-crate workspace this file creates and owns.
//!
//! `src/fact_context.rs`'s own inline
//! `Test_Discover_Workspace_And_Materialize_Workspace_Should_Find_Every_Real_Workspace_Member`
//! already drives `StdProcessLauncher` for real, over this whole repository — deliberately
//! left in place rather than moved or duplicated here: a whole-workspace `cargo clippy` pass
//! is expensive, and this crate already pays that cost once. This file proves the identical
//! wiring (`StdProcessLauncher` really implements `nomos_platform::ProcessLauncher`, and
//! `Materialize_Workspace` really drives a real subprocess through it end to end) over a
//! single trivial crate instead, the same scratch-fixture pattern
//! `nomos-lang-rust-cargo/tests/invalidation.rs` already uses for the identical reason: fast,
//! self-contained, and destructive to nothing in this tree.

use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
use nomos_lang_rust_clippy::{FactContext, Materialize_Workspace};
use nomos_platform_std::{StdEnvironment, StdProcessLauncher};
use std::path::{Path, PathBuf};

/// A one-member scratch workspace with no dependencies of its own, removed when the test
/// ends — small enough that a real `cargo clippy --workspace --all-targets` pass over it
/// finishes in a few seconds rather than the whole-repository pass's minutes.
struct Fixture
{
    root: PathBuf,
}

impl Fixture
{
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-lang-rust-clippy-it-{name}-{}", std::process::id()));

        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("tiny/src")).expect("a scratch member directory");

        std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"tiny\"]\nresolver = \"2\"\n")
            .expect("writing the scratch workspace manifest");
        std::fs::write(
            root.join("tiny/Cargo.toml"),
            "[package]\nname = \"tiny\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("writing the scratch member's manifest");
        // `answer`, not `Answer`, and no `return` -- this scratch crate carries none of this
        // workspace's own `clippy.toml`/`rustfmt.toml`, so ordinary `clippy::style` defaults
        // apply and a PascalCase name or a trailing `return` would report a real diagnostic,
        // which is exactly the noise this fixture exists to have none of.
        std::fs::write(root.join("tiny/src/lib.rs"), "pub fn answer() -> i32\n{\n    42\n}\n")
            .expect("writing the scratch member's source");

        return Self { root };
    }

    fn Path(&self) -> &Path
    {
        return &self.root;
    }
}

impl Drop for Fixture
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// Fill bytes distinct enough that a context's three digests differ from one another; each
/// value carries no meaning beyond "not equal to the others".
const VARIANT_DIGEST_FILL: u8 = 2;
const CONFIGURATION_DIGEST_FILL: u8 = 3;

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

/// The happy path: a real `cargo clippy` pass, launched through the real
/// `nomos_platform_std::StdProcessLauncher`, over a scratch crate with nothing for clippy to
/// report.
#[test]
fn Test_Materialize_Workspace_Should_Run_A_Real_Clippy_Pass_Through_The_Real_Std_Process_Launcher()
{
    let fixture = Fixture::New("clippy-std-launcher");

    let facts = Materialize_Workspace(fixture.Path(), Context(), &StdProcessLauncher, &StdEnvironment)
        .expect("a real, trivial one-crate workspace under a real cargo clippy pass");

    assert_eq!(facts.len(), 1, "{facts:?}");
    let tiny = facts.first().expect("asserted len 1 above");

    assert_eq!(tiny.path, "tiny");
    assert_eq!(tiny.subject, nomos_model::Subject_Of_Path("tiny"));

    let decoded = nomos_cap_lint::Parse_Payload(&tiny.fact.payload.bytes).expect("this crate's own encoding");
    assert_eq!(decoded.package, "tiny");
    assert!(
        decoded.diagnostics.is_empty(),
        "a trivial function with no lint issues must report clean: {:?}",
        decoded.diagnostics
    );
}
