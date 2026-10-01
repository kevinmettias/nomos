//! The agreement itself.
//!
//! Below every party to it: the provider that answers `nomos.cap.rust.nested_locks`
//! (`nomos-lang-rust-compiler`) and the rule that reads it (`nomos-rules`) both import this
//! module rather than either one naming the other.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate carries the contract for.
///
/// Named for what a caller gets -- a lock guarding a value that is itself already behind
/// a lock -- rather than for the mechanism (`ra_ap_hir`) that answers it, the same reason
/// `nomos_cap_rust_copy_clones::CAPABILITY` is named for its own answer rather than for the
/// engine behind it: a second provider answering the same question through a different
/// compiler frontend must be able to name this capability honestly.
pub const CAPABILITY: &str = "nomos.cap.rust.nested_locks";

/// The payload schema every answer to this capability is stamped with.
pub const SCHEMA: &str = "nomos.rust.nested_locks.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::SemanticallyResolved`]: whether a `Mutex<T>`/`RwLock<T>`'s own `T` is
/// itself a `Mutex`/`RwLock` only has an answer once `T` is resolved past whatever type
/// alias, re-export, or generic substitution stands between the syntax and the real type
/// -- a syntax tree alone (`FactVariant::Syntactic`) sees the name written at the nesting
/// site, not the type it names, and this capability's own fixture
/// (`nomos-lang-rust-compiler`'s `fixtures/nested_lock_sample`) is built specifically so its
/// one real positive case is invisible without resolving a type alias.
///
/// Soundness [`Assurance::Sound`] at the ceiling: every finding this capability's real
/// provider reports names an outer lock whose type argument a real compiler frontend
/// actually resolved to another lock type, never one inferred from a name that merely
/// looks like `Mutex` or `RwLock`.
///
/// Completeness [`Assurance::Unknown`]: this ceiling leaves room for a stronger future
/// provider than today's one real answer honestly claims -- see
/// `nomos_lang_rust_compiler::Nested_Locks_Declared_Guarantee` for why the one provider that
/// exists today does not claim it either.
///
/// [`IncrementalGranularity::Project`]: resolving one type's generic argument can depend
/// on any item reachable from it through the crate's own module tree (a type alias
/// declared anywhere in the crate, an item re-exported from a dependency), so the unit
/// that must recompute together is the whole crate being analyzed, the same reasoning
/// `nomos_cap_rust_copy_clones::Ceiling` already gives for the identical structural reason.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::Project,
    );
}

#[must_use]
pub fn Capability() -> CapabilityId
{
    return CapabilityId::New(CAPABILITY);
}

#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

/// The contract, to be declared once by whichever composition root builds a registry.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "Every std::sync::Mutex<T> or std::sync::RwLock<T> in an analyzed crate \
                  whose type argument T a real compiler frontend resolved -- past \
                  whatever type alias, re-export, or generic substitution stood in the \
                  way -- to another std::sync::Mutex<U> or std::sync::RwLock<U>: a lock \
                  guarding a value that is already, itself, behind a lock. Found by \
                  asking rust-analyzer's own semantic-analysis engine to resolve every \
                  type annotation and check the resolved type's own generic arguments \
                  -- never inferred from a name that merely looks like a lock type."
            .to_owned(),
        ceiling: Ceiling(),
    };
}
