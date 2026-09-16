//! The process's own environment, as a dependency rather than as an ambient fact.

use crate::EnvironmentError;
use nomos_contracts::Strategy;
use std::ffi::OsString;
use std::path::PathBuf;

/// What the process was started with and where it is standing.
///
/// A dependency rather than a call to [`std::env`], for a reason this workspace measured
/// rather than anticipated. Three providers — `nomos-lang-rust-cargo`,
/// `nomos-lang-rust-clippy` and `nomos-lang-rust-deny` — each build a [`crate::Command`]
/// for an injected [`crate::ProcessLauncher`] and then decide that command's *name* with a
/// bare `std::env::var("CARGO")`, byte-identical in all three. `clippy_error.rs` does both
/// within twenty lines: one function takes a launcher parameter, the next reads
/// `std::env::current_dir()` directly.
///
/// The consequence is that injecting a launcher does not make those providers testable.
/// A fake launcher can say what a command *produced*, but not which `cargo` was going to
/// be run, nor what a relative root was resolved against — both of those were decided
/// past every seam, by the machine the test was trying not to depend on.
///
/// # Scope
///
/// Two operations, because two have callers. `std::env::args` is deliberately absent:
/// every one of its production call sites is a `main.rs`, which is the composition root
/// choosing reality on purpose and not a leak past a port. `std::env::temp_dir` has one
/// production caller, in `nomos-agent-contracts`, and belongs to whichever item migrates
/// it — this crate's own scope rule is that a trait nothing calls is a claim about the
/// future.
///
/// Neither operation writes. `std::env::set_var` is unsound under threads and has no
/// caller in this workspace; a port method for it would be an invitation rather than a
/// capability.
///
/// # What an implementor promises
///
/// The supertrait is [`nomos_contracts::Strategy`], so every implementor states its
/// determinism triple. The real one promises nothing and says so; a double built from a
/// fixed map reproduces and says that. A caller reading `S::STRENGTH` can tell them apart
/// without knowing either type.
pub trait Environment: Strategy
{
    /// One environment variable's value, or `None` when it is unset.
    ///
    /// Returns [`OsString`] rather than [`String`] because the two existing readers of
    /// this data disagree about which they want, and both are right:
    /// `nomos-lang-rust-cargo` wants a program name and `nomos-api` wants a corpus path.
    /// A path is not guaranteed to be UTF-8 on either platform this workspace runs on, so
    /// narrowing here would decide that question for a caller that had already chosen
    /// `var_os` deliberately.
    ///
    /// A value that is not valid UTF-8 is returned, not refused. A caller wanting text
    /// asks for it with [`OsString::into_string`], which reproduces exactly what
    /// [`std::env::var`] already did with one: report it as unusable rather than as
    /// present.
    fn Variable(&self, name: &str) -> Option<OsString>;

    /// The directory the process is currently standing in.
    ///
    /// # Errors
    ///
    /// Returns [`EnvironmentError::WorkingDirectoryUnreadable`] when it cannot be read —
    /// it has been removed underneath the process, or permission to resolve it was
    /// withdrawn.
    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>;
}
