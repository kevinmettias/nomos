//! Why a declared rule set and a registered implementation set could not be matched.

use nomos_contracts::RuleId;

/// What [`crate::Resolve_Rules`] refused, and which side of the match it found wanting.
///
/// `OD-RULES-022` decided resolution is refused in both directions, and named what each
/// direction means: "A declaration naming no implementation, when it claims to be mechanical,
/// is a rule the gate would report and cannot perform. An implementation with no declaration
/// is the eighteen exported-but-uncomposed rules, which read to any reader of the crate as
/// though they are in force."
///
/// `Debug` only, no `Display`, matching `nomos_rules::RuleRegistryError` — the nearest
/// precedent for a composition-root defect in this workspace, and what
/// `nomos_api::GatePlanResponse` already prints for its own registry error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleResolutionError
{
    /// No rule was declared at all.
    ///
    /// Refused rather than resolved to an empty set. A run over no declarations judges
    /// nothing and would otherwise report exactly as a run that judged everything and found
    /// nothing, which is `OD-GATE-001`'s own defect: this crate already keeps
    /// [`crate::CheckOutcome::NoSource`] apart from a clean judgment for the same reason, one
    /// layer down.
    NothingDeclared,
    /// The two sets do not match, in one or more of three ways.
    ///
    /// All three lists are reported together rather than failing at the first, because an
    /// author who has just moved a rule between the two sides usually has more than one to
    /// fix and a refusal naming one of three is a refusal they will meet three times. Each
    /// list is sorted, so the same disagreement reads the same way twice.
    Disagreed
    {
        /// Declared mechanical, and no implementation is registered for it. The gate would
        /// report this rule and could not perform it.
        unimplemented: Vec<RuleId>,
        /// An implementation is registered and nothing declares it. This is the shape of the
        /// eighteen rules `P46-UNCOMPOSED-RULES-ARE-COUNTED` had to find by hand-diffing two
        /// lists.
        undeclared: Vec<RuleId>,
        /// Declared model-judged, and an implementation is registered anyway.
        ///
        /// The declaration says nothing judges this rule mechanically while linked code does,
        /// so a real judgment would sit unrun behind a declaration claiming it does not
        /// exist. That is the same defect as `undeclared` wearing a declaration, and it is
        /// listed separately because the repair is the opposite one: change the judgment,
        /// not the registration.
        contradicted: Vec<RuleId>,
    },
}
