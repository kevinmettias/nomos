//! The two roots every test works between, and the intents it declares.

use nomos_materialization::{MaterializationIntent, MaterializationRoots, OwnedRegion, OwnershipClass, PublicationScope};
use std::path::{Path, PathBuf};

/// Where sources are read from.
const SOURCE_ROOT: &str = "sources";
/// Where targets are written. A different root from the sources, so a test cannot pass by
/// accidentally reading what it just wrote.
const TARGET_ROOT: &str = "tree";

/// The roots every test hands the materializer.
pub(crate) fn Roots() -> MaterializationRoots
{
    return MaterializationRoots::New(SOURCE_ROOT, TARGET_ROOT);
}

/// One source path, resolved the way the materializer resolves it.
pub(crate) fn Source(relative: &str) -> PathBuf
{
    return Path::new(SOURCE_ROOT).join(relative);
}

/// One target path, resolved the way the materializer resolves it.
pub(crate) fn Target(relative: &str) -> PathBuf
{
    return Path::new(TARGET_ROOT).join(relative);
}

/// An intent placing `source` at `target` under one class, `Shared` and with no region.
pub(crate) fn Intent(source: &str, target: &str, ownership_class: OwnershipClass) -> MaterializationIntent
{
    return MaterializationIntent {
        surface: format!("the {target} surface"),
        source: source.to_owned(),
        target: target.to_owned(),
        ownership_class,
        publication_scope: PublicationScope::Shared,
        owned_region: None,
    };
}

/// The same intent, at another publication scope.
pub(crate) fn Scoped(intent: MaterializationIntent, publication_scope: PublicationScope) -> MaterializationIntent
{
    return MaterializationIntent { publication_scope, ..intent };
}

/// The same intent, declaring where its owned region sits.
pub(crate) fn In_Region(intent: MaterializationIntent, region: OwnedRegion) -> MaterializationIntent
{
    return MaterializationIntent { owned_region: Some(region), ..intent };
}
