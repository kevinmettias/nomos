//! How a listing's filters admitted one item.

/// How a listing's filters admitted one item.
///
/// Two arms rather than a `bool`, because a row whose overlap could not be decided is printed
/// and has to say so. The claim check refuses on exactly that answer, so a listing that left
/// such an item out would tell its reader a path is free that a claim says is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission
{
    /// Every filter given admits the item, and each answer was decided.
    Admitted,
    /// Every filter given admits the item, and whether its territory overlaps the path
    /// `touching` names could not be decided: an unexpanded pattern, or a territory authored at
    /// another resolution.
    Undecided,
}
