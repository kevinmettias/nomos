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
pub(crate) const UNMIRRORED_TOTAL: usize = 6;

/// Every declared universe in this workspace, classified by hand.
///
/// The three instances `OD-COMPLETENESS-001` analyses are the first three rows, and
/// [`super::hole_size::Test_The_Three_Instances_Should_Have_Failed_This_Check`] removes their
/// confirm each would have been caught here as originally written.
pub(crate) const UNIVERSES: &[Universe] = &[
    // ---- the rule population, mirrored against what a run actually composes ----
    Universe {
        path: "crates/rules/nomos-rules/src/rule_descriptor.rs",
        name: "DESCRIPTORS",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_Every_Composed_Rule_Should_Have_A_Descriptor",
        },
    },
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
        path: "crates/spec/nomos-spec-store/src/store/seed_report.rs",
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
    // `Backend::ALL` arrived here already claiming this standing: its doc comment said a
    // variant missing from the array is why `Test_Every_Variant_Should_Be_Listed` exists
    // beside it, and that test did not. `P122` wrote it rather than reclassifying the
    // universe, because the row and the claim are two spellings of one thing and the claim
    // was the one already committed.
    Universe {
        path: "crates/orchestration/nomos-agent-orchestration/src/backend.rs",
        name: "ALL",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_Every_Variant_Should_Be_Listed",
        },
    },
    // ---- mirrored for other reasons ----
    //
    // `OD-COMPLETENESS-002`: this row named `Test_A_Rule_Nobody_Declared_Should_Fail_The_Run`,
    // a unit test over a synthetic rule, while the real reconciliation sat uncited in
    // `preservation_holds.rs`. The row was wrong about which test, never about the standing.
    Universe {
        path: "crates/spec/nomos-spec-validate/src/validation_run.rs",
        name: "DECLARED_RULES",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_The_Registry_Should_Match_The_Manifest",
        },
    },
    // ---- declared holes ----
    Universe {
        path: "crates/kernel/nomos-store/src/document/kind.rs",
        name: "Kind::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_All_Should_Enumerate_Every_Kind_Exhaustively",
        },
    },
    // `OD-GATE-004`: this was a private census array (`Every_Exit_Code`) in
    // crates/host/nomos-cli/src/check/tests.rs, kept private because promoting it to
    // ExitCode::All() needed exactly this row, in a file outside that item's territory.
    Universe {
        path: "crates/host/nomos-cli/src/check/exit_code.rs",
        name: "ExitCode::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_ExitCode_Should_Be_Matched_Exhaustively",
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
        path: "crates/spec/nomos-spec-ingest/src/siblings/sibling.rs",
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
        path: "crates/spec/nomos-spec-store/src/migration.rs",
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
        path: "crates/substrate/nomos-workspace/src/change_source.rs",
        name: "ChangeSource::All",
        kind: UniverseKind::Enumeration,
        standing: Standing::Mirrored {
            by: "Test_Every_ChangeSource_Should_Be_Matched_Exhaustively",
        },
    },
    // `P73-LSP-CORRECTION-FAMILY-DUPLICATED-2` exported this list so `nomos-lsp` could stop
    // keeping its own copy of the correction-family membership, and introduced a universe
    // with it. Scanned as `Constant` rather than `Enumeration` because it is a `const` array
    // rather than an `All()` function, but it enumerates a closed enum and takes the same
    // exhaustive-match mirror every `All` above it does.
    Universe {
        path: "crates/orchestration/nomos-correction-orchestration/src/correction_family.rs",
        name: "ALL",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_Every_Correction_Family_Should_Be_Matched_Exhaustively",
        },
    },
    Universe {
        path: "crates/orchestration/nomos-workspace-discovery/src/lib.rs",
        name: "SCRIPT_EXTENSIONS",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "check-script-discipline's own five recognized script extensions, restated \
                   as a literal rather than read from standards.json's own forbidden_extensions \
                   -- the same risk the three original per-host copies this constant replaces \
                   already carried, uncaught until this constant's arrival made it module-level \
                   and this scanner's own subject; a real script extension standards.json adds \
                   is not picked up here until somebody notices and extends this list by hand",
        },
    },
    Universe {
        path: "crates/packages/nomos-lang-rust-package/src/known_providers.rs",
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
    Universe {
        path: "crates/packages/nomos-lang-go-package/src/known_providers.rs",
        name: "KNOWN_PROVIDERS",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "a second Go provider crate this package could register (of the same \
                   nomos.cap.syntax.items capability, since nomos-lang-go-modules is \
                   deliberately excluded here) is not added to this list automatically, the \
                   identical risk nomos-lang-rust-package's own row states — bounded the same way, \
                   to additions rather than drift on the one entry it holds today, pulled \
                   from nomos-lang-go's own PROVIDER constant rather than retyped",
        },
    },
    Universe {
        path: "crates/packages/nomos-lang-csharp-package/src/known_providers.rs",
        name: "KNOWN_PROVIDERS",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "a second C# provider crate this package could register is not added to this \
                   list automatically, the identical risk nomos-lang-rust-package's and \
                   nomos-lang-go-package's own rows state — bounded the same way, to \
                   additions rather than drift on the one entry it holds today, pulled from \
                   nomos-lang-csharp's own PROVIDER constant rather than retyped. Narrower \
                   than either sibling's today, because this workspace has no second C# \
                   provider of any capability to have been left out",
        },
    },
    Universe {
        path: "crates/packages/nomos-tool-package/src/known_providers.rs",
        name: "KNOWN_PROVIDERS",
        kind: UniverseKind::Constant,
        standing: Standing::Unmirrored {
            risk: "a third ToolProvider crate added to the workspace is not added to this list \
                   automatically, so its registration is refused by Providers_Field until \
                   somebody notices and extends it by hand — the identical risk \
                   nomos-lang-rust-package's and nomos-lang-go-package's own rows state — \
                   bounded the same way, to additions rather than drift on the two entries it \
                   holds today, pulled from nomos-lang-rust-clippy's and nomos-lang-rust-deny's \
                   own PROVIDER constants rather than retyped",
        },
    },
    // ---- what an external surface over nomos-api may serve ----
    Universe {
        path: "crates/host/nomos-api-transport/src/served_method.rs",
        name: "REGISTRY",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_The_Transport_Should_Name_No_Repo_Tooling_Handler",
        },
    },
    Universe {
        path: "crates/host/nomos-mcp/src/served_tool.rs",
        name: "REGISTRY",
        kind: UniverseKind::Constant,
        standing: Standing::Mirrored {
            by: "Test_The_Tool_Registry_Should_Name_The_Same_Operations_As_The_Served_Method_Registry",
        },
    },
    // ---- OD-RULES-020's dependency model is no longer a constant of this workspace's ----
    // ---- own, so it is no longer a declared universe here ----
    //
    // ZONES, SAME_ZONE_EDGES, ALL and WRITE_DOORS were four entries above this line. They were
    // `const` tables in `nomos-rules`, and `OD-RULES-003`'s third prerequisite moved every one
    // of them into `nomos-architecture.json`, which is data a repository authors rather than a
    // list this workspace compiles. A universe is a closed set *this source* declares; a
    // declaration another repository can replace wholesale is not one, and listing it here
    // would claim a completeness obligation over content nobody here writes.
    //
    // The mirrors did not go with them. `Test_Every_Member_Should_Declare_A_Band`,
    // `Test_Every_Same_Zone_Edge_Should_Be_A_Real_Dependency`,
    // `Test_Every_Write_Door_Should_Be_A_Real_Dependency` and
    // `Test_Same_Zone_Edges_Should_Each_Name_Two_Members_Of_The_Same_Zone` all still run, in
    // `boundaries/graph.rs`, and still compare the declaration against what `cargo metadata`
    // reports. What changed is which artifact they read, not whether anything checks it.
];
