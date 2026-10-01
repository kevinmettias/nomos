//! Reading `nomos-csharp-builds.json` into the builds a repository wants its C# judged under.
//!
//! # Why a repository declares them
//!
//! Which branch of a C# `#if` chain compiles depends on the build: a configuration's `DEBUG`, a
//! target framework's `NET8_0_OR_GREATER`, a project's own `DefineConstants`. A repository builds
//! some of those combinations and not others, and nothing in its files says which it ships:
//! a project may list three frameworks and ship two, and every project has a `Debug` and a
//! `Release` whether or not anybody builds both. So the builds are declared, and never guessed
//! -- a guessed build would judge a branch dead that the repository's real build compiles.
//!
//! # Why its own file
//!
//! `OD-RULES-035` section 4: a declaration is readable only from a place the repository can
//! write it, and `standards.json` is decoded strictly by code-standards, which names no such key.
//! So it is `nomos-<concern>.json` at the root, the convention `OD-HOST-009` set and
//! `nomos-architecture.json`, `nomos-test-material.json` and `nomos-limits.json` follow.
//!
//! # Why it is not a fact, unlike every other module here
//!
//! Each sibling module is the provider of a capability a rule reads, because `OD-RULES-011` sends a
//! repository's configuration to a rule as a fact. This declaration does not reach a rule. It
//! reaches the composition, which asks `MSBuild` about exactly the builds declared and hands the
//! rule the answers; a rule that read the declaration too would hold a second copy of which builds
//! were judged, and could disagree with the facts it was given. What it shares with its siblings is
//! the acquisition -- a file a repository owns, read through a [`FileSystem`] port -- and that is
//! why it is here.
//!
//! # Strict
//!
//! A key this reader does not know is refused, not skipped, at every level: the file is Nomos's
//! own, so an unread key is a declaration that will not be applied -- the equality `OD-RULES-035`
//! section 5 holds this workspace's own files to. A misspelled `target_framwork` would otherwise
//! silently judge a multi-targeting project under no framework at all.

use nomos_platform::{FileSystem, FileSystemError};
use std::collections::BTreeSet;
use std::path::Path;

/// The file a repository declares its C# builds in.
pub const CSHARP_BUILDS_JSON: &str = "nomos-csharp-builds.json";

/// One build a repository declares: a project, a configuration and, for a project that targets
/// more than one framework, which of them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredBuild
{
    /// The project file, relative to the repository root with forward slashes.
    pub project: String,
    /// The build configuration, such as `Release`.
    pub configuration: String,
    /// The target framework, or `None` for a project that targets exactly one -- the evaluation
    /// refuses a project that targets several with none named, rather than picking one.
    pub target_framework: Option<String>,
}

/// `nomos-csharp-builds.json` could not be read as a declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildsDeclarationError
{
    pub reason: String,
}

impl core::fmt::Display for BuildsDeclarationError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// The builds `root`'s `nomos-csharp-builds.json` declares, each once and in a fixed order --
/// none when the file is absent, since a repository that declares nothing has not declared
/// something this reader failed to read.
///
/// # Errors
///
/// [`BuildsDeclarationError`] when the file exists and cannot be read, is not JSON, is not an
/// object holding one `builds` array of build objects, names a key this reader does not know, or
/// gives a build a field of the wrong kind, an empty one, or a project path that is not relative.
pub fn Read_Declared_Builds<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<Vec<DeclaredBuild>, BuildsDeclarationError>
{
    let text = match filesystem.Read_To_String(&root.join(CSHARP_BUILDS_JSON))
    {
        Ok(text) => text,
        Err(FileSystemError::NotFound { .. }) => return Ok(Vec::new()),
        Err(error) => return Err(Refused(&format!("could not be read: {error}"))),
    };
    let declared: serde_json::Value = serde_json::from_str(&text).map_err(|error| return Refused(&format!("is not valid JSON: {error}")))?;
    let document = declared.as_object().ok_or_else(|| return Refused("is not a JSON object"))?;
    Refuse_Unknown_Keys(document, &["builds"], "the top level")?;
    let Some(listed) = document.get("builds")
    else
    {
        return Ok(Vec::new());
    };
    let entries = listed.as_array().ok_or_else(|| return Refused("has a `builds` that is not an array"))?;

    let mut builds = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate()
    {
        builds.insert(Declared_Build(entry, index)?);
    }

    return Ok(builds.into_iter().collect());
}

/// One entry of `builds`, the `index`th.
fn Declared_Build(entry: &serde_json::Value, index: usize) -> Result<DeclaredBuild, BuildsDeclarationError>
{
    let place = format!("build {index}");
    let fields = entry.as_object().ok_or_else(|| return Refused(&format!("has a {place} that is not an object")))?;
    Refuse_Unknown_Keys(fields, &["project", "configuration", "target_framework"], &place)?;

    let project = Required_Text(fields, "project", &place)?;
    if project.starts_with('/') || project.contains('\\') || project.contains(':')
    {
        return Err(Refused(&format!("has a {place} whose project `{project}` is not a path relative to the repository root with forward slashes")));
    }
    let configuration = Required_Text(fields, "configuration", &place)?;
    let target_framework = match fields.get("target_framework")
    {
        None => None,
        Some(_) => Some(Required_Text(fields, "target_framework", &place)?),
    };

    return Ok(DeclaredBuild { project, configuration, target_framework });
}

/// `key`'s value in `fields`, which must be present and a non-empty string.
fn Required_Text(fields: &serde_json::Map<String, serde_json::Value>, key: &str, place: &str) -> Result<String, BuildsDeclarationError>
{
    let value = fields.get(key).ok_or_else(|| return Refused(&format!("has a {place} with no `{key}`")))?;
    let text = value.as_str().ok_or_else(|| return Refused(&format!("has a {place} whose `{key}` is not a string")))?;
    if text.trim().is_empty()
    {
        return Err(Refused(&format!("has a {place} whose `{key}` is empty")));
    }

    return Ok(text.to_owned());
}

fn Refuse_Unknown_Keys(fields: &serde_json::Map<String, serde_json::Value>, known: &[&str], place: &str) -> Result<(), BuildsDeclarationError>
{
    return match fields.keys().find(|key| return !known.contains(&key.as_str()))
    {
        Some(unknown) => Err(Refused(&format!("names `{unknown}` at {place}, which is not a key this declaration has, so it would not be applied"))),
        None => Ok(()),
    };
}

fn Refused(what: &str) -> BuildsDeclarationError
{
    return BuildsDeclarationError { reason: format!("{CSHARP_BUILDS_JSON} {what}") };
}

#[cfg(test)]
#[path = "csharp_builds/tests.rs"]
mod tests;
