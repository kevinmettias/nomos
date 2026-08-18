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
pub(crate) const UNMIRRORED_TOTAL: usize = 2;

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
        path: "crates/spec/nomos-spec-store/src/store/governing.rs",
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
        standing: Standing::Mirrored {
            by: "Test_Every_DocumentKind_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/reconciliation/overlay.rs",
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
        standing: Standing::Mirrored {
            by: "Test_Every_Family_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/reconciliation/restored.rs",
        name: "Restored::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_Restored_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-ingest/src/siblings.rs",
        name: "Sibling::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_Sibling_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-model/src/table/row_kind.rs",
        name: "RowKind::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_RowKind_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-project/src/catalogue.rs",
        name: "SHIPPED",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_Every_Profile_File_Should_Be_Shipped",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-project/src/projection/content.rs",
        name: "Content::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_Content_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-project/src/projection/format.rs",
        name: "Format::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_Format_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/spec/nomos-spec-store/src/schema.rs",
        name: "MIGRATIONS",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_Every_Table_In_The_Schema_Should_Be_Declared",
        },
    },
    Universe {
        path: "crates/substrate/nomos-analysis/src/component.rs",
        name: "Component::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_Component_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/substrate/nomos-workspace/src/change/source.rs",
        name: "ChangeSource::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_ChangeSource_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/packages/nomos-lang-package/src/known_providers.rs",
        name: "KNOWN_PROVIDERS",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "a third Rust language provider crate added to the workspace is not added \
                   to this list automatically, so its registrations are refused by \
                   Providers_Field until somebody notices and extends it by hand — the two \
                   entries it holds today are pulled from nomos-lang-rust's and \
                   nomos-lang-rust-scan's own PROVIDER constants rather than retyped, which \
                   bounds the risk to additions rather than drift on the existing two",
        },
    },
];
