//! Band 25 — the one provider of `nomos.cap.lint.diagnostics` (`nomos-cap-lint`).
//!
//! # Why this provider is a crate of its own, unlike a second offer inside another
//!
//! `nomos-lang-rust` and `nomos-lang-rust-scan` are both pure functions over bytes a
//! caller already read. This provider cannot be that shape — a lint tool's own diagnostic
//! is `cargo clippy`'s own analysis over fully type-checked code, not a fact this
//! workspace's own text already holds, so answering it means running `cargo clippy` and
//! reading its own JSON stream. That is the same real difference in what a provider *is*
//! that already put `nomos-lang-rust-cargo` in its own crate, and it is why this one is
//! too, at the same band, naming neither of its two `nomos.cap.dependency.edges`-reading
//! and `nomos.cap.syntax.items`-reading siblings.
//!
//! # What `OD-RULES-010` this satisfies
//!
//! `OD-RULES-010` decided the first real `ToolProvider`'s output is a fact a native rule
//! judges. [`reading::Discover_Workspace`] is the reader; [`provider::Materialize_Workspace`]
//! turns its answer into facts. Composing this crate into a real check run, and the rule
//! that reads what it materializes, are `nomos-check-orchestration` and `nomos-rules`'
//! own territory, not this crate's.
//!
//! The process itself runs through a caller-supplied [`nomos_platform::ProcessLauncher`],
//! not `std::process::Command` directly, the same port `nomos-lang-rust-cargo` already
//! runs its own subprocess through — so this crate depends on `nomos-platform` and not on
//! any concrete implementation of it; the composition root chooses that.

#![forbid(unsafe_code)]

#[path = "lint_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "fact_context.rs"]
mod provider;
#[path = "clippy_error.rs"]
mod reading;

pub use determinism::LintFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{DiagnosticsFact, FactContext, Materialize_Workspace};
pub use reading::{ClippyError, DiscoveredDiagnostics, Discover_Workspace};
