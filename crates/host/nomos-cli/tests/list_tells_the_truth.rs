//! `work list` as an agent meets it: the real binary, a real ledger, the real column.
//!
//! P9-DEPENDS-ON is about one word. `ready` meant "nobody holds this" and was read as "I
//! can take this", and the two came apart whenever a dependency was unfinished or somebody
//! held overlapping ground. Measured on 2026-08-09: the column said `ready` for eight items
//! that a single held claim refused, three separate times.
//!
//! These tests run the program rather than calling a function, because the defect was never
//! in the exclusion logic — `claim` always refused correctly. It was in what the listing
//! *said* about what claiming would do, so the assertion has to be made on the output a
//! person and an agent actually read.
//!
//! # Why this file declares modules through `#[path]`
//!
//! The scratch board this suite runs against is the one `stale_writer_is_refused.rs` and
//! `takeover_is_recorded.rs` run against, and a module shared between three test binaries
//! has to sit where all three can name it without leaving their own directory. That makes
//! `tests/` the shared root, and a bare `mod x;` in a root there resolves to `tests/x.rs`,
//! which cargo would build as a further test target. `#[path]` is what keeps this suite's
//! own parts beside it while letting it reach the shared board.


#[path = "list_tells_the_truth/audit.rs"]
mod audit;
#[path = "list_tells_the_truth/authored.rs"]
mod authored;
#[path = "scratch_ledger/board.rs"]
mod board;
#[path = "list_tells_the_truth/dependencies.rs"]
mod dependencies;
#[path = "list_tells_the_truth/territory.rs"]
mod territory;
