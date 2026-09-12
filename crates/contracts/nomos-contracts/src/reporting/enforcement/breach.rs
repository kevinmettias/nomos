

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use serde::{Deserialize, Serialize};

use super::EnforcerRef;

/// A way an enforcement claim can be false.
///
/// Four of these are xvpe's taxonomy; the fifth — [`Breach::OutOfReach`] —
/// comes from the Nomos prototype, where it was the most expensive of the five. A rule
/// named a check, the check named the rule back, every cross-reference agreed, and the
/// rule was still unenforced in every language but one because the check imported a
/// single front end. It reported clean everywhere it could not see.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Breach
{
    /// The name resolves to nothing. No such check exists.
    ///
    /// Worse than declaring no enforcer at all: nothing runs, nothing can fail, and the
    /// declaration says the rule is covered so no reader looks twice.
    Phantom
    {
        /// The name that resolved to nothing.
        name: String,
    },
    /// The check exists, and it judges something else.
    ///
    /// What was verified was that the enforcer *exists*. Nothing verified that it
    /// *applies*.
    Misclaimed
    {
        /// The enforcer named.
        enforcer: EnforcerRef,
        /// What that enforcer actually judges.
        actual_subject: String,
    },
    /// The delegation names a real tool and a setting that tool is never configured
    /// with, so the delegated judgment never happens.
    ExternalUnconfigured
    {
        /// The delegated-to tool.
        tool: String,
        /// The setting that is absent from every shipped template.
        setting: String,
    },
    /// The check exists in the source tree but not in this build, behind a feature or
    /// build flag.
    ///
    /// Distinguished from [`Breach::Phantom`] so the finding is *true* — the
    /// remedy is a build configuration change, not writing a check.
    OptIn
    {
        /// The enforcer named.
        enforcer: EnforcerRef,
        /// The feature that would enable it.
        missing_feature: String,
    },
    /// The check exists, runs, and cannot reach most of what the rule binds.
    ///
    /// A false clean, and the most dangerous entry in this enum, because every
    /// cross-reference between the rule and the check agrees.
    OutOfReach
    {
        /// The enforcer named.
        enforcer: EnforcerRef,
        /// What the rule binds that this enforcer cannot judge.
        unreached: Vec<String>,
    },
}

impl Breach
{
    /// A one-line description naming both the defect and why it matters.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Phantom { name } => format!(
                "`{name}` resolves to no check, so this rule is declared enforced and never runs"
            ),
            Self::Misclaimed {
                enforcer,
                actual_subject,
            } => format!("`{enforcer}` exists but judges {actual_subject}, not this rule"),
            Self::ExternalUnconfigured { tool, setting } => format!(
                "`{tool}:{setting}` is delegated to but appears in no template this repository ships"
            ),
            Self::OptIn {
                enforcer,
                missing_feature,
            } => format!("`{enforcer}` is not in this build; it needs feature `{missing_feature}`"),
            Self::OutOfReach {
                enforcer,
                unreached,
            } => format!(
                "`{enforcer}` runs but cannot judge {}, where this rule reports clean without looking",
                unreached.join(", ")
            ),
        };
    }
}

#[cfg(test)]
mod tests
{
    use alloc::vec;
    use alloc::borrow::ToOwned;
    use super::*;

    const MINIMUM_USEFUL_DESCRIPTION_LENGTH: usize = 20;

    /// Every breach must say what is wrong in terms an author can act on. A breach that
    /// renders as a type name teaches nobody anything.
    #[test]
    fn Test_Every_Breach_Should_Describe_Itself_Usefully()
    {
        let breaches = [
            Breach::Phantom {
                name: "check-imaginary".to_owned(),
            },
            Breach::Misclaimed {
                enforcer: EnforcerRef::Check {
                    name: "check-scope-discipline".to_owned(),
                },
                actual_subject: "dot imports and mutable globals".to_owned(),
            },
            Breach::ExternalUnconfigured {
                tool: "editorconfig".to_owned(),
                setting: "CA1822".to_owned(),
            },
            Breach::OptIn {
                enforcer: EnforcerRef::Check {
                    name: "check-slow".to_owned(),
                },
                missing_feature: "heavy-checks".to_owned(),
            },
            Breach::OutOfReach {
                enforcer: EnforcerRef::Check {
                    name: "check-cohesion".to_owned(),
                },
                unreached: vec!["kotlin".to_owned()],
            },
        ];

        for breach in &breaches
        {
            let description = breach.Describe();

            assert!(
                description.len() > MINIMUM_USEFUL_DESCRIPTION_LENGTH,
                "{description} is too terse to act on"
            );
        }
    }
}
