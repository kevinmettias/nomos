//! Every declared universe is classified, and the classification cannot go stale.
//!
//! `OD-COMPLETENESS-001`: a completeness guard is only as complete as the universe it
//! quantifies over, and when that universe is *declared* rather than derived, comparing the
//! declaration against reality is the other direction. Three guards in this workspace were
//! half-checks for exactly that reason, and all three were found by accident rather than by
//! looking.
//!
//! Discovery is mechanical and classification is not. `Declared_Universes` finds the lists;
//! whether a given list has a check comparing it against the reality it claims to enumerate
//! is a question about meaning, and this crate deliberately has no types to answer it with.
//! So the classification is declared in [`table::UNIVERSES`] and checked against what is
//! derived — the shape `OD-GATE-001` already uses. Neither side is trusted alone, and in
//! particular the table cannot go stale in the direction that flatters: a new universe that
//! nobody classified fails [`derivation::Test_The_Declared_Table_Should_Match_What_Is_Derived`],
//! and a new *unmirrored* universe additionally fails
//! [`hole_size::Test_The_Number_Of_Unmirrored_Universes_Should_Be_Declared`], so it cannot be
//! added quietly.
//!
//! # What this does not do
//!
//! It does not close the twelve holes it counts. A gate that can never be green is a gate
//! everybody learns to ignore, and twelve mirrors is not one item's work. What it does is
//! make the number a figure somebody chose rather than a silence nobody measured, which is
//! the same remedy `OD-GATE-001` applies to the corpus gates.


mod claims;
mod derivation;
mod hole_size;
mod mirrors;
mod table;
