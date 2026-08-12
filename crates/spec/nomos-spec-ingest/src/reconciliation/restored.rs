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
