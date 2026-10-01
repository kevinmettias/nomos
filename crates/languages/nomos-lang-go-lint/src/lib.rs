//! Zone: Provider -- `go vet`'s diagnostics for each Go module, as `nomos.cap.lint.diagnostics`
//! facts.
//!
//! # Why this exists
//!
//! `nomos.cap.lint.diagnostics` had one provider, `nomos-lang-rust-clippy`, so a Go repository's lint
//! findings could not enter this workspace as facts at all. `OD-RULES-010` already decided the shape
//! -- a tool's output is a fact a native rule judges, never a finding the tool emits -- and the
//! clippy provider is its worked example. This is the same shape for Go: the rule that reads the
//! family relays a Go module's diagnostics exactly as it relays a Rust member's.
//!
//! # Why `go vet`
//!
//! Three linters were weighed. `staticcheck` and `golangci-lint` find more, and each is a separate
//! install -- neither was on the host this was measured on -- so choosing one would make a Go
//! repository's lint facts depend on a tool most Go hosts do not have. `go vet` ships inside every
//! Go toolchain: a host that can build the code can vet it. Its analyzers run over type-checked
//! packages, which is the ground for claiming `SemanticallyResolved`. And it has a machine-readable
//! form: `-json` prints the analysis framework's own JSON tree, one object per package, rather than
//! text written for a person. `go help vet` promises no stability for that form, so `crate::vet`
//! reads exactly the shape measured and refuses any other, rather than guessing at a changed one.
//! A second Go linter would be a second provider under its own name, not this one widened.
//!
//! # A module at a time, found from the walk
//!
//! The unit is the Go module: the nearest `go.mod` at or above each walked `.go` source, never
//! above the root. One `go vet ./...` per module, one fact per module, filed under the module
//! directory's subject. The modules are found from the filesystem, not by asking `go`, because
//! asking `go` launches it in repositories with no Go at all -- where a missing toolchain would
//! be reported as unavailable over code nobody asked it to read -- and because `go` answers "a
//! module" even where there is none (`crate::modules` has the measurement). So a repository with no
//! Go source launches nothing and reports nothing.
//!
//! # What a caller can be told
//!
//! [`Materialize_Modules`] answers a [`LintAnswer`]: the facts, and everything [`Unlinted`]. A
//! module whose `go vet` could not run, did not finish, or failed on code that does not type-check
//! is [`Unlinted::Module`] with its [`VetFailure`], never a fact with no diagnostic in it, since a
//! tool that failed has not found nothing. Go sources under no `go.mod` are [`Unlinted::Outside`]:
//! no module holds them, so `go vet` has no package to load them in.
//!
//! # What the host needs
//!
//! A Go toolchain. `go` is run from `$GOROOT/bin` when the environment names a `GOROOT`, and from
//! the path otherwise. That lets a host with more than one Go on its path say which one it means,
//! as this workspace's own host must: the `go` first on its shell path is a trimmed build that
//! cannot find its root.
//!
//! # What it does not answer
//!
//! Which types a Go expression has. `go vet` resolves them to run its analyzers and exposes none of
//! it. What a rule has asked of Go's types so far -- which discarded values are errors -- is its own
//! capability with its own provider, `nomos-lang-go-types`, which gives `go list`'s export data to
//! `go/types` directly rather than asking `go vet`.

#![forbid(unsafe_code)]

mod fact_context;
mod guarantee;
mod lint_answer;
mod lint_fact_production;
mod lint_ports;
mod module_fact;
mod modules;
mod production;
mod unlinted;
mod vet;
mod vet_failure;

pub use fact_context::FactContext;
pub use guarantee::{Declared_Guarantee, PROVIDER, Provider_Offer};
pub use lint_answer::LintAnswer;
pub use lint_fact_production::LintFactProduction;
pub use lint_ports::LintPorts;
pub use module_fact::ModuleFact;
pub use production::Materialize_Modules;
pub use unlinted::Unlinted;
pub use vet_failure::VetFailure;
