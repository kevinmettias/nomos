//! Reading `nomos-test-material.json` into a repository's declared fixture locations.
//!
//! # Why its own file, and not `standards.json`
//!
//! The five `OD-RULES-011` modules beside this one read `standards.json`, and this one cannot.
//! That file is shared: `code-standards` decodes the whole of it into one struct with unknown
//! fields disallowed, and measured against the live checkout on 2026-09-14, every one of this
//! repository's thirteen top-level keys is a field that struct names. A repository's own
//! fixture locations have no such key to piggyback on. `OD-HOST-009` settled where a
//! repository-declared criterion without a `code-standards` key travels: a dedicated,
//! language-neutral file at the repository root, the convention `nomos-gate.json` and
//! `nomos-architecture.json` already set.
//!
//! A Cargo `workspace.metadata` table is wrong for the same reason it is for architecture:
//! nomos judges Go repositories, and a declaration only a Cargo workspace could carry would
//! make this family Rust-only.
//!
//! `fixture_locations` is an array because each entry is a repository-relative directory
//! prefix and the list is one flat set the whole workspace shares — a location has no scope
//! to qualify, the same "bare list" shape `nomos_cap_words_policy` already carries for a
//! repository's own vocabulary additions.

use nomos_cap_test_material_policy::TestMaterialPolicyPayload;
use nomos_platform::{FileSystem, FileSystemError};
use std::path::Path;

/// The file a repository declares its fixture locations in.
///
/// `nomos-<concern>.json` at the repository root, the convention
/// `nomos_gate_orchestration`'s own `nomos-gate.json` already set for a file this workspace
/// owns outright, as against `standards.json`, which it shares.
pub const TEST_MATERIAL_JSON: &str = "nomos-test-material.json";

/// `nomos-test-material.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestMaterialPolicyError
{
    pub reason: String,
}

impl core::fmt::Display for TestMaterialPolicyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// The fixture locations `root`'s own `nomos-test-material.json` declares — declaring nothing
/// when the file is absent or declares no `fixture_locations` key, since an unconfigured
/// repository is not a repository this capability failed to read.
///
/// # Errors
///
/// [`TestMaterialPolicyError`] if `nomos-test-material.json` exists but could not be read for
/// a reason other than absence, is not valid JSON, or declares `fixture_locations` as
/// something other than an array of strings.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<TestMaterialPolicyPayload, TestMaterialPolicyError>
{
    let path = root.join(TEST_MATERIAL_JSON);
    let text = match filesystem.Read_To_String(&path)
    {
        Ok(text) => text,
        // An absent file is a repository that has not declared its fixture locations, which
        // is a real answer rather than a read this provider failed at.
        Err(FileSystemError::NotFound { .. }) => return Ok(TestMaterialPolicyPayload::default()),
        Err(error) => return Err(TestMaterialPolicyError { reason: format!("{TEST_MATERIAL_JSON} could not be read: {error}") }),
    };

    let declared: serde_json::Value = serde_json::from_str(&text).map_err(|error| TestMaterialPolicyError {
        reason: format!("{TEST_MATERIAL_JSON} is not valid JSON: {error}"),
    })?;

    return Declared_Locations(&declared);
}

/// The declared document itself, once it is known to be present and parsed.
fn Declared_Locations(declared: &serde_json::Value) -> Result<TestMaterialPolicyPayload, TestMaterialPolicyError>
{
    let Some(listed) = declared.get("fixture_locations")
    else
    {
        return Ok(TestMaterialPolicyPayload::default());
    };

    let Some(entries) = listed.as_array()
    else
    {
        return Err(TestMaterialPolicyError {
            reason: format!("{TEST_MATERIAL_JSON}'s fixture_locations is not an array"),
        });
    };

    let mut locations = Vec::new();
    for entry in entries
    {
        let Some(location) = entry.as_str()
        else
        {
            return Err(TestMaterialPolicyError {
                reason: format!("{TEST_MATERIAL_JSON}'s fixture_locations has a non-string entry"),
            });
        };
        locations.push(location.to_owned());
    }

    // Sorted so the encoded bytes do not depend on a JSON array's own order, the same
    // canonical order the five siblings impose on their own rows for the identical reason.
    locations.sort();

    return Ok(TestMaterialPolicyPayload { locations });
}

#[cfg(test)]
#[path = "reading/tests.rs"]
mod tests;
