//! A statement that disagrees with its source.

/// A statement whose recorded hash does not match its recorded text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergence
{
    pub id: String,
    pub recorded: String,
    pub recomputed: String,
    pub text_is_canonical: bool,
}
