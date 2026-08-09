//! Facts about the workspace, asserted rather than intended.
//!
//! The architecture describes a strict dependency order and a contracts crate that
//! names almost nothing. Both are the kind of property that holds on the day it is
//! written and erodes one convenient import at a time, so both are tests.
//!
//! This crate holds the shared machinery; the assertions live in `tests/`.

#![forbid(unsafe_code)]

mod gates;
mod metadata;

pub use gates::{Corpus_Gates, CorpusGate, CORPUS_VARIABLES};
pub use metadata::{Package, Workspace};
