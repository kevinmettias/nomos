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

use nomos_analysis::{FactKey, FactStore, GenerationCause, InvalidationReport, MaterializedFact, MemoryFactStore};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, Digest128, GenerationId, IncrementalGranularity, SnapshotId, SubjectId,
};
use nomos_lang_rust_cargo::{FactContext, Materialize_Workspace, PackageFact};
use nomos_platform_std::{StdEnvironment, StdProcessLauncher};
use std::path::{Path, PathBuf};

/// Fill bytes distinct enough that the three digests below differ from one another; each
/// value carries no meaning beyond "not equal to the others".
const VARIANT_DIGEST_FILL: u8 = 2;
const CONFIGURATION_DIGEST_FILL: u8 = 3;

fn Context(generation: GenerationId) -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_DIGEST_FILL; Digest128::BYTE_LENGTH])),
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

/// Alpha's first fact as the store took it: the key it is addressed by, the generation it
/// was written at, and the subject the edit will later invalidate -- the three things every
/// assertion below reads the store back by, named rather than returned as a bare tuple.
struct Initial
{
    key: FactKey,
    generation: GenerationId,
    subject: SubjectId,
}

/// The refreshed fact and the key it is addressed by, for the same reason as [`Initial`].
struct Refreshed
{
    key: FactKey,
    fact: PackageFact,
}

#[test]
fn Test_An_Edited_Manifests_Old_Fact_Should_Not_Survive_The_Generation_It_Was_Invalidated_At()
{
    let fixture = Fixture::New("survive");
    let mut store = MemoryFactStore::New();

    let Initial { key: old_key, generation: old_generation, subject: alpha_subject } = Materialize_And_Store_Initial(&fixture, &mut store);
    Assert_Initial_Fact_Readable(&store, &old_key, old_generation);

    let next = GenerationId::INITIAL.Next();
    let report = Drop_Edge_And_Invalidate(&fixture, &mut store, alpha_subject, next);
    Assert_Invalidation_Report(&report, &old_key);
    Assert_Pre_Edit_Fact_Is_Not_Current(&store, &old_key, old_generation, next);

    let historical_fact = Assert_Historical_Fact(&store, &old_key, next);

    let Refreshed { key: new_key, fact: alpha_after } = Materialize_And_Store_Refresh(&fixture, &mut store, next);
    let current = Refreshed_Current(&store, &new_key, next);
    Assert_Refreshed_Payload(&current, &alpha_after);
    Assert_Stale_Payload_Differs(&current, &historical_fact);
}

/// Materializes alpha's fact at the initial generation, stores it, and returns its key,
/// generation, and subject — everything the rest of this test invalidates and re-reads by.
fn Materialize_And_Store_Initial(fixture: &Fixture, store: &mut MemoryFactStore) -> Initial
{
    let initial = Materialize_Workspace(fixture.Path(), Context(GenerationId::INITIAL), &StdProcessLauncher, &StdEnvironment)
        .expect("a real cargo workspace with an edge");
    let alpha_before = initial
        .iter()
        .find(|package| package.path == "alpha")
        .expect("alpha is a workspace member");
    assert!(
        !alpha_before.fact.payload.bytes.is_empty(),
        "a real dependency payload was written"
    );
    let key = alpha_before.fact.Key().clone();
    let generation = alpha_before.fact.Generation();
    store
        .Materialize(alpha_before.fact.clone(), &[])
        .expect("the first generation's fact is never backdated");

    return Initial { key, generation, subject: alpha_before.subject };
}

fn Assert_Initial_Fact_Readable(store: &MemoryFactStore, old_key: &FactKey, old_generation: GenerationId)
{
    assert!(
        store.Current(&old_key.clone().At(old_generation), old_generation).is_some(),
        "the fact must be readable at the generation it was written"
    );
}

/// Edits alpha's manifest on disk and invalidates the subject the edit changed, returning
/// the store's report of what that invalidation reached.
fn Drop_Edge_And_Invalidate(
    fixture: &Fixture,
    store: &mut MemoryFactStore,
    subject: SubjectId,
    next: GenerationId,
) -> InvalidationReport
{
    fixture.Drop_Alphas_Edge();

    return store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject,
            granularity: IncrementalGranularity::Project,
        },
        next,
    );
}

/// What the invalidation reached, from the store's own report.
fn Assert_Invalidation_Report(report: &InvalidationReport, old_key: &FactKey)
{
    assert_eq!(
        report.direct,
        vec![old_key.clone()],
        "the edited package's own fact is what the cause names: {report:#?}"
    );
}

/// And what it left behind: the pre-edit fact is gone as the *current* fact, though not as
/// history -- [`Assert_Historical_Fact`] is the other half of that same claim.
fn Assert_Pre_Edit_Fact_Is_Not_Current(store: &MemoryFactStore, old_key: &FactKey, old_generation: GenerationId, next: GenerationId)
{
    assert!(
        store.Current(&old_key.clone().At(old_generation), next).is_none(),
        "the pre-edit fact must not survive as current at the generation it was invalidated at"
    );
}

fn Assert_Historical_Fact(store: &MemoryFactStore, old_key: &FactKey, next: GenerationId) -> MaterializedFact
{
    let (historical_fact, supersession) = store
        .Historical(old_key)
        .expect("an invalidated fact is reported historically rather than disappearing");
    assert_eq!(historical_fact.Key(), old_key);
    assert_eq!(
        supersession.invalidated_at, next,
        "the supersession must name the generation the edit was invalidated at"
    );

    return historical_fact;
}

/// Re-runs `cargo metadata` at the post-edit generation, stores the refreshed fact, and
/// returns its key alongside the fact itself.
fn Materialize_And_Store_Refresh(fixture: &Fixture, store: &mut MemoryFactStore, next: GenerationId) -> Refreshed
{
    let refreshed = Materialize_Workspace(fixture.Path(), Context(next), &StdProcessLauncher, &StdEnvironment)
        .expect("a real cargo workspace with the edge removed");
    let alpha_after = refreshed
        .into_iter()
        .find(|package| package.path == "alpha")
        .expect("alpha is still a workspace member");
    let key = alpha_after.fact.Key().clone();
    store
        .Materialize(alpha_after.fact.clone(), &[])
        .expect("the later generation's fact is not backdated against the invalidated one");

    return Refreshed { key, fact: alpha_after };
}

/// The fact the store itself calls current at `next`, addressed by `key` -- read back out
/// of the store rather than taken from the value the caller just wrote.
fn Refreshed_Current(store: &MemoryFactStore, key: &FactKey, next: GenerationId) -> MaterializedFact
{
    return store
        .Current(&key.clone().At(next), next)
        .expect("the re-materialized fact is current at the generation it was written");
}

/// The current fact at the new generation is the edited one.
fn Assert_Refreshed_Payload(current: &MaterializedFact, alpha_after: &PackageFact)
{
    assert_eq!(
        current.payload.bytes, alpha_after.fact.payload.bytes,
        "the current fact at the new generation is the edited one, not the stale edge"
    );
}

/// And it differs from what the pre-edit generation held -- without this the other half
/// proves nothing about which fact survived.
fn Assert_Stale_Payload_Differs(current: &MaterializedFact, historical_fact: &MaterializedFact)
{
    assert_ne!(
        current.payload.bytes, historical_fact.payload.bytes,
        "the edited manifest must actually have produced a different payload, or this test proves nothing about which fact survived"
    );
}
