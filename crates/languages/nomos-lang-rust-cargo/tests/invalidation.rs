//! `dependency.edges`' own fact, taken through a real `MemoryFactStore::Invalidate` at a
//! real later generation.
//!
//! Every other test in this crate reads one generation. The generic invalidation lifecycle
//! is proven against synthetic payloads (`nomos-analysis`'s own tests) and against a
//! different capability's leaf fact (`nomos-lang-rust`, through
//! `nomos-lang-rust/tests/rollup.rs`). Neither exercises the claim `provider.rs`'s `Keyed`
//! makes for its own reason to leave `semantic_inputs` empty: that generation is the only
//! axis able to tell two resolutions of the same manifest apart. This file is that
//! exercise, over a real `cargo metadata` run against a fixture workspace rather than the
//! real repository, so nothing here is destructive to this tree's own `Cargo.toml`.

use nomos_analysis::{FactStore, GenerationCause, MemoryFactStore};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, IncrementalGranularity, SnapshotId};
use nomos_lang_rust_cargo::{FactContext, Materialize_Workspace};
use nomos_platform_std::StdProcessLauncher;
use std::path::{Path, PathBuf};

fn Context(generation: GenerationId) -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
        generation,
    };
}

const BETA_MANIFEST: &str = "[package]\nname = \"beta\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const ALPHA_MANIFEST_WITH_EDGE: &str =
    "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nbeta = { path = \"../beta\" }\n";
const ALPHA_MANIFEST_WITHOUT_EDGE: &str = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// A two-member fixture workspace, alpha depending on beta, removed when the test ends.
struct Fixture
{
    root: PathBuf,
}

impl Fixture
{
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-dependency-invalidation-{name}-{}", std::process::id()));

        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("alpha/src")).expect("a scratch alpha member");
        std::fs::create_dir_all(root.join("beta/src")).expect("a scratch beta member");

        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"alpha\", \"beta\"]\nresolver = \"2\"\n",
        )
        .expect("writing the workspace manifest");
        std::fs::write(root.join("alpha/Cargo.toml"), ALPHA_MANIFEST_WITH_EDGE).expect("writing alpha's manifest");
        std::fs::write(root.join("alpha/src/lib.rs"), "").expect("writing alpha's source");
        std::fs::write(root.join("beta/Cargo.toml"), BETA_MANIFEST).expect("writing beta's manifest");
        std::fs::write(root.join("beta/src/lib.rs"), "").expect("writing beta's source");

        return Self { root };
    }

    /// Rewrites alpha's manifest to drop its edge onto beta -- the edit this test invalidates
    /// a generation over.
    fn Drop_Alphas_Edge(&self)
    {
        std::fs::write(self.root.join("alpha/Cargo.toml"), ALPHA_MANIFEST_WITHOUT_EDGE)
            .expect("rewriting alpha's manifest");
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

#[test]
fn Test_An_Edited_Manifests_Old_Fact_Should_Not_Survive_The_Generation_It_Was_Invalidated_At()
{
    let fixture = Fixture::New("survive");
    let mut store = MemoryFactStore::New();

    let initial = Materialize_Workspace(fixture.Path(), Context(GenerationId::INITIAL), &StdProcessLauncher)
        .expect("a real cargo workspace with an edge");
    let alpha_before = initial
        .iter()
        .find(|package| package.path == "alpha")
        .expect("alpha is a workspace member");
    assert!(
        !alpha_before.fact.payload.bytes.is_empty(),
        "a real dependency payload was written"
    );
    let old_key = alpha_before.fact.Key().clone();
    let old_generation = alpha_before.fact.Generation();
    store
        .Materialize(alpha_before.fact.clone(), &[])
        .expect("the first generation's fact is never backdated");

    assert!(
        store.Current(&old_key.clone().At(old_generation), old_generation).is_some(),
        "the fact must be readable at the generation it was written"
    );

    fixture.Drop_Alphas_Edge();
    let next = GenerationId::INITIAL.Next();
    let report = store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: alpha_before.subject,
            granularity: IncrementalGranularity::Project,
        },
        next,
    );

    assert_eq!(
        report.direct,
        vec![old_key.clone()],
        "the edited package's own fact is what the cause names: {report:#?}"
    );
    assert!(
        store.Current(&old_key.clone().At(old_generation), next).is_none(),
        "the pre-edit fact must not survive as current at the generation it was invalidated at"
    );

    let (historical_fact, supersession) = store
        .Historical(&old_key)
        .expect("an invalidated fact is reported historically rather than disappearing");
    assert_eq!(historical_fact.Key(), &old_key);
    assert_eq!(
        supersession.invalidated_at, next,
        "the supersession must name the generation the edit was invalidated at"
    );

    let refreshed = Materialize_Workspace(fixture.Path(), Context(next), &StdProcessLauncher)
        .expect("a real cargo workspace with the edge removed");
    let alpha_after = refreshed
        .iter()
        .find(|package| package.path == "alpha")
        .expect("alpha is still a workspace member");
    let new_key = alpha_after.fact.Key().clone();
    store
        .Materialize(alpha_after.fact.clone(), &[])
        .expect("the later generation's fact is not backdated against the invalidated one");

    let current = store
        .Current(&new_key.clone().At(next), next)
        .expect("the re-materialized fact is current at the generation it was written");
    assert_eq!(
        current.payload.bytes, alpha_after.fact.payload.bytes,
        "the current fact at the new generation is the edited one, not the stale edge"
    );
    assert_ne!(
        current.payload.bytes, historical_fact.payload.bytes,
        "the edited manifest must actually have produced a different payload, or this test \
         proves nothing about which fact survived"
    );
}
