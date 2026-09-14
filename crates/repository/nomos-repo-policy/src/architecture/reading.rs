//! Reading `nomos-architecture.json` into a declaration.
//!
//! Five keys, each a plain JSON shape a person writes by hand:
//!
//! ```json
//! {
//!   "components": ["Domain", "Infrastructure", "Api"],
//!   "members":    { "billing": "Domain", "postgres": "Infrastructure" },
//!   "permits":    { "Api": ["Domain"], "Infrastructure": ["Domain"] },
//!   "exceptions": { "billing": ["billing-core"] },
//!   "authorities": { "ledger-store": ["billing"] }
//! }
//! ```
//!
//! # Why its own file, and not `standards.json`
//!
//! The five `OD-RULES-011` modules beside this one read `standards.json`, and this one cannot.
//! That file is shared: `code-standards` decodes the whole of it into one struct with unknown
//! fields disallowed, and measured against the live checkout on 2026-09-14, every one of this
//! repository's thirteen top-level keys is a field that struct names. The five families are
//! not a counterexample -- each reads a key `code-standards` already owns and decodes, so
//! nomos piggybacks on that schema rather than extending it. A declared architecture has no
//! such key to piggyback on, and the one that looks closest, `tiers`, is the one
//! `OD-RULES-029`'s amendment forbids by name as another tool's declared input.
//!
//! A Cargo `workspace.metadata` table is the idiomatic Rust extension point and is wrong for a
//! different reason: nomos judges Go repositories, and a declaration only a Cargo workspace
//! could carry would make this family Rust-only -- a worse failure of the property this
//! provider exists for than the collision it avoids. So: a dedicated, language-neutral file at
//! the repository root, named by the convention `nomos-gate.json` already set.
//!
//! `components` is an array because its order is the repository's own and a reader is shown
//! it; the other four are objects because each is a lookup keyed by a name, and an object is
//! how a person writes a lookup. Nothing here interprets a component's spelling: the strings
//! are carried through to [`nomos_cap_architecture::ArchitecturePayload`] as written, which is
//! what makes a repository dividing itself into `Domain`/`Infrastructure`/`Api` expressible
//! without this crate knowing any of those words.

use nomos_cap_architecture::{ArchitecturePayload, Authority, Exception, Membership, Permission};
use nomos_platform::{FileSystem, FileSystemError};
use std::path::Path;

/// The file a repository declares its architecture in.
///
/// `nomos-<concern>.json` at the repository root, the convention
/// `nomos_gate_orchestration`'s own `nomos-gate.json` already set for a file this workspace
/// owns outright, as against `standards.json`, which it shares.
pub const ARCHITECTURE_JSON: &str = "nomos-architecture.json";

/// `nomos-architecture.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchitectureError
{
    pub reason: String,
}

impl core::fmt::Display for ArchitectureError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// The architecture `root`'s own `nomos-architecture.json` declares — an empty declaration
/// when the file is absent.
///
/// An empty declaration is the honest answer for a repository that has not taken architecture
/// conformance on, and `OD-RULES-003` decided what the rules owe it: a positive statement that
/// they do not bind, never a gap reported against every member. That is the case the compiled
/// table could not express at all.
///
/// # Errors
///
/// [`ArchitectureError`] if `nomos-architecture.json` exists but could not be read for a
/// reason other than absence, is not valid JSON, or declares a shape this reader does not
/// recognize — a `components` value that is not an array of strings, or any of the four
/// lookups holding something other than a string or an array of strings.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<ArchitecturePayload, ArchitectureError>
{
    let path = root.join(ARCHITECTURE_JSON);
    let text = match filesystem.Read_To_String(&path)
    {
        Ok(text) => text,
        // An absent file is a repository that has not declared an architecture, which is a
        // real answer rather than a read this provider failed at.
        Err(FileSystemError::NotFound { .. }) => return Ok(ArchitecturePayload::default()),
        Err(error) => return Err(ArchitectureError { reason: format!("{ARCHITECTURE_JSON} could not be read: {error}") }),
    };

    let declared: serde_json::Value = serde_json::from_str(&text).map_err(|error| ArchitectureError {
        reason: format!("{ARCHITECTURE_JSON} is not valid JSON: {error}"),
    })?;

    return Declared_Architecture(&declared);
}

/// The declared document itself, once it is known to be present and parsed.
fn Declared_Architecture(declared: &serde_json::Value) -> Result<ArchitecturePayload, ArchitectureError>
{
    let components = Components(declared)?;
    let membership = Membership_Rows(declared)?;
    let permissions = Permission_Rows(declared)?;
    let exceptions = Exception_Rows(declared)?;
    let authorities = Authority_Rows(declared)?;

    return Ok(ArchitecturePayload { components, membership, permissions, exceptions, authorities });
}

/// `components`, in the order the repository wrote them.
fn Components(declared: &serde_json::Value) -> Result<Vec<String>, ArchitectureError>
{
    let Some(listed) = declared.get("components")
    else
    {
        return Ok(Vec::new());
    };

    let Some(entries) = listed.as_array()
    else
    {
        return Err(Refusal("components is not an array"));
    };

    return entries.iter().map(|entry| return Name(entry, "components")).collect();
}

/// `members`, sorted by package so the encoding does not depend on a JSON object's own order.
fn Membership_Rows(declared: &serde_json::Value) -> Result<Vec<Membership>, ArchitectureError>
{
    let Some(listed) = declared.get("members")
    else
    {
        return Ok(Vec::new());
    };

    let Some(entries) = listed.as_object()
    else
    {
        return Err(Refusal("members is not an object"));
    };

    let mut rows = Vec::new();
    for (package, component) in entries
    {
        let component = Name(component, "members")?;
        rows.push(Membership { package: package.clone(), component });
    }
    rows.sort_by(|left, right| return left.package.cmp(&right.package));

    return Ok(rows);
}

/// `permits`, flattened from `component -> [component]` into one row per admitted pair.
fn Permission_Rows(declared: &serde_json::Value) -> Result<Vec<Permission>, ArchitectureError>
{
    let pairs = Pairs(declared, "permits")?;

    return Ok(pairs
        .into_iter()
        .map(|(from, to)| return Permission { from, to })
        .collect());
}

/// `exceptions`, flattened from `package -> [package]` into one row per named pair.
fn Exception_Rows(declared: &serde_json::Value) -> Result<Vec<Exception>, ArchitectureError>
{
    let pairs = Pairs(declared, "exceptions")?;

    return Ok(pairs
        .into_iter()
        .map(|(from, to)| return Exception { from, to })
        .collect());
}

/// `authorities`, each with its own doors, both sorted.
fn Authority_Rows(declared: &serde_json::Value) -> Result<Vec<Authority>, ArchitectureError>
{
    let Some(listed) = declared.get("authorities")
    else
    {
        return Ok(Vec::new());
    };

    let Some(entries) = listed.as_object()
    else
    {
        return Err(Refusal("authorities is not an object"));
    };

    let mut rows = Vec::new();
    for (package, doors) in entries
    {
        let mut doors = Names(doors, "authorities")?;
        doors.sort();
        rows.push(Authority { package: package.clone(), doors });
    }
    rows.sort_by(|left, right| return left.package.cmp(&right.package));

    return Ok(rows);
}

/// One `key -> [name]` block, flattened and sorted into `(from, to)` pairs.
///
/// Shared by `permits` and `exceptions` because the *shape* is one shape, while what a pair
/// means stays each caller's own: [`Permission`] is between components and [`Exception`] is
/// between packages, which is why the two rows are built by the two functions above rather
/// than here.
fn Pairs(declared: &serde_json::Value, key: &str) -> Result<Vec<(String, String)>, ArchitectureError>
{
    let Some(listed) = declared.get(key)
    else
    {
        return Ok(Vec::new());
    };

    let Some(entries) = listed.as_object()
    else
    {
        return Err(ArchitectureError {
            reason: format!("{ARCHITECTURE_JSON}'s {key} is not an object"),
        });
    };

    let mut pairs = Vec::new();
    for (from, targets) in entries
    {
        for to in Names(targets, &format!("{key}"))?
        {
            pairs.push((from.clone(), to));
        }
    }
    pairs.sort();

    return Ok(pairs);
}

/// An array of names, refused when it is not one.
fn Names(value: &serde_json::Value, key: &str) -> Result<Vec<String>, ArchitectureError>
{
    let Some(entries) = value.as_array()
    else
    {
        return Err(ArchitectureError {
            reason: format!("{ARCHITECTURE_JSON}'s {key} holds a value that is not an array of names"),
        });
    };

    return entries.iter().map(|entry| return Name(entry, key)).collect();
}

/// One name, refused when it is not a string.
fn Name(value: &serde_json::Value, key: &str) -> Result<String, ArchitectureError>
{
    let Some(name) = value.as_str()
    else
    {
        return Err(ArchitectureError {
            reason: format!("{ARCHITECTURE_JSON}'s {key} holds a value that is not a name"),
        });
    };

    return Ok(name.to_owned());
}

/// A refusal naming one key, for the cases whose message needs nothing but the key.
fn Refusal(what: &str) -> ArchitectureError
{
    return ArchitectureError {
        reason: format!("{ARCHITECTURE_JSON}'s {what}"),
    };
}

#[cfg(test)]
mod tests;
