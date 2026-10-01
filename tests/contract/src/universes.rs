//! Declared universes — the lists a completeness guard quantifies over.
//!
//! `OD-COMPLETENESS-001` names the shape: a completeness guard is only as complete as the
//! universe it quantifies over, and when that universe is *declared* rather than derived,
//! comparing the declaration against reality is the other direction.
//!
//! # What is here and what moved
//!
//! Only the walk. Which files this crate looks at is a question about the workspace, and
//! answering it needs `cargo metadata`, which is this crate's whole job.
//!
//! Recognising a universe *in* a file moved to `nomos-rules`, where the same recognition
//! is what `nomos check` runs. It was implemented twice for about an hour — once here and
//! once there — and two guards for one rule is how they come to disagree, which this
//! workspace has already written down twice: `P9-PREDICATE` derived the gate's lint step
//! rather than copying it, for this reason, and `OD-COMPLETENESS-001` is itself about a
//! comparison that could not fail. A second scanner would have been a third instance of
//! the shape the record was written about.
//!
//! Sharing it also upgraded it. What this crate had was a line matcher; what it calls now
//! parses with `syn`, because a trimmed continuation line of a string literal looks
//! exactly like a declaration and the line matcher reported test fixtures as real
//! universes three runs running. `D-134` records why.
//!
//! The dependency direction is the safe one. `nomos-rules` sits at band 3 and knows
//! nothing about this crate; this crate sits at band 100 and observes it, along with
//! everything else in the workspace.

use std::path::{Path, PathBuf};

// check-dependency-placement reports three of this crate's edges -- nomos_rules,
// nomos_lang_rust and nomos_cap_syntax -- as existing for this file alone, and keeping it
// that way is deliberate. This crate watches the workspace from outside and compiles
// against almost nothing; the three are named here because a declared universe has to be
// read out of the crate that declares it, through the provider that parses it. Spreading
// them over more files would widen the only place the observer is also a participant.
pub use nomos_rules::{DeclaredUniverse, UniverseKind};

/// Every declared universe in the workspace, derived from the source.
///
/// Deliberately over-inclusive. A list that turns out to need no mirror is cheap to
/// classify once; a list that is never surfaced is the defect this exists to prevent.
#[must_use]
pub fn Declared_Universes() -> Vec<DeclaredUniverse>
{
    use crate::Workspace;

    let workspace = Workspace::Load();
    let root = Workspace::Workspace_Root();
    let mut universes = Vec::new();
    for member in workspace.Members()
    {
        for directory in ["src", "tests"]
        {
            let found = Universes_Under(&member.root.join(directory), &root);

            universes.extend(found);
        }
    }

    universes.sort();
    universes.dedup();
    return universes;
}

/// Every universe declared by any file under one source root.
fn Universes_Under(source_root: &Path, root: &Path) -> Vec<DeclaredUniverse>
{
    let mut universes = Vec::new();
    for file in Source_Files(source_root)
    {
        let found = Universes_In_File(&file, root);

        universes.extend(found);
    }

    return universes;
}

/// Every universe one file declares.
///
/// Read through the provider and its encoding, because that is the subject the rule is handed
/// at run time. `nomos-rules` no longer parses — it reads a syntax fact — so a walk that
/// handed it text would be testing a signature the product does not use. See `OD-SYNTAX-002`.
fn Universes_In_File(file: &Path, root: &Path) -> Vec<DeclaredUniverse>
{
    let Ok(text) = std::fs::read_to_string(file)
    else
    {
        return Vec::new();
    };
    let nomos_lang_rust::Reading::Parsed(facts) = nomos_lang_rust::Read_Source(&text)
    else
    {
        return Vec::new();
    };
    let encoded = nomos_lang_rust::Encode_Payload(&facts);
    let Ok(payload) = nomos_cap_syntax::Parse_Payload(&encoded)
    else
    {
        return Vec::new();
    };
    let relative = Relative_To(root, file);

    return nomos_rules::Universes_In(&relative, &payload);
}

/// A file's repo-relative path with forward slashes, which is how a rule names its subject.
fn Relative_To(root: &Path, file: &Path) -> String
{
    return file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string()
        .replace('\\', "/");
}

/// Every `.rs` file under a directory.
fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };
        for entry in entries.flatten()
        {
            Keep_Or_Descend(&entry.path(), &mut pending, &mut files);
        }
    }

    files.sort();
    return files;
}

/// A directory to walk later, a Rust file to keep, or neither.
fn Keep_Or_Descend(path: &Path, pending: &mut Vec<PathBuf>, files: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path.to_path_buf());
    }
    else if path.extension().is_some_and(|extension| extension == "rs")
    {
        files.push(path.to_path_buf());
    }
}
