//! The capability families a rule reads facts from, and their canonical identifiers.
//!
//! Its own file rather than a second type beside [`super::RuleDescriptor`]: the one-public-type
//! rule wants one file-home type per file, and `nomos-lsp` and `nomos-check-orchestration` both
//! name this type without naming a descriptor at all.

use nomos_contracts::CapabilityId;

/// A capability family a rule reads facts from.
///
/// A closed set naming the families this workspace has, rather than a string: each variant
/// resolves through the capability crate that owns the contract, so the identifier is
/// spelled once, where it is declared, and not again here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredFact
{
    SyntaxItems,
    DependencyEdges,
    LintDiagnostics,
    DependencyPolicy,
    Reachability,
    NamingPolicy,
    LimitsPolicy,
    ScriptingPolicy,
    GoalsPolicy,
    WordsPolicy,
    TestMaterialPolicy,
    ReviewFindings,
    RequirementTrace,
    ArchitectureDeclaration,
    /// Every `.clone()` call an analyzed crate makes on a value whose type a real compiler
    /// frontend resolved to already implement `Copy`.
    ///
    /// The first of the two families whose provider is a compiler frontend rather than a
    /// parser, a manifest reader, a subprocess or a declaration file. Nothing here says so
    /// -- a variant names the capability and not the mechanism behind it -- but it is worth
    /// knowing that a run demanding either of these two loads `ra_ap_hir` and every crate
    /// reachable from the analyzed root, which is why a selection that demands neither must
    /// not pay for them. `OD-ANALYSIS-007`.
    CopyClones,
    /// Every `std::sync::Mutex<T>` or `RwLock<T>` an analyzed crate declares whose `T` a
    /// real compiler frontend resolved to another lock type.
    NestedLocks,
}

impl RequiredFact
{
    /// The canonical identifier of the capability this family names.
    #[must_use]
    pub fn Capability(self) -> CapabilityId
    {
        return match self
        {
            Self::SyntaxItems => nomos_cap_syntax::Capability(),
            Self::DependencyEdges => nomos_cap_dependency::Capability(),
            Self::LintDiagnostics => nomos_cap_lint::Capability(),
            Self::DependencyPolicy => nomos_cap_dependency_policy::Capability(),
            Self::Reachability => nomos_cap_controlflow::Capability(),
            Self::NamingPolicy => nomos_cap_naming_policy::Capability(),
            Self::LimitsPolicy => nomos_cap_limits_policy::Capability(),
            Self::ScriptingPolicy => nomos_cap_scripting_policy::Capability(),
            Self::GoalsPolicy => nomos_cap_goals_policy::Capability(),
            Self::WordsPolicy => nomos_cap_words_policy::Capability(),
            Self::TestMaterialPolicy => nomos_cap_test_material_policy::Capability(),
            Self::ReviewFindings => nomos_cap_review_finding::Capability(),
            Self::RequirementTrace => nomos_cap_requirement_trace::Capability(),
            Self::ArchitectureDeclaration => nomos_cap_architecture::Capability(),
            Self::CopyClones => nomos_cap_rust_copy_clones::Capability(),
            Self::NestedLocks => nomos_cap_rust_nested_locks::Capability(),
        };
    }
}
