//! Zone: Provider -- every value a Go file assigns to the blank identifier, typed by `go/types`, as
//! `nomos.cap.go.discarded_values` facts.
//!
//! # Why this exists
//!
//! `a-discarded-error-is-explained` judges an error thrown away with `_`, and reading Go on its face
//! it could not tell an error from anything else: it flagged every `_ = f()` line -- a discarded
//! `fmt.Sprintf` string among them -- and saw neither `n, _ := strconv.Atoi(s)` nor
//! `_, _ = w.Write(b)` nor `var _ = os.Chdir(d)`, the forms most Go code discards an error in.
//! Whether a discarded value is an error is a fact about its type, and the call may be declared in
//! another package, so only a type checker answers it. That is the verdict `OD-ROADMAP-004` names as
//! the trigger for a new family: one the existing reading does not reach. Measured before this was
//! written, over one file holding each form: the text reading reported one of six discarded errors
//! correctly, missed three, and reported a string as an error.
//!
//! # Why a standard-library helper run through `go list`
//!
//! Two mechanisms were weighed. `golang.org/x/tools/go/packages` is the library Go's own tools
//! load type-checked packages with; a helper built on it is a module that must be downloaded before
//! it first runs, so a host with the toolchain but no network -- or a module proxy that refuses --
//! could not answer, and the answer would turn on a third-party version this workspace does not pin.
//! `gopls` answers the same questions through a running language server, which this provider would
//! have to start, speak LSP to, and stop, for every run; nothing else here keeps a server alive, and
//! a check is a batch.
//!
//! What was built is the first candidate without its download. `go/packages` is itself a reader of
//! `go list -json`: the metadata, the file lists and the export data it hands `go/types` all come
//! from `go list -export`, which ships with every toolchain. The helper, `helper/main.go`, asks
//! `go list` for exactly that and gives it to `go/types` directly, so it needs nothing beyond the
//! standard library of the toolchain that runs it. It is compiled into this crate, written under the
//! host's temporary directory, and run with `go run` -- see `crate::helper` for why `go run` and not
//! a binary built once.
//!
//! # A module at a time, answered a file at a time
//!
//! Modules are found from the walked sources exactly as `nomos-lang-go-lint` finds them for `go vet`
//! -- the nearest `go.mod` -- and the helper runs once per module. Its tests are part of the answer:
//! each package is checked as its test variant, whose files include the `_test.go` ones, and each
//! external test package on its own. Facts are per file, because the rule judges a file at a time.
//!
//! # What a caller can be told
//!
//! [`Materialize_Files`] answers a [`TypesAnswer`]: a fact for every file the helper checked, and
//! everything [`Untyped`]. A module the helper could not answer for at all is [`Untyped::Module`]
//! with its [`TypesFailure`]. Within a module that answered, a package that did not type-check is
//! [`Untyped::Package`] with the compiler's words, and a file no compiled package holds -- excluded
//! by a build constraint on this host, under `testdata`, a cgo source -- is [`Untyped::Unchecked`].
//! None of those is ever an empty fact, since a file nobody checked is not a file that discards
//! nothing.
//!
//! # What the host needs
//!
//! A Go toolchain, found as `nomos-lang-go-lint` finds it: `$GOROOT/bin/go` when the environment
//! names a root, `go` on the path otherwise. And a temporary directory to write the helper in.

#![forbid(unsafe_code)]

mod fact_context;
mod file_fact;
mod guarantee;
mod helper;
mod modules;
mod production;
mod typecheck;
mod types_answer;
mod types_fact_production;
mod types_failure;
mod types_ports;
mod untyped;

pub use fact_context::FactContext;
pub use file_fact::FileFact;
pub use guarantee::{Declared_Guarantee, PROVIDER, Provider_Offer};
pub use production::Materialize_Files;
pub use types_answer::TypesAnswer;
pub use types_fact_production::TypesFactProduction;
pub use types_failure::TypesFailure;
pub use types_ports::TypesPorts;
pub use untyped::Untyped;
