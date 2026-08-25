//! Band 25 — a second provider of `nomos.cap.dependency.edges` (`nomos-cap-dependency`),
//! for a Go workspace rather than a Cargo one.
//!
//! # Why this provider reads text rather than running a subprocess
//!
//! `nomos-lang-rust-cargo` runs `cargo metadata` because a Cargo manifest's
//! `dependencies = {...}` entry is genuinely ambiguous on its face: the same string can
//! name a path dependency or an unrelated crate of the same name on a registry, and only
//! Cargo's own resolution tells the two apart. A Go module's `require` line carries no
//! equivalent ambiguity — a module path (`github.com/foo/bar`) is already the globally
//! unique identity Go's own tooling would resolve it to; there is no second candidate a
//! text read could confuse it with. Matching a `require` line's module path against
//! another workspace member's own declared `module` line is therefore a complete answer
//! to "does this member depend on that one", not an approximation waiting on a resolver —
//! the same standard `nomos_cap_dependency::Ceiling`'s own doc holds Cargo's manifest
//! text to and finds it short of, this provider's read meets for the question this
//! capability actually asks (first-party edges only, the same restriction
//! `nomos-lang-rust-cargo`'s own `--no-deps` invocation imposes).
//!
//! This is also a deliberate environmental choice, not only an architectural one: this
//! workspace already requires a working Cargo toolchain to build and test itself, so
//! `nomos-lang-rust-cargo` runs `cargo metadata` through a real, unconditionally
//! satisfiable dependency. It does not already require a working Go toolchain anywhere,
//! and a provider that could only be tested against a real `go` binary would introduce
//! exactly that dependency for the first time, silently, into every session's and every
//! CI run's environment. Reading `go.work`/`go.mod` text needs nothing this workspace
//! does not already have.
//!
//! # What is deliberately not read
//!
//! `replace` directives, in either a `go.work` or a `go.mod`, are not resolved. A
//! `replace` can point a module path anywhere, including outside the workspace entirely,
//! and honestly following it needs the same real resolution this crate's whole argument
//! above says a Go module path does not usually need — reading it now would be a second,
//! narrower version of the ambiguity this crate exists to avoid taking on. No workspace
//! this provider has been measured against uses one; the day one does, this is the first
//! place that answer is wrong.
//!
//! A module's `require` version is read only far enough to confirm the line has one; it
//! is not carried into the payload. `nomos_cap_dependency::DependencyPayload` states an
//! edge's target, kind and optionality — not a version — the same shape
//! `nomos-lang-rust-cargo`'s own payload carries for a Cargo dependency.
//!
//! Every edge this provider reports is [`nomos_cap_dependency::DependencyKind::Normal`]
//! and never optional. Go's module system has no equivalent of Cargo's dev/build
//! dependency tables or optional, feature-gated dependencies: every `require` line
//! stands for the same kind of edge, whether or not the code that uses it is a test.
//!
//! # Composed into `nomos-check-orchestration`
//!
//! See `nomos-check-orchestration::composition::Declare_Dependency_Capability`'s own doc
//! for whether this provider's offer is registered, and why.

#![forbid(unsafe_code)]

mod determinism;
mod discovery;
mod guarantee;
mod provider;

pub use determinism::DependencyFactProduction;
pub use discovery::{DiscoveredModule, Discover_Workspace, ModuleError};
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, ModuleFact};
