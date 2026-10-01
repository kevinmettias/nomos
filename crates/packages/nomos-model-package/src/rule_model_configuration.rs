//! One rule's independently-configured check and fix execution.

use crate::ModelExecutionProfile;

/// `MODEL-ROUTE-002`: "Check/review execution and correction/fix execution shall be
/// independently configurable. A rule may use one backend, model, effort level,
/// context policy, tool set, and budget to judge a finding and another to propose or
/// implement remediation."
///
/// Only "backend, model, effort level" of the sentence's six-item list has a real
/// built type today -- [`ModelSelector`](crate::ModelSelector)/[`crate::EffortLevel`]
/// inside [`ModelExecutionProfile`]. "Context policy", "tool set", and "budget" name
/// domains with no existing shape anywhere in this workspace; inventing field shapes
/// for those three now would fabricate a design `ModelExecutionProfile`'s own prior
/// increment deliberately did not build. The genuinely buildable kernel is the
/// pairing itself: "one ... to judge a finding and another to propose or implement
/// remediation" is two independently-assigned [`ModelExecutionProfile`] values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleModelConfiguration
{
    /// The profile used to judge a finding ("check/review execution").
    pub check: ModelExecutionProfile,
    /// The profile used to propose or implement remediation ("correction/fix
    /// execution").
    pub fix: ModelExecutionProfile,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{EffortLevel, ModelSelector};

    #[test]
    fn Test_A_Configuration_May_Assign_Different_Profiles_To_Check_And_Fix()
    {
        let check = ModelExecutionProfile::New(ModelSelector::BackendFamily("fast-review".to_owned()), EffortLevel::Low);
        let fix = ModelExecutionProfile::New(ModelSelector::BackendFamily("careful-fix".to_owned()), EffortLevel::High);

        let configuration = RuleModelConfiguration {
            check: check.clone(),
            fix: fix.clone(),
        };

        assert_eq!(configuration.check, check);
        assert_eq!(configuration.fix, fix);
        assert_ne!(configuration.check, configuration.fix);
    }
}
