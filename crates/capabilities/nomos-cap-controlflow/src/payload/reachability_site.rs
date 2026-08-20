//! One flagged `Err(binding) => <body>` arm, and how its body was classified.

use super::ArmShape;

/// One `Err(binding) => <body>` arm this file's provider found and could classify as one
/// of [`ArmShape`]'s four obvious-defect shapes.
///
/// Absence from a payload is not a claim of cleanliness — an arm whose body is anything
/// else is simply not recorded, the same "no finding is not confirmed clean" convention
/// every rule in this workspace already keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachabilitySite
{
    /// The nearest enclosing function's own name, unqualified. Tier-1's canonical subject
    /// is a control-flow edge inside one function body in one file; a fully qualified path
    /// through enclosing `impl`/`mod` blocks is real information a sound provider should
    /// carry, and is not attempted here.
    pub function: String,
    /// The identifier the `Err(...)` pattern binds. Not asserted to be `applicability` by
    /// this type — the provider that writes it is the one narrowing to that name.
    pub binding: String,
    pub shape: ArmShape,
}
