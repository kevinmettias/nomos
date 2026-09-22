//! One subject two plans cannot both touch in the same wave, and why.

use super::OverlapClass;
use crate::ReadWriteResolution;

/// One subject two plans both reach, at the tier it was declared at, with the class the
/// collision has.
///
/// `COR-CONFLICT-008`: "Every conflict record shall identify involved operations and
/// subjects, conflict subtype, evidence, confidence, affected obligations, whether
/// automatic resolution is permitted, candidate dispositions, and resulting validation
/// requirements." This carries the subject and the subtype. The involved operations are
/// the two plans the [`crate::Compatibility`] holding it was asked about, and the rest of
/// that list is not derivable from declared sets, so it is not invented here.
///
/// Ordered tier first, then spelling, then class -- the order a
/// [`crate::Compatibility::Conflicting`] lists its overlaps in.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Overlap
{
    resolution: ReadWriteResolution,
    subject: String,
    class: OverlapClass,
}

impl Overlap
{
    #[must_use]
    pub fn New(resolution: ReadWriteResolution, subject: impl Into<String>, class: OverlapClass) -> Self
    {
        return Self {
            resolution,
            subject: subject.into(),
            class,
        };
    }

    /// The tier the subject was declared at by both plans.
    #[must_use]
    pub const fn Resolution(&self) -> ReadWriteResolution
    {
        return self.resolution;
    }

    /// The subject's spelling, exactly as both plans declared it.
    #[must_use]
    pub fn Subject(&self) -> &str
    {
        return &self.subject;
    }

    #[must_use]
    pub const fn Class(&self) -> OverlapClass
    {
        return self.class;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_New_Should_Carry_Every_Field_It_Was_Given()
    {
        let overlap = Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::WriteWrite);

        assert_eq!(overlap.Resolution(), ReadWriteResolution::Artifact);
        assert_eq!(overlap.Subject(), "a.rs");
        assert_eq!(overlap.Class(), OverlapClass::WriteWrite);
    }

    #[test]
    fn Test_Resolution_Should_Report_The_Declared_Tier()
    {
        let overlap = Overlap::New(ReadWriteResolution::Symbol, "a::item", OverlapClass::ReadWrite);

        assert_eq!(overlap.Resolution(), ReadWriteResolution::Symbol);
    }

    #[test]
    fn Test_Subject_Should_Report_The_Declared_Spelling()
    {
        let overlap = Overlap::New(ReadWriteResolution::Symbol, "a::item", OverlapClass::ReadWrite);

        assert_eq!(overlap.Subject(), "a::item");
    }

    #[test]
    fn Test_Class_Should_Report_How_The_Plans_Collide()
    {
        let overlap = Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::ReadWrite);

        assert_eq!(overlap.Class(), OverlapClass::ReadWrite);
    }

    #[test]
    fn Test_Overlaps_Should_Order_By_Tier_Before_Spelling()
    {
        let artifact = Overlap::New(ReadWriteResolution::Artifact, "z.rs", OverlapClass::WriteWrite);
        let symbol = Overlap::New(ReadWriteResolution::Symbol, "a::item", OverlapClass::WriteWrite);

        assert!(artifact < symbol, "a weaker tier orders first whatever its spelling");
    }
}
