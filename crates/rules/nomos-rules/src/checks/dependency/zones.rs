//! This workspace's own declared architecture: eleven named zones, which crate belongs to
//! each, which zones a zone may reach, and the few same-zone edges a real crate needs.
//!
//! `OD-RULES-020` decided the shape: a total order over band numbers forces two crates
//! with no real precedence between them to be given one anyway, the moment either
//! legitimately depends on the other — `nomos-gate-orchestration`'s move from band 40 to
//! band 41, and `nomos-workflow-orchestration`'s three renumbers across two items in one
//! session, both did nothing but satisfy the arithmetic. Zones replace the number line:
//! a crate belongs to exactly one zone, a zone may depend only on the zones named in
//! [`Permits`], and two crates in the same zone are peers by default — neither may name
//! the other — unless the pair appears in [`SAME_ZONE_EDGES`].
//!
//! This is the one declaration [`super::violations`] and [`super::completeness`] read
//! inside this crate, and the one `tests/contract/tests/boundaries/bands.rs` and `graph.rs`
//! read from outside it, through `nomos-rules`' own public surface. `OD-RULES-020`'s own
//! "What This Does Not Do" named the population above as its own proposal rather than a
//! line-by-line audit result; this migration carries that population over unchanged; a
//! misclassification found later is fixed here, the same way a wrong band entry was.

/// One of this workspace's eleven architectural zones.
///
/// Ordered so that `derive(PartialOrd)` is never reached for — a zone's own position in
/// this list means nothing; [`Permits`] is the only source of truth for what a zone may
/// reach, so that adding a zone here can never silently change what an existing zone is
/// allowed to depend on the way inserting a band number could always shift a comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone
{
    /// The shared identity and protocol vocabulary. `nomos-contracts` alone.
    Protocol,
    /// Foundational primitives: subjects, documents, ports, snapshots, exclusion, the
    /// capability registry, the fact store.
    Substrate,
    /// The specification database and its own orchestration layer — sits beside the
    /// kernel rather than above it, per `ARC-ECOSYSTEM-001` and `OD-PROJECT-004`.
    Specification,
    /// A capability's own contract: the ten `nomos-cap-*` crates.
    CapabilityContract,
    /// One provider against one capability contract, or a package manifest format.
    Provider,
    /// Pure judgment functions over facts. `nomos-rules` alone.
    Rules,
    /// Correction planning and agent execution.
    Agent,
    /// The seams composing capability, provider, rule and agent facts into a real
    /// `nomos check` / `nomos gate` / `nomos workflow` run.
    ApplicationService,
    /// Development and preservation machinery for this repository itself, per
    /// `OD-PROJECT-004`: not a Nomos product feature.
    RepoTooling,
    /// The composition roots: a caller-facing binary or transport.
    Host,
    /// Observes the whole workspace. Nothing observes it.
    Verification,
}

/// Every zone this workspace declares, for a caller that must enumerate them all —
/// `tests/contract/tests/boundaries/readme.rs` parses `README.md`'s own table back out and
/// needs the reverse of [`Zone`]'s `Display`: which zone, if any, a cell's text names.
///
/// Mirrored by `Test_Every_Zone_Should_Be_Matched_Exhaustively`: an exhaustive match over
/// every variant with no wildcard arm, below. It fails to compile, not merely to pass, if a
/// variant is added to [`Zone`] without being added there.
pub const ALL: [Zone; 11] = [
    Zone::Protocol,
    Zone::Substrate,
    Zone::Specification,
    Zone::CapabilityContract,
    Zone::Provider,
    Zone::Rules,
    Zone::Agent,
    Zone::ApplicationService,
    Zone::RepoTooling,
    Zone::Host,
    Zone::Verification,
];

impl core::fmt::Display for Zone
{
    /// The name a reader sees — in a `Finding`'s own summary, and in `README.md`'s table,
    /// which is checked against this exact spelling rather than against `Debug`'s
    /// unspaced variant name.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        let name = match self
        {
            Self::Protocol => "Protocol",
            Self::Substrate => "Substrate",
            Self::Specification => "Specification",
            Self::CapabilityContract => "Capability Contract",
            Self::Provider => "Provider",
            Self::Rules => "Rules",
            Self::Agent => "Agent",
            Self::ApplicationService => "Application Service",
            Self::RepoTooling => "Repo Tooling",
            Self::Host => "Host",
            Self::Verification => "Verification",
        };

        return write!(formatter, "{name}");
    }
}

/// Every workspace member's own zone.
///
/// Mirrored by `Test_Every_Member_Should_Declare_A_Band`, in
/// `tests/contract/tests/boundaries/graph.rs`: every real workspace member `cargo metadata`
/// reports must resolve to a zone here.
///
/// Carried over from the band table this replaces without re-deriving it: `OD-RULES-020`
/// measured `nomos-rules`' own Provider exclusion and the six orchestration crates' real
/// dependencies directly, and proposed the rest from the band clusters they were already
/// drawn from. A crate found here to be misclassified is fixed in this table, the same way
/// a wrong band number was fixed in `BANDS`.
pub const ZONES: &[(&str, Zone)] = &[
    ("nomos-contracts", Zone::Protocol),
    ("nomos-model", Zone::Substrate),
    ("nomos-store", Zone::Substrate),
    ("nomos-platform", Zone::Substrate),
    ("nomos-platform-std", Zone::Substrate),
    ("nomos-workspace", Zone::Substrate),
    ("nomos-scope-verification", Zone::Substrate),
    ("nomos-capability", Zone::Substrate),
    ("nomos-analysis", Zone::Substrate),
    ("nomos-spec-model", Zone::Specification),
    ("nomos-spec-store", Zone::Specification),
    ("nomos-spec-bundle", Zone::Specification),
    ("nomos-spec-ingest", Zone::Specification),
    ("nomos-spec-validate", Zone::Specification),
    ("nomos-spec-project", Zone::Specification),
    ("nomos-spec-orchestration", Zone::Specification),
    ("nomos-cap-syntax", Zone::CapabilityContract),
    ("nomos-cap-dependency", Zone::CapabilityContract),
    ("nomos-cap-controlflow", Zone::CapabilityContract),
    ("nomos-cap-lint", Zone::CapabilityContract),
    ("nomos-cap-dependency-policy", Zone::CapabilityContract),
    ("nomos-cap-naming-policy", Zone::CapabilityContract),
    ("nomos-cap-limits-policy", Zone::CapabilityContract),
    ("nomos-cap-scripting-policy", Zone::CapabilityContract),
    ("nomos-cap-words-policy", Zone::CapabilityContract),
    ("nomos-cap-goals-policy", Zone::CapabilityContract),
    // A connector under ARC-CONNECTOR-001, not a nomos-cap-* crate -- but classified
    // Capability Contract zone rather than Provider zone anyway, because it bundles
    // nomos.cap.review.finding's contract with its one provider in one crate
    // (OD-CAPABILITY-002 licenses this while there is only one provider) and Rules zone
    // may not name Provider zone at all. Provider zone would leave this fact family
    // structurally unreachable by nomos-rules, not merely misfiled.
    ("nomos-connector-coderabbit", Zone::CapabilityContract),
    // Also bundles a single provider with its contract in one crate (OD-CAPABILITY-002),
    // the identical reason nomos-connector-coderabbit does one row above -- reading
    // arbitrary files across the repository tree needs a nomos_platform::FileSystem, which
    // Permits forbids Rules zone from reaching except through a Capability Contract zone
    // crate.
    ("nomos-cap-requirement-trace", Zone::CapabilityContract),
    ("nomos-package", Zone::Provider),
    ("nomos-lang-rust", Zone::Provider),
    ("nomos-lang-rust-scan", Zone::Provider),
    ("nomos-lang-go", Zone::Provider),
    ("nomos-lang-go-modules", Zone::Provider),
    ("nomos-lang-rust-cargo", Zone::Provider),
    ("nomos-lang-rust-clippy", Zone::Provider),
    ("nomos-lang-rust-deny", Zone::Provider),
    ("nomos-lang-rust-compiler", Zone::Provider),
    ("nomos-repo-policy", Zone::Provider),
    ("nomos-lang-rust-package", Zone::Provider),
    ("nomos-lang-go-package", Zone::Provider),
    ("nomos-model-package", Zone::Provider),
    ("nomos-rule-package", Zone::Provider),
    ("nomos-rules", Zone::Rules),
    ("nomos-corrections", Zone::Agent),
    ("nomos-agent-contracts", Zone::Agent),
    ("nomos-agent-executor-claude-code", Zone::Agent),
    ("nomos-model-backend-ollama", Zone::Agent),
    ("nomos-check-orchestration", Zone::ApplicationService),
    ("nomos-gate-orchestration", Zone::ApplicationService),
    ("nomos-correction-orchestration", Zone::ApplicationService),
    ("nomos-workflow-orchestration", Zone::ApplicationService),
    ("nomos-agent-orchestration", Zone::ApplicationService),
    ("nomos-ledger", Zone::RepoTooling),
    ("nomos-work-orchestration", Zone::RepoTooling),
    ("nomos-surface-provenance", Zone::RepoTooling),
    ("nomos-cli", Zone::Host),
    ("nomos-api", Zone::Host),
    ("nomos-api-transport", Zone::Host),
    ("nomos-mcp", Zone::Host),
    ("nomos-lsp", Zone::Host),
    ("nomos-contract-tests", Zone::Verification),
    ("nomos-integration-tests", Zone::Verification),
];

/// Two crates in the same zone that are not peers: `from` may name `to`.
///
/// Mirrored by `Test_Every_Same_Zone_Edge_Should_Be_A_Real_Dependency`, in
/// `tests/contract/tests/boundaries/graph.rs`: every pair named here must be a real,
/// direct `Cargo.toml` dependency, or the exception permits an edge nobody's code draws.
///
/// `OD-RULES-020`'s own worked example measured this for `Application Service` alone and
/// named it as the one zone needing this; this migration measured every zone directly
/// against its members' own `Cargo.toml` files and found six more carry the identical real
/// internal structure the band numbers used to encode one crate at a time — `OD-RULES-020`
/// is corrected on that point by its own migration item rather than left standing on an
/// incomplete measurement. Adding a new same-zone edge is still a decision with the same
/// weight as widening `graph.rs`'s own `PLATFORM_ADAPTER` — named here, not inferred from a
/// crate compiling.
pub const SAME_ZONE_EDGES: &[(&str, &str)] = &[
    // Substrate: measured directly against every member's own Cargo.toml, not proposed
    // from a band cluster. A zone this size has real internal build-up — subjects before
    // documents, ports before their std implementation, the model and the store before
    // anything that snapshots or verifies over them — the same structure the band numbers
    // 10 through 22 used to carry one crate at a time.
    ("nomos-store", "nomos-model"),
    ("nomos-platform-std", "nomos-platform"),
    ("nomos-workspace", "nomos-model"),
    ("nomos-workspace", "nomos-store"),
    ("nomos-scope-verification", "nomos-model"),
    ("nomos-analysis", "nomos-model"),
    ("nomos-analysis", "nomos-capability"),
    // Specification: the family ARC-ECOSYSTEM-001 and OD-PROJECT-004 place beside the
    // kernel, with its own real internal layering (normalizer, then store, then bundle
    // and ingest over the store, then validate and project over ingest).
    ("nomos-spec-store", "nomos-spec-model"),
    ("nomos-spec-bundle", "nomos-spec-model"),
    ("nomos-spec-bundle", "nomos-spec-store"),
    ("nomos-spec-ingest", "nomos-spec-model"),
    ("nomos-spec-ingest", "nomos-spec-store"),
    ("nomos-spec-validate", "nomos-spec-model"),
    ("nomos-spec-validate", "nomos-spec-store"),
    ("nomos-spec-validate", "nomos-spec-ingest"),
    ("nomos-spec-project", "nomos-spec-model"),
    ("nomos-spec-project", "nomos-spec-store"),
    ("nomos-spec-orchestration", "nomos-spec-model"),
    ("nomos-spec-orchestration", "nomos-spec-store"),
    ("nomos-spec-orchestration", "nomos-spec-ingest"),
    ("nomos-spec-orchestration", "nomos-spec-project"),
    // Provider: the package-manifest crates wrap nomos-package's generic core and, for the
    // two language packages, their own language's syntax/dependency providers.
    ("nomos-lang-rust-package", "nomos-package"),
    ("nomos-lang-rust-package", "nomos-lang-rust"),
    ("nomos-lang-rust-package", "nomos-lang-rust-scan"),
    ("nomos-lang-go-package", "nomos-package"),
    ("nomos-lang-go-package", "nomos-lang-go"),
    ("nomos-model-package", "nomos-package"),
    ("nomos-rule-package", "nomos-package"),
    // Agent: TaskEnvelope/WorkResult naming a correction candidate a rule produced, and
    // both AgentExecutors depending on the contracts they implement.
    ("nomos-agent-contracts", "nomos-corrections"),
    ("nomos-agent-executor-claude-code", "nomos-agent-contracts"),
    ("nomos-model-backend-ollama", "nomos-agent-contracts"),
    // Application Service: OD-RULES-020's own worked measurement — gate and correction
    // each reach check, and workflow dispatches to all three.
    ("nomos-gate-orchestration", "nomos-check-orchestration"),
    ("nomos-correction-orchestration", "nomos-check-orchestration"),
    ("nomos-workflow-orchestration", "nomos-check-orchestration"),
    ("nomos-workflow-orchestration", "nomos-correction-orchestration"),
    ("nomos-workflow-orchestration", "nomos-gate-orchestration"),
    // Repo Tooling: nomos-work-orchestration dispatches to the ledger it coordinates.
    ("nomos-work-orchestration", "nomos-ledger"),
    // Host: the transport layers each wrap the composition root or transport beneath them.
    ("nomos-api-transport", "nomos-api"),
    ("nomos-mcp", "nomos-api-transport"),
];

/// This workspace member's own zone, or `None` if it is not one this table names.
#[must_use]
pub fn Zone_Of(name: &str) -> Option<Zone>
{
    return ZONES
        .iter()
        .find(|(crate_name, _)| *crate_name == name)
        .map(|(_, zone)| *zone);
}

/// Whether a crate in `from` may depend on a crate in `to`, by zone alone — same-zone
/// pairs are [`SAME_ZONE_EDGES`]'s own question, not this function's, and calling it with
/// `from == to` always answers `false` for exactly that reason: two crates sharing a zone
/// are peers unless a named exception says otherwise, never zone membership alone.
#[must_use]
pub fn Permits(from: Zone, to: Zone) -> bool
{
    use Zone::{Agent, ApplicationService, CapabilityContract, Host, Protocol, Provider, RepoTooling, Rules, Specification, Substrate, Verification};

    return match from
    {
        Protocol => false,
        Substrate => matches!(to, Protocol),
        Specification | CapabilityContract => matches!(to, Protocol | Substrate),
        Provider | Rules => matches!(to, Protocol | Substrate | CapabilityContract),
        // Measured directly, not proposed: nomos-agent-contracts and nomos-agent-executor-
        // claude-code both depend on nomos-model-package for the package-kind vocabulary
        // AGT-002's WorkResult and OD-EXECUTOR-001's executor boundary carry.
        Agent => matches!(to, Protocol | Substrate | Provider),
        RepoTooling => matches!(to, Protocol | Substrate),
        ApplicationService => matches!(to, Protocol | Substrate | CapabilityContract | Provider | Rules | Agent),
        Host => matches!(
            to,
            Protocol | Substrate | Specification | CapabilityContract | Provider | Rules | Agent | ApplicationService | RepoTooling
        ),
        Verification => !matches!(to, Verification),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Every_Entry_Should_Be_Findable_By_Name()
    {
        for (name, zone) in ZONES
        {
            assert_eq!(Zone_Of(name), Some(*zone), "{name}");
        }
    }

    #[test]
    fn Test_An_Unknown_Name_Should_Have_No_Declared_Zone()
    {
        assert_eq!(Zone_Of("nomos-does-not-exist"), None);
    }

    #[test]
    fn Test_Display_Should_Space_A_Multi_Word_Zone_Name()
    {
        assert_eq!(Zone::CapabilityContract.to_string(), "Capability Contract");
        assert_eq!(Zone::ApplicationService.to_string(), "Application Service");
        assert_eq!(Zone::RepoTooling.to_string(), "Repo Tooling");
    }

    #[test]
    fn Test_Every_Zone_Should_Round_Trip_Through_Its_Own_Display_Text()
    {
        for zone in ALL
        {
            let text = zone.to_string();
            let found = ALL.iter().find(|candidate| candidate.to_string() == text);

            assert_eq!(found, Some(&zone), "{text}");
        }
    }

    /// `ALL`'s own mirror. An exhaustive match with no wildcard arm fails to compile, not
    /// merely to pass, the moment a variant is added to `Zone` without a matching arm here —
    /// the same guard `ExitCode::All()` uses.
    #[test]
    fn Test_Every_Zone_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal(zone: Zone) -> usize
        {
            return match zone
            {
                Zone::Protocol => 0,
                Zone::Substrate => 1,
                Zone::Specification => 2,
                Zone::CapabilityContract => 3,
                Zone::Provider => 4,
                Zone::Rules => 5,
                Zone::Agent => 6,
                Zone::ApplicationService => 7,
                Zone::RepoTooling => 8,
                Zone::Host => 9,
                Zone::Verification => 10,
            };
        }

        for (index, zone) in ALL.iter().enumerate()
        {
            assert_eq!(
                Ordinal(*zone),
                index,
                "{zone:?} is not matched at the position ALL puts it, so the exhaustive \
                 match and the universe have drifted apart"
            );
        }
    }

    #[test]
    fn Test_A_Zone_Should_Never_Permit_Itself()
    {
        for zone in ALL
        {
            assert!(!Permits(zone, zone), "{zone:?} must not permit itself; same-zone edges are SAME_ZONE_EDGES's own question");
        }
    }

    #[test]
    fn Test_Protocol_Should_Permit_Nothing()
    {
        assert!(!Permits(Zone::Protocol, Zone::Substrate));
    }

    #[test]
    fn Test_Application_Service_Should_Reach_Every_Zone_It_Composes()
    {
        for to in [Zone::Protocol, Zone::Substrate, Zone::CapabilityContract, Zone::Provider, Zone::Rules, Zone::Agent]
        {
            assert!(Permits(Zone::ApplicationService, to), "{to:?}");
        }

        assert!(!Permits(Zone::ApplicationService, Zone::Host));
        assert!(!Permits(Zone::ApplicationService, Zone::RepoTooling));
    }

    #[test]
    fn Test_Same_Zone_Edges_Should_Each_Name_Two_Members_Of_The_Same_Zone()
    {
        for (from, to) in SAME_ZONE_EDGES
        {
            let from_zone = Zone_Of(from).unwrap_or_else(|| panic!("{from} has no declared zone"));
            let to_zone = Zone_Of(to).unwrap_or_else(|| panic!("{to} has no declared zone"));

            assert_eq!(from_zone, to_zone, "{from} -> {to}: a same-zone edge must name two members of one zone");
        }
    }
}
