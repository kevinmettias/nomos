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

    /// `Restored::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Restored` without a matching arm
    /// added here fails this file to *compile*, not merely to pass — the property D-134
    /// asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_Restored_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal(restored: Restored) -> usize
        {
            return match restored
            {
                Restored::RoadmapMilestone => 0,
                Restored::Scenario => 1,
                Restored::Service => 2,
                Restored::AppendixD => 3,
                Restored::AppendixH => 4,
                Restored::HeadlessInventory => 5,
                Restored::IdeProfile => 6,
                Restored::GlossaryTerm => 7,
                Restored::CanonicalDomainModel => 8,
            };
        }

        for (index, restored) in Restored::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal(*restored),
                index,
                "{} is not matched at the position Restored::All() puts it, so the \
                 exhaustive match and the universe have drifted apart",
                restored.Label()
            );
        }
    }
}
