//! Whether the store renders a just-committed record back as the bytes that were staged.

/// Whether the store renders a just-committed record back as the bytes that were staged.
#[derive(Debug)]
pub enum Reproduction
{
    /// The store's own rendering matches what was staged, byte for byte.
    Matched
    {
        hash: String,
    },
    /// The store renders something else. The commit already happened; this says the round
    /// trip did not close.
    Mismatched
    {
        hash: String,
    },
}
