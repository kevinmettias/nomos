//! Every shipped profile, rendered from one store and checked from six directions.
//!
//! A projection is a rendering of the store and never a second authority, so what this suite
//! asserts is mostly about what must *not* reach the output: the order the store was written
//! in, the machine that built it, a node somebody deleted, a subject somebody else asked for.


mod catalogue;
mod common;
mod determinism;
mod freshness;
mod rebuildability;
mod renderers;
mod selection;
mod subjects;
