pub(crate) mod claimed_record;
mod disposition;
pub(crate) mod projection;
pub(crate) mod write;

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
/// `Test_Kind_Of_Should_Invert_What_The_Kind_Encodes_As_Text` is what says the pair is one
/// mapping. `None` for a label this build does not know, so a store written by a newer one
/// reads as unknown rather than as prose.
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

    fn All_Block_Kinds() -> [BlockKind; 3]
    {
        return [BlockKind::Heading, BlockKind::Prose, BlockKind::Code];
    }

    #[test]
    fn Test_Kind_Label_Should_Name_Every_Known_Block_Kind()
    {
        assert_eq!(Kind_Label(BlockKind::Heading), "heading");
        assert_eq!(Kind_Label(BlockKind::Prose), "prose");
        assert_eq!(Kind_Label(BlockKind::Code), "code");
    }

    #[test]
    fn Test_Kind_Of_Should_Invert_What_The_Kind_Encodes_As_Text()
    {
        for kind in All_Block_Kinds()
        {
            assert_eq!(Kind_Of(Kind_Label(kind)), Some(kind));
        }
        assert_eq!(Kind_Of("paragraph"), None);
    }
}
