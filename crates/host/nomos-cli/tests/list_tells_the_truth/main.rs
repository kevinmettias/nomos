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


mod audit;
mod common;
mod dependencies;
mod territory;
