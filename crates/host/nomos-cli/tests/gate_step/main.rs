//! The gate runs the rule layer, and nothing here can excuse it from failing.
//!
//! `OD-GATE-004` appended a `Rules` step to `.github/workflows/gate.yml` running
//! `cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .`. Everything that decides
//! whether that step *judges anything* lives in three places, and only one of them is code:
//! the step's presence, the tree it is pointed at, and whether it is allowed to fail.
//! `check_command.rs` beside this file asserts what the command does; this file asserts that
//! CI still runs it, over the whole workspace, with its exit code intact.
//!
//! # Why these are structural assertions and not a workflow run
//!
//! There is no runner here, so nothing in this repository can observe GitHub Actions failing
//! a step on a non-zero exit — and a test of that would be a test of a fixture rather than of
//! the platform's contract. What is checkable is everything on this side of it: that the step
//! exists and is derivable, that its argv still names `check` and still judges `.`, that no
//! step in the file carries an excuse, and that the workflow was read at all. The exit-code
//! policy itself is one assertion in `check.rs`,
//! `Test_Only_Ok_Should_Carry_The_Passing_Exit_Code`.
//!
//! The step is derived rather than grepped for, following
//! `crates/substrate/nomos-ledger/tests/gate_covers_finish.rs`. Deriving it means a step
//! rewritten as a shell script fails these assertions instead of satisfying them by
//! containing the right words.


mod workflow;
mod excuses;
mod lint_step;
mod pinning;
mod rules_step;
mod vacuity;
