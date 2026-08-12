//! Every declared universe in this workspace, classified by hand.
//!
//! Discovery is mechanical and classification is not. `Declared_Universes` finds the lists;
//! whether a given list has a check comparing it against the reality it claims to enumerate
//! is a question about meaning, and this crate deliberately has no types to answer it with.
//! So the classification is declared here and checked against what is derived by
//! [`super::derivation`].

use nomos_contract_tests::UniverseKind;

/// How a declared universe stands with respect to the rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Standing
{
    /// A check compares this declaration against the reality it claims to enumerate.
    Mirrored
    {
        /// The test that is that comparison.
        ///
        /// A copy, not the claim. The claim lives in the universe's own doc comment, where
        /// `nomos check` can read it, and
        /// [`super::claims::Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror`] holds this
        /// it. Two places to spell one claim is how the two guards came to disagree about
        /// `DECLARED_RULES`; this is the one that is checked.
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
pub(crate) struct Universe
{
    /// Repo-relative, forward slashes. Matched against what is derived.
    pub(crate) path: &'static str,
    /// `GOVERNING_RECORD_IDS`, or `Table::All`.
    pub(crate) name: &'static str,
    /// How it is written down.
    pub(crate) kind: UniverseKind,
    /// Where it stands.
    pub(crate) standing: Standing,
}

/// How many universes have no mirror.
///
/// A number somebody chose. Raising it is the deliberate step that adding an unmirrored
/// universe is meant to cost, and lowering it is what closing one earns.
pub(crate) const UNMIRRORED_TOTAL: usize = 12;

/// Every declared universe in this workspace, classified by hand.
///
/// The three instances `OD-COMPLETENESS-001` analyses are the first three rows, and
/// [`super::hole_size::Test_The_Three_Instances_Should_Have_Failed_This_Check`] removes their
/// confirm each would have been caught here as originally written.
pub(crate) const UNIVERSES: &[Universe] = &[
    // ---- the three instances, now mirrored ----
    Universe {
        path: "crates/spec/nomos-spec-store/src/table.rs",
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
    //
    // `OD-COMPLETENESS-002`: this row named `Test_A_Rule_Nobody_Declared_Should_Fail_The_Run`,
    // a unit test over a synthetic rule, while the real reconciliation sat uncited in
    // `preservation_holds.rs`. The row was wrong about which test, never about the standing.
    Universe {
        path: "crates/spec/nomos-spec-validate/src/run.rs",
        name: "DECLARED_RULES",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_The_Registry_Should_Match_The_Manifest",
        },
    },
    // ---- declared holes ----
    Universe {
        path: "crates/kernel/nomos-store/src/document/kind.rs",
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
        path: "crates/spec/nomos-spec-ingest/src/family.rs",
        name: "Family::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a family added without adding it here is never overlaid",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/restored.rs",
        name: "Restored::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a restored kind added without adding it here is never restored",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/siblings/mod.rs",
        name: "Sibling::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a sibling suite added without adding it here is never run",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-model/src/row_kind.rs",
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
        path: "crates/spec/nomos-spec-project/src/content.rs",
        name: "Content::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a content kind added without adding it here is never rendered",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-project/src/format.rs",
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
        path: "crates/substrate/nomos-analysis/src/component.rs",
        name: "Component::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a component added without adding it here is left out of fact identity, \
                   and OD-ANALYSIS-001 makes that a reuse defect",
        },
    },
    Universe {
        path: "crates/substrate/nomos-workspace/src/change/source.rs",
        name: "ChangeSource::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Unmirrored {
            risk: "a change source added without adding it here is never walked",
        },
    },
];
