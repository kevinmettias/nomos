//! What this crate promises, exercised at the two seams `nomos-cli` now calls through.
//!
//! Split by verb family, one child module each: `reads` covers the three read-only resolvers
//! and the two that hand a projection back, `render` covers the verbs that place or read a
//! governed output on disk, `authoring` covers `D-129`'s preview/commit round trip, and
//! `submit` covers the `Submit` sibling of `Run`. The corpus request, the scratch build root
//! and the embedded profile all four share live here.

mod authoring;
mod reads;
mod render;
mod submit;

use std::path::PathBuf;

use crate::corpus::CorpusRequest;

/// A corpus request naming no corpus at all, so every test in this module's children runs on
/// a machine that has never heard of the v14 corpus.
fn No_Corpus() -> CorpusRequest
{
    return CorpusRequest {
        variable: "A_SPEC_ORCHESTRATION_TEST_CORPUS_VARIABLE".to_owned(),
        root: None,
        revision: "v14.36".to_owned(),
    };
}

/// A build root under this process's own temporary directory, unique per test, so
/// `Render` and `Freshness` can be exercised against a real, disk-backed
/// `nomos-platform-std::StdFileSystem` the same way `nomos-cli` runs them.
fn Scratch(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!(
        "nomos-spec-orchestration-{name}-{}",
        std::process::id()
    ));
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch build root");

    return root;
}

/// A profile that builds from the embedded governing records alone, so a test naming it
/// needs no corpus -- the same profile `crates/host/nomos-cli/tests/read_surface.rs` uses
/// for the same reason.
const EMBEDDED_PROFILE: &str = "domain-specification";
