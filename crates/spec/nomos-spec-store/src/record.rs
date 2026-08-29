pub(crate) mod claimed_record;
mod disposition;
pub(crate) mod record_projection;
pub(crate) mod record_write;

use nomos_spec_model::BlockKind;

pub use disposition::Disposition;

#[must_use]
pub const fn Kind_Label(kind: BlockKind) -> &'static str
{
    return match kind
    {
        BlockKind::Heading => "heading",
        BlockKind::Prose => "prose",
        BlockKind::Code => "code",
    };
}

/// The kind a stored label names.
///
/// [`Kind_Label`]'s inverse, and its counterpart rather than a second opinion: reading a
/// block back out of the store needs the label to mean what writing it meant, and
/// `Test_A_Block_Kind_Should_Survive_The_Label` is what says the pair is one mapping.
/// `None` for a label this build does not know, so a store written by a newer one reads as
/// unknown rather than as prose.
#[must_use]
pub(crate) fn Kind_Of(label: &str) -> Option<BlockKind>
{
    return [BlockKind::Heading, BlockKind::Prose, BlockKind::Code]
        .into_iter()
        .find(|kind| return Kind_Label(*kind) == label);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Block_Kind_Should_Survive_The_Label()
    {
        for kind in [BlockKind::Heading, BlockKind::Prose, BlockKind::Code]
        {
            assert_eq!(Kind_Of(Kind_Label(kind)), Some(kind));
        }
        assert_eq!(Kind_Of("paragraph"), None);
    }
}
