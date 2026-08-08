//! Whether the thing that claims to enforce a rule can actually fail a build.
//!
//! Adopted from the sibling xvpe workspace, which invented the distinction and proved
//! it against a real corpus. The idea is one sentence long:
//!
//! > Naming an enforcer is a claim about **existence**. Whether that enforcer runs is a
//! > claim about **execution**. Only the second one can be checked against reality, and
//! > it is the one that matters.
//!
//! Both prototypes measured what happens without it. In the Nomos prototype, 747 of 860
//! standards declared `enforced_by: [review]` — 87% of the law was a wish — and fifty
//! landed documents named a tool that did not exist anywhere, found only by reading them
//! all by hand. In xvpe, six rules currently declare an enforcer that no workflow
//! invokes, including *both* rules that define its strategy-surface admission bar.
//!
//! The point is not that unenforced rules are bad. It is that an unenforced rule and an
//! enforced one must not look the same, because a rule that claims a check it does not
//! have teaches every reader to disbelieve the rest of the corpus.

use crate::identity::RuleId;
use serde::{Deserialize, Serialize};

const BLOCKING_LABEL: &str = "Blocking";
const ADVISORY_LABEL: &str = "Advisory";
const UNREACHABLE_LABEL: &str = "Unreachable";
const REVIEW_LABEL: &str = "Review";

/// What a rule's declared enforcers can actually do to a build.
///
/// This value is **derived**, never authored. It is computed by walking the real gate
/// wiring — from the gate roots outward — and asking what would happen if the rule were
/// violated. A rule may state its expectation, and a mismatch between the expectation
/// and this value is itself a finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GateCategory
{
    /// No mechanical enforcer is claimed. Honest, and the correct declaration for
    /// anything a machine cannot judge.
    Review,
    /// An enforcer is named, and nothing invokes it. The rule is declared enforced and
    /// never runs.
    Unreachable,
    /// A gate invokes the enforcer and discards its result. It reports; it cannot fail
    /// the build.
    Advisory,
    /// A gate invokes the enforcer and honors its result.
    Blocking,
}

impl GateCategory
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Blocking => BLOCKING_LABEL,
            Self::Advisory => ADVISORY_LABEL,
            Self::Unreachable => UNREACHABLE_LABEL,
            Self::Review => REVIEW_LABEL,
        };
    }

    /// Whether a violation of a rule in this category can fail a build.
    #[must_use]
    pub const fn Can_Fail_A_Build(self) -> bool
    {
        return matches!(self, Self::Blocking);
    }

    /// The category of a rule whose enforcers land in several categories.
    ///
    /// The strongest wins: running advisory in one place does not undo being enforced
    /// in another. Ordering the enum weakest-first is what makes this a `max` rather
    /// than a table nobody maintains.
    #[must_use]
    pub fn Strongest_Of(self, other: Self) -> Self
    {
        return core::cmp::max(self, other);
    }
}

impl core::fmt::Display for GateCategory
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// A named enforcer of a rule.
///
/// The name must be a **tool identity**, not a path and not a CI job name. xvpe records
/// the failure directly: one of its rules names
/// `[fast-tier-benchmarks, .github/workflows/pr-fast.yml, tools/perf/fast-tier-bench]`,
/// and because the resolver matches tool-module leaf names, none of the three resolves.
/// The rule silently classifies as unreachable while its own prose says it is advisory.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EnforcerRef
{
    /// A human judges this. The honest declaration when no tool can.
    Review,
    /// A registered check, named by its identity.
    Check
    {
        /// The check's registered name.
        name: String,
    },
    /// A setting in a tool this repository does not write, such as an analyzer
    /// configuration entry. The setting must exist in a template the repository ships,
    /// or the delegation is a claim with nothing behind it.
    External
    {
        /// The delegated-to tool, such as `editorconfig`.
        tool: String,
        /// The setting within it, such as `CA1822`.
        setting: String,
    },
}

impl core::fmt::Display for EnforcerRef
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Review => formatter.write_str("review"),
            Self::Check { name } => formatter.write_str(name),
            Self::External { tool, setting } => write!(formatter, "{tool}:{setting}"),
        };
    }
}

/// A way an enforcement claim can be false.
///
/// Four of these are xvpe's taxonomy; the fifth — [`EnforcementBreach::OutOfReach`] —
/// comes from the Nomos prototype, where it was the most expensive of the five. A rule
/// named a check, the check named the rule back, every cross-reference agreed, and the
/// rule was still unenforced in every language but one because the check imported a
/// single front end. It reported clean everywhere it could not see.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementBreach
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
    /// Distinguished from [`EnforcementBreach::Phantom`] so the finding is *true* — the
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

impl EnforcementBreach
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

/// What a rule claims about its enforcement, and what is actually true.
///
/// Three separate fields, and the separation is the whole design. A rule names its
/// enforcers (`declared`), states what it believes that amounts to (`expected`), and the
/// wiring walk says what it really amounts to (`computed`). Collapsing any two of these
/// destroys the property being protected:
///
/// - Dropping `expected` and inferring it from `declared` is what the first draft of
///   this type did, and it made a rule naming a check that nothing invokes read as
///   truthful — because it did name a check. Naming an enforcer is not a claim that the
///   enforcer runs; those are the two halves this whole module exists to separate.
/// - Overwriting `expected` with `computed` erases the disagreement, and the
///   disagreement *is* the finding.
///
/// A rule may honestly declare [`GateCategory::Unreachable`]. That is a true statement
/// about a real gap, and it is exactly what makes the gap countable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnforcementReach
{
    /// The rule this describes.
    pub rule: RuleId,
    /// The enforcers the rule names.
    pub declared: Vec<EnforcerRef>,
    /// What the rule claims those enforcers amount to. Authored.
    pub expected: GateCategory,
    /// What those enforcers actually amount to, derived from the real wiring.
    pub computed: GateCategory,
    /// Every way the declaration is false.
    pub breaches: Vec<EnforcementBreach>,
}

impl EnforcementReach
{
    /// Whether the rule's claims about its own enforcement are true.
    ///
    /// Three conditions, all necessary:
    ///
    /// 1. No breach was found — every named enforcer resolves, applies, is in this
    ///    build, and reaches what the rule binds.
    /// 2. The claimed gate matches the computed one.
    /// 3. Naming only `review` and claiming [`GateCategory::Review`] agree with each
    ///    other. A rule that names a real check while claiming `review` is understating
    ///    working enforcement; a rule that names only `review` while claiming to block
    ///    is claiming a machine that does not exist.
    #[must_use]
    pub fn Is_Truthful(&self) -> bool
    {
        if !self.breaches.is_empty()
        {
            return false;
        }
        if self.expected != self.computed
        {
            return false;
        }

        let declares_only_review = self
            .declared
            .iter()
            .all(|enforcer| matches!(enforcer, EnforcerRef::Review));

        return declares_only_review == (self.expected == GateCategory::Review);
    }

    /// Whether this rule is mechanically enforced in a way that can fail a build.
    ///
    /// Uses `computed`, never `expected`. What a rule claims about itself has no
    /// bearing on whether a violation actually stops anything.
    #[must_use]
    pub fn Is_Enforced(&self) -> bool
    {
        return self.breaches.is_empty() && self.computed.Can_Fail_A_Build();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Reach(
        declared: Vec<EnforcerRef>,
        expected: GateCategory,
        computed: GateCategory,
    ) -> EnforcementReach
    {
        return EnforcementReach {
            rule: RuleId::New("example-rule"),
            declared,
            expected,
            computed,
            breaches: Vec::new(),
        };
    }

    fn Named_Check(name: &str) -> EnforcerRef
    {
        return EnforcerRef::Check {
            name: name.to_owned(),
        };
    }

    #[test]
    fn Test_Only_Blocking_Should_Fail_A_Build()
    {
        assert!(GateCategory::Blocking.Can_Fail_A_Build());
        assert!(!GateCategory::Advisory.Can_Fail_A_Build());
        assert!(!GateCategory::Unreachable.Can_Fail_A_Build());
        assert!(!GateCategory::Review.Can_Fail_A_Build());
    }

    /// Running advisory in one place does not undo being enforced in another.
    #[test]
    fn Test_Strongest_Category_Should_Win()
    {
        assert_eq!(
            GateCategory::Advisory.Strongest_Of(GateCategory::Blocking),
            GateCategory::Blocking
        );
        assert_eq!(
            GateCategory::Unreachable.Strongest_Of(GateCategory::Advisory),
            GateCategory::Advisory
        );
        assert_eq!(
            GateCategory::Review.Strongest_Of(GateCategory::Unreachable),
            GateCategory::Unreachable
        );
    }

    #[test]
    fn Test_Review_Declaration_Should_Be_Truthful_When_Nothing_Enforces_It()
    {
        assert!(
            Reach(
                vec![EnforcerRef::Review],
                GateCategory::Review,
                GateCategory::Review
            )
            .Is_Truthful()
        );
    }

    /// The headline case, and the one the first draft of this type got wrong. A rule
    /// that names a check while claiming to block, when nothing invokes that check, is
    /// the state both prototypes accumulated. It must not read as truthful merely
    /// because it named something real.
    #[test]
    fn Test_Declaring_A_Check_That_Nothing_Invokes_Should_Not_Be_Truthful()
    {
        let overclaimed = Reach(
            vec![Named_Check("check-strategy-docs")],
            GateCategory::Blocking,
            GateCategory::Unreachable,
        );

        assert!(!overclaimed.Is_Truthful());
        assert!(!overclaimed.Is_Enforced());
    }

    /// The counterpart, and the reason `expected` exists as a separate field: a rule
    /// may *honestly* declare that its enforcer is unreachable. That is a true
    /// statement about a real gap, it is what makes the gap countable, and it must not
    /// be conflated with the overclaim above.
    #[test]
    fn Test_Honestly_Declared_Unreachable_Should_Be_Truthful()
    {
        let honest = Reach(
            vec![Named_Check("check-strategy-docs")],
            GateCategory::Unreachable,
            GateCategory::Unreachable,
        );

        assert!(honest.Is_Truthful(), "an admitted gap is an honest declaration");
        assert!(!honest.Is_Enforced(), "but it is still not enforcement");
    }

    /// The inverse understatement: claiming review while a gate really does enforce it.
    /// Less dangerous, still false, and it hides working enforcement from the roll-up.
    #[test]
    fn Test_Declaring_Review_While_A_Gate_Enforces_It_Should_Not_Be_Truthful()
    {
        assert!(
            !Reach(
                vec![EnforcerRef::Review],
                GateCategory::Blocking,
                GateCategory::Blocking
            )
            .Is_Truthful()
        );
    }

    /// Naming a real, running check while claiming only `review` understates it. The
    /// gate categories agree here, so only the declared/expected consistency rule
    /// catches this one.
    #[test]
    fn Test_Naming_A_Check_While_Claiming_Review_Should_Not_Be_Truthful()
    {
        assert!(
            !Reach(
                vec![Named_Check("check-orphan-modules")],
                GateCategory::Review,
                GateCategory::Review
            )
            .Is_Truthful()
        );
    }

    #[test]
    fn Test_A_Breach_Should_Make_A_Rule_Untruthful_Even_When_Blocking()
    {
        let mut reach = Reach(
            vec![Named_Check("check-cohesion")],
            GateCategory::Blocking,
            GateCategory::Blocking,
        );
        reach.breaches.push(EnforcementBreach::OutOfReach {
            enforcer: Named_Check("check-cohesion"),
            unreached: vec!["kotlin".to_owned(), "swift".to_owned()],
        });

        assert!(!reach.Is_Truthful());
        assert!(
            !reach.Is_Enforced(),
            "a check that cannot see most of what the rule binds is not enforcement"
        );
    }

    /// Every breach must say what is wrong in terms an author can act on. A breach that
    /// renders as a type name teaches nobody anything.
    #[test]
    fn Test_Every_Breach_Should_Describe_Itself_Usefully()
    {
        let breaches = [
            EnforcementBreach::Phantom {
                name: "check-imaginary".to_owned(),
            },
            EnforcementBreach::Misclaimed {
                enforcer: EnforcerRef::Check {
                    name: "check-scope-discipline".to_owned(),
                },
                actual_subject: "dot imports and mutable globals".to_owned(),
            },
            EnforcementBreach::ExternalUnconfigured {
                tool: "editorconfig".to_owned(),
                setting: "CA1822".to_owned(),
            },
            EnforcementBreach::OptIn {
                enforcer: EnforcerRef::Check {
                    name: "check-slow".to_owned(),
                },
                missing_feature: "heavy-checks".to_owned(),
            },
            EnforcementBreach::OutOfReach {
                enforcer: EnforcerRef::Check {
                    name: "check-cohesion".to_owned(),
                },
                unreached: vec!["kotlin".to_owned()],
            },
        ];

        for breach in &breaches
        {
            let description = breach.Describe();

            assert!(description.len() > 20, "{description} is too terse to act on");
        }
    }

    #[test]
    fn Test_External_Enforcer_Should_Render_With_Its_Colon_Form()
    {
        let external = EnforcerRef::External {
            tool: "editorconfig".to_owned(),
            setting: "CA1822".to_owned(),
        };

        assert_eq!(external.to_string(), "editorconfig:CA1822");
    }
}
