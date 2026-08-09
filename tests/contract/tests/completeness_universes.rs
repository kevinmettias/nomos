//! Every declared universe is classified, and the classification cannot go stale.
//!
//! `OD-COMPLETENESS-001`: a completeness guard is only as complete as the universe it
//! quantifies over, and when that universe is *declared* rather than derived, comparing the
//! declaration against reality is the other direction. Three guards in this workspace were
//! half-checks for exactly that reason, and all three were found by accident rather than by
//! looking.
//!
//! Discovery is mechanical and classification is not. `Declared_Universes` finds the lists;
//! whether a given list has a check comparing it against the reality it claims to enumerate
//! is a question about meaning, and this crate deliberately has no types to answer it with.
//! So the classification is declared in [`UNIVERSES`] below and checked against what is
//! derived — the shape `OD-GATE-001` already uses. Neither side is trusted alone, and in
//! particular the table cannot go stale in the direction that flatters: a new universe that
//! nobody classified fails [`Test_The_Declared_Table_Should_Match_What_Is_Derived`], and a
//! new *unmirrored* universe additionally fails
//! [`Test_The_Number_Of_Unmirrored_Universes_Should_Be_Declared`], so it cannot be added
//! quietly.
//!
//! # What this does not do
//!
//! It does not close the twelve holes it counts. A gate that can never be green is a gate
//! everybody learns to ignore, and twelve mirrors is not one item's work. What it does is
//! make the number a figure somebody chose rather than a silence nobody measured, which is
//! the same remedy `OD-GATE-001` applies to the corpus gates.

use nomos_contract_tests::{Declared_Universes, UniverseKind, Workspace};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// How a declared universe stands with respect to the rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Standing
{
    /// A check compares this declaration against the reality it claims to enumerate.
    Mirrored
    {
        /// The test that is that comparison. Asserted to exist.
        by: &'static str,
    },
    /// No such check. A declared hole, counted rather than hidden.
    Unmirrored
    {
        /// What would go wrong, so the next reader can judge whether to close it.
        risk: &'static str,
    },
}

/// One classified universe.
struct Universe
{
    /// Repo-relative, forward slashes. Matched against what is derived.
    path: &'static str,
    /// `GOVERNING_RECORD_IDS`, or `Table::All`.
    name: &'static str,
    /// How it is written down.
    kind: UniverseKind,
    /// Where it stands.
    standing: Standing,
}

/// How many universes have no mirror.
///
/// A number somebody chose. Raising it is the deliberate step that adding an unmirrored
/// universe is meant to cost, and lowering it is what closing one earns.
const UNMIRRORED_TOTAL: usize = 12;

/// Every declared universe in this workspace, classified by hand.
///
/// The three instances `OD-COMPLETENESS-001` analyses are the first three rows, and
/// [`Test_The_Three_Instances_Should_Have_Failed_This_Check`] removes their mirrors to
/// confirm each would have been caught here as originally written.
const UNIVERSES: &[Universe] = &[
    // ---- the three instances, now mirrored ----
    Universe {
        path: "crates/spec/nomos-spec-store/src/store.rs",
        name: "Table::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_Table_In_The_Schema_Should_Be_Declared",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-store/src/governing.rs",
        name: "GOVERNING_RECORD_IDS",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_Every_Canonical_Record_On_Disk_Should_Be_Governing",
        },
    },
    Universe {
        path: "tests/contract/src/gates.rs",
        name: "CORPUS_VARIABLES",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_The_Scanner_And_This_Table_Should_Name_The_Same_Variables",
        },
    },
    // ---- mirrored for other reasons ----
    Universe {
        path: "crates/spec/nomos-spec-validate/src/run.rs",
        name: "DECLARED_RULES",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_A_Rule_Nobody_Declared_Should_Fail_The_Run",
        },
    },
    // ---- declared holes ----
    Universe {
        path: "crates/kernel/nomos-store/src/document.rs",
        name: "DocumentKind::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a variant added without adding it here drops out of every guard built \
                   on All(), and OD-STORE-001 makes a document kind a behaviour",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/overlay.rs",
        name: "FILLER_PATTERNS",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "a heuristic blocklist with no reality to enumerate; OD-SPEC-004 already \
                   records that it missed the wording that hollowed v15",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/overlay.rs",
        name: "Family::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a family added without adding it here is never overlaid",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/restore.rs",
        name: "Restored::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a restored kind added without adding it here is never restored",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/siblings.rs",
        name: "Sibling::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a sibling suite added without adding it here is never run",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-model/src/table.rs",
        name: "RowKind::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a row kind added without adding it here escapes the row census, which \
                   D-132 makes the single home for those numbers",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-project/src/catalogue.rs",
        name: "SHIPPED",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "a profile that exists and is not shipped is invisible to `nomos spec \
                   profiles`, and nothing compares this against the renderers",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-project/src/profile.rs",
        name: "Content::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a content kind added without adding it here is never rendered",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-project/src/profile.rs",
        name: "Format::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a format added without adding it here is never offered",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-store/src/schema.rs",
        name: "MIGRATIONS",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "nothing compares the migration list against the schema it is supposed \
                   to produce, which is the same axis Table::All sits on",
        },
    },
    Universe {
        path: "crates/substrate/nomos-analysis/src/identity.rs",
        name: "Component::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a component added without adding it here is left out of fact identity, \
                   and OD-ANALYSIS-001 makes that a reuse defect",
        },
    },
    Universe {
        path: "crates/substrate/nomos-workspace/src/change.rs",
        name: "ChangeSource::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a change source added without adding it here is never walked",
        },
    },
];

// ---------------------------------------------------------------------------
// The table and the source must agree, in both directions.
// ---------------------------------------------------------------------------

/// This check is itself a completeness guard, and its universe is derived rather than
/// declared — so by `OD-COMPLETENESS-001`'s own corollary it owes nothing further on that
/// axis. The cost is paid by deriving.
#[test]
fn Test_The_Declared_Table_Should_Match_What_Is_Derived()
{
    // The kind is part of the identity, so a row that calls a constant an enumeration is a
    // mismatch rather than a harmless mislabel — the two carry different risks, and the
    // risk text is what the next reader acts on.
    let derived: BTreeSet<(String, String, UniverseKind)> = Declared_Universes()
        .into_iter()
        .map(|universe| return (universe.path, universe.name, universe.kind))
        .collect();

    let declared: BTreeSet<(String, String, UniverseKind)> = UNIVERSES
        .iter()
        .map(|universe| {
            return (
                universe.path.to_owned(),
                universe.name.to_owned(),
                universe.kind,
            );
        })
        .collect();

    assert!(
        !derived.is_empty(),
        "nothing was derived, so every assertion here would pass having read nothing"
    );

    let unclassified: Vec<&(String, String, UniverseKind)> =
        derived.difference(&declared).collect();
    assert!(
        unclassified.is_empty(),
        "these declared universes are not classified in UNIVERSES: {unclassified:#?}.\n\
         Add a row saying whether something compares the list against the reality it \
         claims to enumerate. If nothing does, say so and raise UNMIRRORED_TOTAL — a \
         universe nobody classified is the shape OD-COMPLETENESS-001 exists to stop."
    );

    let vanished: Vec<&(String, String, UniverseKind)> =
        declared.difference(&derived).collect();
    assert!(
        vanished.is_empty(),
        "these rows name universes that are no longer in the source: {vanished:#?}.\n\
         A table that keeps rows for things that are gone flatters itself in the other \
         direction."
    );
}

// ---------------------------------------------------------------------------
// A named mirror has to exist.
// ---------------------------------------------------------------------------

/// A row can claim a mirror that was renamed or deleted, and the table would still read as
/// though the universe were covered. That is the same defect one level up.
#[test]
fn Test_Every_Named_Mirror_Should_Exist_In_The_Source()
{
    let functions = Function_Names();
    assert!(
        !functions.is_empty(),
        "no functions were found, so this assertion would pass having read nothing"
    );

    let mut missing = Vec::new();
    for universe in UNIVERSES
    {
        if let Standing::Mirrored { by } = universe.standing
            && !functions.contains(by)
        {
            missing.push((universe.name, by));
        }
    }

    assert!(
        missing.is_empty(),
        "these rows name a mirror that does not exist: {missing:#?}.\n\
         Either the test was renamed, in which case update the row, or it was deleted, in \
         which case the universe is unmirrored and UNMIRRORED_TOTAL must rise."
    );
}

// ---------------------------------------------------------------------------
// The size of the hole is a number somebody chose.
// ---------------------------------------------------------------------------

#[test]
fn Test_The_Number_Of_Unmirrored_Universes_Should_Be_Declared()
{
    let unmirrored: Vec<&str> = Unmirrored_Names(UNIVERSES);

    assert_eq!(
        unmirrored.len(),
        UNMIRRORED_TOTAL,
        "{} declared universes have no mirror, and UNMIRRORED_TOTAL says {UNMIRRORED_TOTAL}: \
         {unmirrored:#?}.\n\
         Raising it is the deliberate step adding an unmirrored universe is meant to cost. \
         Lowering it is what writing a mirror earns.",
        unmirrored.len()
    );
}

// ---------------------------------------------------------------------------
// The item's own bar: the three instances would have failed this.
// ---------------------------------------------------------------------------

/// `P9-ONE-DIRECTION` requires that all three instances would have failed the enforcement
/// as originally written. They cannot be replayed — each was repaired at the site — so each
/// is reconstructed by taking its mirror away and asserting it lands in the counted hole.
///
/// Without this, a check that classified everything as mirrored would pass the tests above
/// and catch nothing.
#[test]
fn Test_The_Three_Instances_Should_Have_Failed_This_Check()
{
    for (name, mirror) in [
        (
            "Table::All",
            "Test_Every_Table_In_The_Schema_Should_Be_Declared",
        ),
        (
            "GOVERNING_RECORD_IDS",
            "Test_Every_Canonical_Record_On_Disk_Should_Be_Governing",
        ),
        (
            "CORPUS_VARIABLES",
            "Test_The_Scanner_And_This_Table_Should_Name_The_Same_Variables",
        ),
    ]
    {
        let row = UNIVERSES
            .iter()
            .find(|universe| return universe.name == name)
            .unwrap_or_else(|| panic!("{name} must be classified"));

        assert_eq!(
            row.standing,
            Standing::Mirrored { by: mirror },
            "{name} is expected to be mirrored by {mirror} today"
        );

        // As originally written, before that mirror existed.
        let before: Vec<&str> = UNIVERSES
            .iter()
            .filter(|universe| return universe.name != name)
            .map(|universe| {
                return match universe.standing
                {
                    Standing::Mirrored { .. } => "",
                    Standing::Unmirrored { .. } => universe.name,
                };
            })
            .filter(|entry| return !entry.is_empty())
            .collect();

        assert_eq!(
            before.len().saturating_add(1),
            UNMIRRORED_TOTAL.saturating_add(1),
            "removing {name}'s mirror must leave the count one above UNMIRRORED_TOTAL, \
             which is what would have failed"
        );
    }
}

/// The negative control for the test above. If every universe were classified unmirrored,
/// removing one mirror would change nothing and the reconstruction would prove nothing.
#[test]
fn Test_Some_Universes_Should_Actually_Be_Mirrored()
{
    let mirrored = UNIVERSES
        .iter()
        .filter(|universe| return matches!(universe.standing, Standing::Mirrored { .. }))
        .count();

    assert!(
        mirrored > 0,
        "if nothing is mirrored, this file measures nothing"
    );
    assert_eq!(
        mirrored.saturating_add(UNMIRRORED_TOTAL),
        UNIVERSES.len(),
        "every row must be one or the other"
    );
}

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

fn Unmirrored_Names(universes: &'static [Universe]) -> Vec<&'static str>
{
    return universes
        .iter()
        .filter(|universe| return matches!(universe.standing, Standing::Unmirrored { .. }))
        .map(|universe| return universe.name)
        .collect();
}

/// Every function name declared anywhere in the workspace's own sources.
fn Function_Names() -> BTreeSet<String>
{
    let workspace = Workspace::Load();
    let mut names = BTreeSet::new();

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

                for line in text.lines()
                {
                    if let Some(rest) = line.trim().strip_prefix("fn ")
                        && let Some((name, _)) = rest.split_once('(')
                    {
                        names.insert(name.trim().to_owned());
                    }
                }
            }
        }
    }

    return names;
}

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

    return files;
}
