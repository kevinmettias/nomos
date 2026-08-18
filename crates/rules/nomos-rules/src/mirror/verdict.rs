//! Deciding what one declared universe is owed, and saying it in a finding.

use super::{DeclaredUniverse, CheckIndex, Finding, Reach_Of, Applicability, GateCategory, RuleId, COMPLETENESS_MIRROR, SubjectId, Content_Digest, EvidenceClass, EnforcementReach, EnforcementBreach, UniverseKind};

/// What one universe's declaration amounts to, and what is really true of it.
///
/// Returns `None` when the universe is mirrored, because a rule that emits a finding per
/// subject it approves of produces a report in which the defects cannot be found.
pub(super) fn Judge(universe: &DeclaredUniverse, index: &CheckIndex<'_>) -> Option<Finding>
{
    let reach = Reach_Of(universe, &index.names);

    if reach.Is_Enforced()
    {
        return None;
    }

    let verdict = Verdict(universe, &reach, index);

    return Some(Shortcoming(universe, verdict));
}

/// A universe's shortfall as a finding.
///
/// An admitted gap does not depend on the index at all — nothing was resolved, so nothing
/// could have been missed — and stays `Supported` however short the index is. Only a claim
/// that failed to resolve inherits the doubt. The evidence is `Derived` either way: computed
/// from source by a deterministic rule, and no stronger than that source.
///
/// The subject is hashed from the *qualified* name (`D-134`), not `universe.name` alone.
/// Two universes named alike in two different crates are two different subjects; without
/// the qualifier they would hash to the same one and be indistinguishable in every finding,
/// suppression, or history keyed on it.
pub(super) fn Shortcoming(universe: &DeclaredUniverse, verdict: Judgment) -> Finding
{
    let Judgment {
        applicability,
        gate,
        summary,
    } = verdict;

    let qualified = match Qualifier_Of(&universe.path)
    {
        Some(qualifier) => format!("{qualifier}::{}", universe.name),
        None => universe.name.clone(),
    };

    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: universe.name.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate,
        summary,
        locations: vec![universe.path.clone()],
    };
}

/// The crate or test-suite directory a universe's path belongs to, if the path names one.
///
/// A pure function of the path string — no filesystem access and no `Cargo.toml` read.
/// `universe.rs`'s module doc explains why this crate carries no parser and reads no
/// files; the same discipline applies here. This workspace's real paths are
/// `crates/<band>/<crate-name>/src/...` or `<suite-root>/src/...` /
/// `<suite-root>/tests/...` (`tests/contract/tests/completeness_universes/table.rs`'s
/// `UNIVERSES` has no exception among its nineteen entries), so the qualifier is the path
/// segment immediately before the last `/src/` or `/tests/` marker — the crate or suite
/// directory the file lives in, which stays the same as the file moves within it and
/// differs across crates that happen to share a bare universe name.
///
/// `None` when neither marker is present. Every real universe path in this workspace has
/// one; a path without either is a synthetic fixture path from a unit test (`"a.rs"`,
/// `"b/c.rs"`), not a shape this rule is ever really handed. Guessing a qualifier there —
/// say, the whole path — would turn "the same universe moved file" into two identities,
/// which is exactly what this module's own same-universe-keeps-one-identity test pins
/// against. `None` leaves the name unqualified instead, which is the old behaviour and
/// the right one for a path that names no crate at all.
fn Qualifier_Of(path: &str) -> Option<&str>
{
    let split_at = ["/src/", "/tests/"]
        .iter()
        .filter_map(|marker| return path.rfind(marker))
        .max()?;

    let before = &path[..split_at];
    let leaf = before.rsplit('/').next()?;

    if leaf.is_empty()
    {
        return None;
    }

    return Some(leaf);
}

/// How the universe's shortfall is reported.
///
/// The claimed name is read back off the universe rather than off the breach, because the
/// breach's payload is `nomos-contracts`' shape and this is the rule's own claim. The two
/// cannot disagree: [`Reach_Of`] produces a breach only in the arm where `claimed_mirror`
/// is `Some` and none in the arm where it is `None`, so the `unwrap_or_default` below is
/// unreachable rather than a fallback with a meaning. An empty name matches every text,
/// which would downgrade rather than block — the safe direction, for the reason
/// [`Unread::Could_Have_Declared`] gives.
pub(super) fn Verdict(
    universe: &DeclaredUniverse,
    reach: &EnforcementReach,
    index: &CheckIndex<'_>,
) -> Judgment
{
    let Some(breach) = reach.breaches.first()
    else
    {
        return Admitted_Gap(universe);
    };

    let claimed = universe.claimed_mirror.as_deref().unwrap_or_default();

    return Unresolved_Claim(breach, claimed, index);
}

/// How to report a claimed mirror that did not resolve against the index.
///
/// The two gates in play are not the same gate, and collapsing them is the mistake this
/// whole module is about. `reach.computed` is what the *universe's* declared mirror amounts
/// to — `Unreachable` for a phantom. The gate returned here is what *this rule* does about
/// that, and a false claim of coverage is the one outcome worth failing a build over.
///
/// Unless the index is short of something that could have resolved *this* name. Then the
/// claim is not established as false — the name may be in the subject that was not read —
/// and reporting it as a phantom would be the rule manufacturing the one finding it is
/// entitled to stop a build over out of its own inability to look, which is the same defect
/// as reporting clean wearing the other face. `Can_Fail_A_Build` consults the applicability
/// as well as the gate, so the refusal is machinery `D-134` already built rather than a
/// second rule about severity.
///
/// `OD-RULES-001` asked that question of the whole run and this asks it of the claim, which
/// is the whole of `OD-RULES-002`. When the index is short of subjects that *could not*
/// have resolved this name, the judgment is made and the shortfall travels with it in the
/// summary rather than suppressing it: a reader who wants to know what the run did not see
/// is owed that on the finding, not instead of it.
pub(super) fn Unresolved_Claim(
    breach: &EnforcementBreach,
    claimed: &str,
    index: &CheckIndex<'_>,
) -> Judgment
{
    let Some(shortfall) = index.Shortfall_For(claimed)
    else
    {
        return Judgment {
            applicability: Applicability::Supported,
            gate: GateCategory::Blocking,
            summary: if index.unread.is_empty()
            {
                breach.Describe()
            }
            else
            {
                format!(
                    "{} — and the check index is short {} subject(s), none of whose text \
                     spells `{claimed}`, so no reading of them could have declared it",
                    breach.Describe(),
                    index.unread.len()
                )
            },
        };
    };

    return Judgment {
        applicability: shortfall.Applicability(),
        gate: GateCategory::Advisory,
        summary: format!("{} — and {}", breach.Describe(), shortfall.Describe()),
    };
}

/// How to report a universe that claims no mirror at all.
///
/// An admitted gap does not depend on the index in any way — nothing was resolved, so
/// nothing could have been missed — which is why no shortfall is consulted here.
pub(super) fn Admitted_Gap(universe: &DeclaredUniverse) -> Judgment
{
    return Judgment {
        applicability: Applicability::Supported,
        gate: GateCategory::Advisory,
        summary: format!(
            "declares no mirror, so nothing compares this list against the reality it \
             enumerates; a {} added without adding it here is outside every guard built \
             on it, and those guards then pass by not looking",
            match universe.kind
            {
                UniverseKind::Constant => "member",
                UniverseKind::Enumeration => "variant",
            }
        ),
    };
}

/// What one universe is owed: how far the rule can stand behind it, what the rule does
/// about it, and what it says.
///
/// Named rather than a triple. At three members a caller is counting positions, and these
/// three travel together through four functions — [`Verdict`], [`Unresolved_Claim`],
/// [`Admitted_Gap`] and [`Shortcoming`] — which is four places for a position to slip.
pub(super) struct Judgment
{
    pub(super) applicability: Applicability,
    pub(super) gate: GateCategory,
    pub(super) summary: String,
}
