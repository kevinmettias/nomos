//! `nomos.cap.syntax.sites` for C#: the family `OD-CAPABILITY-019` placed beside
//! `nomos.cap.syntax.items`, offered so that C# can state what it declines.
//!
//! A decline is the second of the three answers `OD-CAPABILITY-019`'s sixth decision separates: a
//! provider declines a kind only with a stated reason that the construct does not exist in its
//! language, and a rule reads that as `NotApplicable` -- a positive statement about an absent
//! judgment, which is different from the gap a kind nobody answered is. The answer is carried in a
//! payload, so C# offers the family and pays its per-provider cost to say it.
//!
//! It declines the labeled jump, for the reason code-standards' C# kernel states in
//! `csharp_declines.go` at `f0d820729`. It offers no kind yet. A decline is a fact about the
//! language rather than about the file, so it is stated for every C# file from its text alone,
//! without a parse: a file this provider could not parse still has no loop a label could name.

mod guarantee;
mod provider;

pub use guarantee::{Declared_Guarantee, Provider_Offer};
pub use provider::Materialize_Sites_Fact;

/// Why C# declines the labeled-jump kind -- the substance of the corpus kernel's reason.
pub const LABELED_JUMP_DECLINE: &str = "C# cannot name a loop: its break and continue take no loop argument -- each leaves or restarts \
                                        the innermost enclosing loop and nothing else -- and the label C# does have belongs to goto, which \
                                        names a point to jump to rather than a loop to leave";
