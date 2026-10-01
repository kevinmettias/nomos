//! Where a configuration value came from -- the ten typed sources Nomos policy resolves
//! across, in the precedence order the corpus's own listings share.

use serde::{Deserialize, Serialize};

const DEFAULT_LABEL: &str = "Default";
const ORGANIZATION_LABEL: &str = "Organization";
const REPOSITORY_LABEL: &str = "Repository";
const WORKSPACE_LABEL: &str = "Workspace";
const USER_LABEL: &str = "User";
const WORKFLOW_LABEL: &str = "Workflow";
const GATE_LABEL: &str = "Gate";
const COMMAND_LINE_LABEL: &str = "CommandLine";
const ENVIRONMENT_LABEL: &str = "Environment";
const TEMPORARY_RUN_OVERRIDE_LABEL: &str = "TemporaryRunOverride";

/// `OD-POLICY-001`: the ten layers Nomos policy resolves across, lowest precedence first.
///
/// The ten of volume 02's `ConfigurationLayer` glossary entry and `US-CONFIG-001`'s
/// acceptance, which are `CONFIG-002`'s nine plus the default every field has before anyone
/// states it. The declaration order *is* the precedence order: `OD-POLICY-001` took it from
/// the three corpus listings that agree on it and confirmed it against `MODEL-ROUTE-005`'s
/// explicit ladder at every name the two have in common, so a derived [`Ord`] over this
/// declaration is a direct transcription of that order rather than an ordering invented here.
///
/// Named whole although five of the ten have no source on this host. That is the record's
/// own decision and not an oversight: "Organization: no source on this host" and
/// "Organization: declared nothing" are different facts, and a resolver that could not name
/// the first would have to report it as the second. Which layers are observable here is
/// `OD-POLICY-001`'s own table, not a property of this type.
///
/// Lives beside [`crate::KnowledgeSourceRole`] for the reason `OD-CONTRACTS-001` gives: it
/// is a closed, corpus-transcribed vocabulary about the authority of a source, and it crosses
/// two boundaries -- orchestration to host inside a gate result, and gate to agent when a
/// model resolver reports the layer a candidate came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfigurationLayer
{
    Default,
    Organization,
    Repository,
    Workspace,
    User,
    Workflow,
    Gate,
    CommandLine,
    Environment,
    TemporaryRunOverride,
}

impl ConfigurationLayer
{
    /// Every layer `OD-POLICY-001` decides, in that record's own order.
    ///
    /// Public and outside a test module because a resolver reports the layers it consulted
    /// -- including the ones that had no source -- and that report quantifies over this
    /// array. A layer missing from it would be a layer no report could ever say was absent,
    /// which is the honesty this whole vocabulary exists for.
    ///
    /// A variant missing from this array is not a compile error; a variant missing a label
    /// already is one, in [`Self::Label`].
    ///
    /// Mirrored by `Test_Every_Layer_Should_Be_Listed`, which compares this array against the
    /// variants named in the test module beside it, in both directions.
    pub const ALL: [Self; 10] = [
        Self::Default,
        Self::Organization,
        Self::Repository,
        Self::Workspace,
        Self::User,
        Self::Workflow,
        Self::Gate,
        Self::CommandLine,
        Self::Environment,
        Self::TemporaryRunOverride,
    ];

    /// The variant's stable `PascalCase` name.
    ///
    /// The spelling volume 02's glossary entry uses, so a peer that never compiles this crate
    /// names a layer the way the corpus does.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Default => DEFAULT_LABEL,
            Self::Organization => ORGANIZATION_LABEL,
            Self::Repository => REPOSITORY_LABEL,
            Self::Workspace => WORKSPACE_LABEL,
            Self::User => USER_LABEL,
            Self::Workflow => WORKFLOW_LABEL,
            Self::Gate => GATE_LABEL,
            Self::CommandLine => COMMAND_LINE_LABEL,
            Self::Environment => ENVIRONMENT_LABEL,
            Self::TemporaryRunOverride => TEMPORARY_RUN_OVERRIDE_LABEL,
        };
    }
}

impl core::fmt::Display for ConfigurationLayer
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::ConfigurationLayer as Subject;
    use alloc::vec::Vec;

    /// The array production reads, bound once so the two assertions below cannot drift on to
    /// different spellings of it.
    const ALL: [Subject; 10] = Subject::ALL;

    /// Every variant reaches [`ConfigurationLayer::ALL`], which is the claim that constant's
    /// own doc comment makes.
    ///
    /// The array is written by hand and sized by hand, so a variant left out of it is not a
    /// compile error and no report would ever name that layer -- neither as a contributor nor
    /// as an absence, which is the worse half.
    ///
    /// The mirror is between two independent spellings of one set: the array, which
    /// production reads, and the variants named below, which this test writes. An eleventh
    /// variant makes the `match` non-exhaustive -- it carries no wildcard arm -- so this stops
    /// compiling until somebody names it here, and naming it here while leaving `[Self; 10]`
    /// alone fails the assertion beneath.
    #[test]
    fn Test_Every_Layer_Should_Be_Listed()
    {
        for listed in ALL
        {
            match listed
            {
                Subject::Default
                | Subject::Organization
                | Subject::Repository
                | Subject::Workspace
                | Subject::User
                | Subject::Workflow
                | Subject::Gate
                | Subject::CommandLine
                | Subject::Environment
                | Subject::TemporaryRunOverride =>
                {}
            }
        }

        for expected in [
            Subject::Default,
            Subject::Organization,
            Subject::Repository,
            Subject::Workspace,
            Subject::User,
            Subject::Workflow,
            Subject::Gate,
            Subject::CommandLine,
            Subject::Environment,
            Subject::TemporaryRunOverride,
        ]
        {
            assert!(ALL.contains(&expected), "{expected:?} is a ConfigurationLayer that ConfigurationLayer::ALL does not list");
        }
    }

    /// A report names a layer by its label, so two layers sharing one could not be told apart
    /// by a reader of that report.
    #[test]
    fn Test_Label_Should_Be_Distinct_Per_Layer()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|layer| return layer.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two layers share a wire spelling");
    }

    /// The declaration order is the precedence order, which is what `OD-POLICY-001` decided
    /// and what the derived [`Ord`] carries.
    ///
    /// Asserted rather than assumed, because the derive is invisible at every call site that
    /// depends on it: a resolver picks the highest layer that stated a field by comparing two
    /// of these values, and a reordered declaration would silently reverse that comparison
    /// while every label still read correctly.
    #[test]
    fn Test_A_Later_Layer_Should_Outrank_An_Earlier_One()
    {
        assert!(Subject::Repository > Subject::Default);
        assert!(Subject::CommandLine > Subject::Repository);
        assert!(Subject::TemporaryRunOverride > Subject::CommandLine);

        let mut ranked = ALL;
        ranked.sort_unstable();

        assert_eq!(ranked, ALL, "ALL is already in precedence order, lowest first");
    }
}
