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

use crate::metadata::Workspace;
use std::path::{Path, PathBuf};

pub use nomos_rules::{DeclaredUniverse, UniverseKind};

/// Every declared universe in the workspace, derived from the source.
///
/// Deliberately over-inclusive. A list that turns out to need no mirror is cheap to
/// classify once; a list that is never surfaced is the defect this exists to prevent.
#[must_use]
pub fn Declared_Universes() -> Vec<DeclaredUniverse>
{
    let workspace = Workspace::Load();
    let root = Workspace::Workspace_Root();
    let mut universes = Vec::new();

    for member in workspace.Members()
    {
        for directory in ["src", "tests"]
        {
            let source_root = member.root.join(directory);
            if !source_root.is_dir()
            {
                continue;
            }

            for file in Source_Files(&source_root)
            {
                let Ok(text) = std::fs::read_to_string(&file)
                else
                {
                    continue;
                };

                let relative = file
                    .strip_prefix(&root)
                    .unwrap_or(&file)
                    .display()
                    .to_string()
                    .replace('\\', "/");

                // Through the provider and its encoding, because that is the subject the
                // rule is handed at run time. `nomos-rules` no longer parses — it reads a
                // syntax fact — so a walk that handed it text would be testing a signature
                // the product does not use. See `OD-SYNTAX-002`.
                let nomos_lang_rust::Reading::Parsed(facts) = nomos_lang_rust::Read_Source(&text)
                else
                {
                    continue;
                };

                let Ok(payload) =
                    nomos_cap_syntax::Parse_Payload(&nomos_lang_rust::Encode_Payload(&facts))
                else
                {
                    continue;
                };

                universes.extend(nomos_rules::Universes_In(&relative, &payload));
            }
        }
    }

    universes.sort();
    universes.dedup();
    return universes;
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
            let path = entry.path();
            if path.is_dir()
            {
                pending.push(path);
            }
            else if path.extension().is_some_and(|extension| extension == "rs")
            {
                files.push(path);
            }
        }
    }

    files.sort();
    return files;
}
