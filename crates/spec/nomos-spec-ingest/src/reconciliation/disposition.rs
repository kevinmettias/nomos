//! What an overlay decided about one block.

use nomos_spec_model::ContentHash;
/// What became of one v14 identifier in v15.0.
///
/// Three arms, and `Absent` is not a variant of `Reworded`. A family that vanished and a
/// family whose wording drifted are different failures, and collapsing them is how a
/// disappearance reads as an edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Disposition
{
    Preserved,
    Reworded
    {
        v14: ContentHash,
        v15: ContentHash,
    },
    Absent,
}
