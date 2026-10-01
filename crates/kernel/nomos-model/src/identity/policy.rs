use serde::{Deserialize, Serialize};

/// How ambiguous declarations are resolved into distinct identities.
///
/// These are the cases where two declarations can legitimately share a qualified name,
/// and each needs a stated answer rather than whatever the first implementation
/// happened to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy
{
    /// Whether overloads sharing a name are distinguished by signature.
    pub distinguish_overloads: bool,
    /// Whether generated declarations are distinguished from hand-written ones.
    pub distinguish_generated: bool,
    /// Whether declarations under different conditional-compilation configurations are
    /// distinct.
    pub distinguish_conditional_compilation: bool,
}

impl Policy
{
    /// The policy Nomos applies unless a language package states otherwise.
    ///
    /// All three on. Every one of them off produces a collision that presents as a
    /// finding attached to the wrong declaration, which is worse than a missing
    /// finding because it sends someone to read code that is fine.
    pub const STRICT: Self = Self {
        distinguish_overloads: true,
        distinguish_generated: true,
        distinguish_conditional_compilation: true,
    };
}
