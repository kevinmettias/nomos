//! P10-RECORD-LOCK acceptance: writing a record must not serialize the board.
//!
//! Every assertion here runs against the repository's own `work/ledger.json` rather than
//! against a fixture. That is deliberate and it is what the item asked for: the defect was
//! never in the exclusion code, which has always compared paths correctly. It was in what
//! the items *say*, so a fixture proving that two invented territories are disjoint would
//! have passed on the day the whole board was blocked.
//!
//! The claims are exercised against a copy in a temporary directory. A test that claimed on
//! the real ledger would take territory from whoever is working the repository while it
//! runs, and a suite with side effects on the thing it measures is not a suite.


mod common;
mod identifier_and_file;
mod record_directory;
mod record_exclusion;
mod serializers;
mod sharing_a_record;
mod snapshot_grain;
