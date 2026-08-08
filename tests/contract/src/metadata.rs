//! Reading the real dependency graph from cargo.
//!
//! Derived from `cargo metadata`, never from a hand-maintained list. A list of "who
//! depends on whom" checked into the repository would be correct on the day it was
//! written and wrong the first time somebody added a dependency without updating it —
//! and the failure mode of a stale allowlist is that it silently permits exactly the
//! thing it exists to forbid.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// One package in the workspace's dependency graph.
#[derive(Clone, Debug)]
pub struct Package
{
    /// The package name.
    pub name: String,
    /// Whether it is a member of this workspace, as opposed to a registry dependency.
    pub is_workspace_member: bool,
    /// The directory containing its manifest.
    pub root: PathBuf,
    /// The names it depends on directly, excluding dev-dependencies.
    ///
    /// Dev-dependencies are excluded from the band ordering deliberately. They do not
    /// ship, cargo permits them to be cyclic, and an upward one is legitimate — a low
    /// crate's tests may reasonably use a higher-level fixture. Counting them would
    /// force that fixture to be duplicated downward, which trades a real architectural
    /// property for a bookkeeping one.
    pub direct_dependencies: BTreeSet<String>,
}

/// The workspace's resolved dependency graph.
#[derive(Debug)]
pub struct Workspace
{
    packages: BTreeMap<String, Package>,
}

impl Workspace
{
    /// Loads the graph by running `cargo metadata`.
    ///
    /// # Panics
    ///
    /// Panics if cargo cannot be run or its output cannot be parsed. A boundary test
    /// that cannot see the graph must fail loudly: a check that silently verifies
    /// nothing reports the same clean result as a check that verified everything, and
    /// that is the failure mode these tests exist to prevent one level up.
    #[must_use]
    pub fn Load() -> Self
    {
        let output = std::process::Command::new(env!("CARGO"))
            .args([
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--all-features",
            ])
            .current_dir(Self::Workspace_Root())
            .output()
            .expect("cargo metadata must be runnable; a boundary test that cannot see the graph verifies nothing");

        assert!(
            output.status.success(),
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
            .expect("cargo metadata must emit valid JSON");

        let members: BTreeSet<String> = parsed
            .get("workspace_members")
            .and_then(serde_json::Value::as_array)
            .expect("workspace_members must be an array")
            .iter()
            .filter_map(|entry| entry.as_str())
            .filter_map(Self::Name_From_Package_Id)
            .collect();

        let mut packages = BTreeMap::new();
        for package in parsed
            .get("packages")
            .and_then(serde_json::Value::as_array)
            .expect("packages must be an array")
        {
            let Some(name) = package.get("name").and_then(serde_json::Value::as_str)
            else
            {
                continue;
            };

            let direct_dependencies = package
                .get("dependencies")
                .and_then(serde_json::Value::as_array)
                .map(|dependencies| {
                    dependencies
                        .iter()
                        .filter(|dependency| {
                            dependency.get("kind").and_then(serde_json::Value::as_str)
                                != Some("dev")
                        })
                        .filter_map(|dependency| {
                            dependency.get("name").and_then(serde_json::Value::as_str)
                        })
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();

            let root = package
                .get("manifest_path")
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from)
                .and_then(|path| path.parent().map(PathBuf::from))
                .unwrap_or_default();

            packages.insert(
                name.to_owned(),
                Package {
                    name: name.to_owned(),
                    is_workspace_member: members.contains(name),
                    root,
                    direct_dependencies,
                },
            );
        }

        assert!(
            !packages.is_empty(),
            "cargo metadata reported no packages; refusing to report a clean result over an \
             empty graph"
        );

        return Self { packages };
    }

    /// The workspace root directory.
    ///
    /// # Panics
    ///
    /// Panics if this crate is not two levels below the workspace root, which would
    /// mean the test crate had been moved without this being updated — and a boundary
    /// test pointed at the wrong tree is one that verifies nothing.
    #[must_use]
    pub fn Workspace_Root() -> PathBuf
    {
        // `tests/contract` is two levels below the root.
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(std::path::Path::parent)
            .map(PathBuf::from)
            .expect("the contract test crate sits two levels below the workspace root");
    }

    /// Every workspace member, in a stable order.
    #[must_use]
    pub fn Members(&self) -> Vec<&Package>
    {
        return self
            .packages
            .values()
            .filter(|package| package.is_workspace_member)
            .collect();
    }

    /// A package by name.
    ///
    /// # Panics
    ///
    /// Does not panic.
    #[must_use]
    pub fn Get(&self, name: &str) -> Option<&Package>
    {
        return self.packages.get(name);
    }

    /// Every package reachable from `name`, directly or transitively, excluding itself.
    ///
    /// Walks the graph rather than reading a declared list, so a dependency added three
    /// levels down is still visible here.
    #[must_use]
    pub fn Transitive_Dependencies(&self, name: &str) -> BTreeSet<String>
    {
        let mut reached = BTreeSet::new();
        let mut pending = vec![name.to_owned()];

        while let Some(current) = pending.pop()
        {
            let Some(package) = self.packages.get(&current)
            else
            {
                // A dependency cargo resolved but did not describe — a platform-specific
                // or optional edge. Recording the name is what matters here; we cannot
                // walk past it, and pretending it does not exist would understate reach.
                continue;
            };

            for dependency in &package.direct_dependencies
            {
                if reached.insert(dependency.clone())
                {
                    pending.push(dependency.clone());
                }
            }
        }

        reached.remove(name);
        return reached;
    }

    /// Extracts a package name from a cargo package id.
    ///
    /// Cargo has used several id formats. The modern one is a URL ending in `#name@version`
    /// or `#version` for path dependencies; the older one is `name version (source)`.
    /// Both are handled because a test that silently matched neither would report an
    /// empty member set and pass.
    fn Name_From_Package_Id(id: &str) -> Option<String>
    {
        if let Some((_, fragment)) = id.rsplit_once('#')
        {
            if let Some((name, _)) = fragment.rsplit_once('@')
            {
                return Some(name.to_owned());
            }
            if !fragment.starts_with(|character: char| character.is_ascii_digit())
            {
                return Some(fragment.to_owned());
            }
            // `…/nomos-model#0.1.0` — the name is the last path segment.
            let path = id.split('#').next()?;
            return path.rsplit('/').next().map(str::to_owned);
        }

        return id.split_whitespace().next().map(str::to_owned);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Package_Ids_Should_Parse_In_Every_Format_Cargo_Emits()
    {
        assert_eq!(
            Workspace::Name_From_Package_Id("path+file:///f/repos/nomos/crates/kernel/nomos-model#0.1.0"),
            Some("nomos-model".to_owned())
        );
        assert_eq!(
            Workspace::Name_From_Package_Id("registry+https://github.com/rust-lang/crates.io-index#serde@1.0.229"),
            Some("serde".to_owned())
        );
        assert_eq!(
            Workspace::Name_From_Package_Id("serde 1.0.229 (registry+https://github.com/rust-lang/crates.io-index)"),
            Some("serde".to_owned())
        );
    }
}
