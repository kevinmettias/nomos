//! Facts about the workspace, asserted rather than intended.
//!
//! The architecture describes a strict dependency order and a contracts crate that
//! names almost nothing. Both are the kind of property that holds on the day it is
//! written and erodes one convenient import at a time, so both are tests.
//!
//! This crate holds the shared machinery; the assertions live in `tests/`.

#![forbid(unsafe_code)]

mod crossing;
mod domain_row;
mod reading;
mod fact_domain;
mod gates;
mod harnessed;
mod package;
mod surface;
mod universes;
mod workspace;

pub use reading::declaration::Declaration;
pub use crossing::{Declared_Revision, Declared_Revisions, Pinning_Manifests, Revisions_In, Repository_Root, XVPE_GIT_URL, XVPE_PREFIX};
pub use domain_row::{Domain_Table, DomainRow};
pub use fact_domain::{Fact_Domains, FactDomain};
pub use gates::{Corpus_Gates, CorpusGate, CORPUS_VARIABLES};
pub use harnessed::Harnessed_Strategies;
pub use package::Package;
pub use workspace::Workspace;
pub use surface::{Crate_Identifier, Public_Surface, Surface};
pub use universes::{Declared_Universes, DeclaredUniverse, UniverseKind};
