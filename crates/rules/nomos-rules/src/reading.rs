//! What became of reading one file for declared universes.

use crate::declared_universe::DeclaredUniverse;
/// What reading one file's syntax fact produced.
///
/// Two variants, and the second is not "the file was empty". A provider that could not
/// observe documentation has told this module nothing about mirrors, and reporting that as
/// a file whose universes all declare none would turn every phantom under that provider
/// into an admitted gap. `Applicability::Unparseable` exists in `nomos-contracts` for the
/// neighbouring distinction and the rule maps this onto it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    /// The provider observed what discovery needs, and these are the universes. Possibly
    /// none, which is a real answer about the file.
    Observed(Vec<DeclaredUniverse>),
    /// The provider that answered cannot see what discovery reads. Nothing is claimed
    /// about what the file contains.
    Unobserved
    {
        /// Which field, so a finding can say what would have to change.
        because: String,
    },
}
