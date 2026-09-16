//! The families a restoration pass can recover.

/// A family v15.0 dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restored
{
    RoadmapMilestone,
    Scenario,
    Service,
    AppendixD,
    AppendixH,
    HeadlessInventory,
    IdeProfile,
    GlossaryTerm,
    CanonicalDomainModel,
}

impl Restored
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "roadmap_milestone",
            Self::Scenario => "scenario",
            Self::Service => "service",
            Self::AppendixD => "appendix_d",
            Self::AppendixH => "appendix_h",
            Self::HeadlessInventory => "headless_inventory",
            Self::IdeProfile => "ide_profile",
            Self::GlossaryTerm => "glossary_term",
            Self::CanonicalDomainModel => "canonical_domain_model",
        };
    }

    /// The node kind a restored member takes.
    #[must_use]
    pub const fn Node_Kind(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "release",
            Self::Scenario => "scenario",
            Self::Service => "service",
            Self::AppendixD => "schema",
            Self::AppendixH => "section",
            Self::HeadlessInventory => "inventory",
            Self::IdeProfile => "projection_profile",
            Self::GlossaryTerm => "glossary_term",
            Self::CanonicalDomainModel => "concept",
        };
    }

    #[must_use]
    pub const fn Prefix(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "RMAP",
            Self::Scenario => "SCEN",
            Self::Service => "SVC",
            Self::AppendixD => "APX-D",
            Self::AppendixH => "APX-H",
            Self::HeadlessInventory => "HLS",
            Self::IdeProfile => "IDE",
            Self::GlossaryTerm => "GLS",
            Self::CanonicalDomainModel => "CDM",
        };
    }

    /// The volume a family lives in, by filename stem.
    ///
    /// Scoped rather than searched tree-wide so a family's membership is the same set the
    /// register measured. A recognition that matched across every volume would count
    /// whatever else happened to be shaped like it.
    #[must_use]
    pub const fn Volume(self) -> &'static str
    {
        return match self
        {
            Self::RoadmapMilestone => "08-roadmap",
            Self::Scenario | Self::AppendixD | Self::GlossaryTerm => "09-reference",
            Self::Service | Self::CanonicalDomainModel => "02-core",
            Self::AppendixH => "06-agents",
            Self::HeadlessInventory | Self::IdeProfile => "07-clients",
        };
    }

    /// Every family a restoration pass can recover.
    ///
    /// Mirrored by `Test_Every_Restored_Should_Be_Matched_Exhaustively`, an exhaustive
    /// match over every variant with no wildcard arm, in this file. It fails to compile,
    /// not merely to pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::RoadmapMilestone,
            Self::Scenario,
            Self::Service,
            Self::AppendixD,
            Self::AppendixH,
            Self::HeadlessInventory,
            Self::IdeProfile,
            Self::GlossaryTerm,
            Self::CanonicalDomainModel,
        ];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::collections::BTreeSet;

    /// Where each variant sits in `Restored::All()`, counted from zero — its ordinal.
    ///
    /// The match below stays a match rather than becoming an index lookup: the property
    /// its doc comment claims is that a variant added to `Restored` and not to this arm
    /// list fails the file to *compile*, and a lookup would only fail to pass. So each arm
    /// names the position it answers instead of spelling a bare ordinal.
    const SERVICE_POSITION: usize = 2;
    const APPENDIX_D_POSITION: usize = 3;
    const APPENDIX_H_POSITION: usize = 4;
    const HEADLESS_INVENTORY_POSITION: usize = 5;
    const IDE_PROFILE_POSITION: usize = 6;
    const GLOSSARY_TERM_POSITION: usize = 7;
    const CANONICAL_DOMAIN_MODEL_POSITION: usize = 8;

    /// The size of the universe the match above mirrors, so the two are read together.
    const RESTORED_FAMILIES: usize = 9;

    /// `Restored::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Restored` without a matching arm
    /// added here fails this file to *compile*, not merely to pass — the property D-134
    /// asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_Restored_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal_Of(restored: Restored) -> usize
        {
            return match restored
            {
                Restored::RoadmapMilestone => 0,
                Restored::Scenario => 1,
                Restored::Service => SERVICE_POSITION,
                Restored::AppendixD => APPENDIX_D_POSITION,
                Restored::AppendixH => APPENDIX_H_POSITION,
                Restored::HeadlessInventory => HEADLESS_INVENTORY_POSITION,
                Restored::IdeProfile => IDE_PROFILE_POSITION,
                Restored::GlossaryTerm => GLOSSARY_TERM_POSITION,
                Restored::CanonicalDomainModel => CANONICAL_DOMAIN_MODEL_POSITION,
            };
        }

        for (index, restored) in Restored::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal_Of(*restored),
                index,
                "{} is not matched at the position Restored::All() puts it, so the \
                 exhaustive match and the universe have drifted apart",
                restored.Label()
            );
        }
    }

    #[test]
    fn Test_Label_Should_Slug_Each_Restored_Family()
    {
        assert_eq!(Restored::RoadmapMilestone.Label(), "roadmap_milestone");
        assert_eq!(Restored::Scenario.Label(), "scenario");
        assert_eq!(Restored::GlossaryTerm.Label(), "glossary_term");
    }

    #[test]
    fn Test_Node_Kind_Should_Give_The_Restored_Graph_Node_Type()
    {
        assert_eq!(Restored::RoadmapMilestone.Node_Kind(), "release");
        assert_eq!(Restored::AppendixD.Node_Kind(), "schema");
        assert_eq!(Restored::CanonicalDomainModel.Node_Kind(), "concept");
    }

    #[test]
    fn Test_Prefix_Should_Give_The_Restored_Identifier_Prefix()
    {
        assert_eq!(Restored::Service.Prefix(), "SVC");
        assert_eq!(Restored::AppendixH.Prefix(), "APX-H");
    }

    #[test]
    fn Test_Volume_Should_Give_The_Filename_Stem_A_Family_Lives_Under()
    {
        assert_eq!(Restored::RoadmapMilestone.Volume(), "08-roadmap");
        assert_eq!(Restored::Scenario.Volume(), "09-reference");
        assert_eq!(Restored::AppendixD.Volume(), "09-reference");
        assert_eq!(Restored::Service.Volume(), "02-core");
        assert_eq!(Restored::AppendixH.Volume(), "06-agents");
        assert_eq!(Restored::HeadlessInventory.Volume(), "07-clients");
    }

    #[test]
    fn Test_All_Should_List_Nine_Restored_Families_With_No_Duplicate()
    {
        let all = Restored::All();

        assert_eq!(all.len(), RESTORED_FAMILIES);

        let unique: BTreeSet<_> = all.iter().collect();
        assert_eq!(unique.len(), all.len());
    }
}
