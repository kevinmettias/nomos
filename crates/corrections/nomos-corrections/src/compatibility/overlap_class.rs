//! Which two accesses collide on one subject.

/// How two plans collide on one subject.
///
/// `COR-CONFLICT-002`: "`TextualConflict` identifies overlapping or incompatible concrete
/// edits." Both classes here are that conflict, read off declared sets rather than off
/// content: two writes to one subject have no correct joint reading, and a write beside a
/// read makes the read's answer depend on which ran first. Read beside read is not a
/// class, because two readers disturb nothing.
///
/// Ordered with the stronger class first, so that a subject reported at its strongest
/// class sorts ahead of the same spelling at a weaker one -- which never happens, since
/// each subject is reported once, but the order says which is stronger.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OverlapClass
{
    /// Both plans write the subject.
    WriteWrite,
    /// One plan writes the subject and the other only reads it.
    ReadWrite,
}

impl OverlapClass
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::WriteWrite => "write-write",
            Self::ReadWrite => "read-write",
        };
    }
}

impl core::fmt::Display for OverlapClass
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL_CLASSES: [OverlapClass; 2] = [OverlapClass::WriteWrite, OverlapClass::ReadWrite];

    #[test]
    fn Test_Label_Should_Be_Distinct_Per_Variant()
    {
        let mut labels: Vec<&str> = ALL_CLASSES.iter().map(|class| return class.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two classes share a wire spelling");
    }

    #[test]
    fn Test_Display_Should_Render_The_Label()
    {
        assert_eq!(OverlapClass::WriteWrite.to_string(), "write-write");
        assert_eq!(OverlapClass::ReadWrite.to_string(), "read-write");
    }

    #[test]
    fn Test_Write_Write_Should_Order_Before_Read_Write()
    {
        assert!(OverlapClass::WriteWrite < OverlapClass::ReadWrite);
    }
}
