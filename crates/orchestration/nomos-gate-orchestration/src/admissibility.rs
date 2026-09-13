//! Whether one crate may name another, asked before any manifest carries the edge.
//!
//! Every other answer this crate gives is retrospective. `Check_Dependency_Direction` reads
//! `nomos.cap.dependency.edges`, which a provider derives by running `cargo metadata` over
//! manifests on disk, so an edge is judged once it has been written and never before. This
//! module is the one prospective answer: a crate pair, and what the architecture says about
//! it, with nothing on disk.
//!
//! # Why this is cheap, and why that is the argument for it existing
//!
//! `OD-GATE-026` measured the two answers. `nomos gate run` over this workspace takes 13,127
//! milliseconds and `nomos gate explain` takes 40, and this answer is in `explain`'s class,
//! not `run`'s — it reads no manifest, runs no subprocess, walks no tree and touches no
//! store. But the thirteen seconds is not the cost that mattered. The cost that mattered is
//! that the retrospective answer requires the edge to exist, so a developer asking "may I?"
//! has to do the thing first. `OD-RULES-020` records what that cost when the answer turned
//! out to be no: `nomos-workflow-orchestration` renumbered three times across two items in a
//! single session.
//!
//! # Why the answer comes through `Zone_Of` and `Permits`
//!
//! And never by reaching around them into `nomos_rules::ZONES` directly. That is the one
//! constraint `OD-GATE-026` puts on this module, and it exists because `OD-RULES-029` decided
//! the layering declaration becomes data read from the repository under check rather than a
//! table compiled into `nomos-rules`. The question this module asks is the same either way;
//! what that migration changes is where those two functions get their answer. Reading the
//! table would harden the half that is moving.

use nomos_rules::{Permits, SAME_ZONE_EDGES, Zone_Of};

/// The crate that would do the naming.
pub struct DependingCrate<'a>(pub &'a str);

/// The crate that would be named.
pub struct DependedCrate<'a>(pub &'a str);

/// What the architecture says about an edge that does not exist yet.
///
/// Three outcomes rather than two, and the third is the one that makes this safe to consult.
/// `Zone_Of` returns nothing for a crate with no declared zone, which is the normal state of
/// every crate in every repository but this one, and `OD-RULES-003` already decided what that
/// case is owed: a positive statement that no judgment was reached, never a judgment.
///
/// **There is deliberately no default of [`Admissibility::Permitted`].** A prospective check
/// that answers "fine" about a crate it has never heard of is worse than no check at all,
/// because the only caller is somebody who does not yet know the answer and has no way to
/// tell a real yes from a shrug.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admissibility
{
    /// Both crates have a declared zone and the architecture admits the edge.
    Permitted,
    /// Both crates have a declared zone and the architecture does not admit the edge.
    Refused,
    /// At least one of the two has no declared zone, so there is nothing to judge against.
    NotJudged,
}

impl Admissibility
{
    /// Whether this answer is a judgment at all.
    ///
    /// A predicate rather than `== NotJudged` at each call site: a caller asking "did I get an
    /// answer" should not have to know which variant means "no", and `OD-RULES-003`'s whole
    /// point is that the absence of a judgment is a thing said rather than a thing inferred.
    #[must_use]
    pub const fn Is_A_Judgment(self) -> bool
    {
        return !matches!(self, Self::NotJudged);
    }
}

/// Whether `depending` may name `depended`, judged against the declared architecture alone.
///
/// The same-zone case is asked separately because the zone lattice cannot express it:
/// `Permits` refuses every zone against itself deliberately — two providers of one capability
/// must not be able to name each other — and `SAME_ZONE_EDGES` is the named list of pairs for
/// which that refusal is lifted. Asking `Permits` alone would refuse thirteen edges this
/// workspace has already decided are correct.
///
/// A crate naming itself is not an edge and is refused rather than admitted by the same-zone
/// list, which names pairs of two different crates.
#[must_use]
pub fn Admits(depending: DependingCrate<'_>, depended: DependedCrate<'_>) -> Admissibility
{
    let DependingCrate(depending) = depending;
    let DependedCrate(depended) = depended;

    let (Some(from), Some(to)) = (Zone_Of(depending), Zone_Of(depended))
    else
    {
        return Admissibility::NotJudged;
    };

    if from == to
    {
        return Is_A_Declared_Peer(depending, depended);
    }

    if Permits(from, to)
    {
        return Admissibility::Permitted;
    }

    return Admissibility::Refused;
}

/// Whether two crates sharing a zone are one of the pairs `SAME_ZONE_EDGES` names.
///
/// Directed, as the list is: that a composition root may name a provider does not mean the
/// provider may name the composition root, and a list read in both directions would admit
/// exactly the cycles a zone forbidding its own members exists to prevent.
fn Is_A_Declared_Peer(depending: &str, depended: &str) -> Admissibility
{
    if SAME_ZONE_EDGES.iter().any(|(from, to)| return *from == depending && *to == depended)
    {
        return Admissibility::Permitted;
    }

    return Admissibility::Refused;
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A real permitted edge from this workspace's own graph.
    #[test]
    fn Test_An_Edge_The_Architecture_Admits_Should_Be_Permitted()
    {
        let answer = Admits(DependingCrate("nomos-rules"), DependedCrate("nomos-contracts"));

        assert_eq!(answer, Admissibility::Permitted);
        assert!(answer.Is_A_Judgment());
    }

    /// The inverse of that edge, which the lattice refuses.
    ///
    /// `nomos-contracts` is `Protocol`, the zone `Permits` refuses every target from, so this
    /// is the direction the rule exists to catch and the one a developer would most want
    /// answered before writing it.
    #[test]
    fn Test_An_Edge_The_Architecture_Refuses_Should_Be_Refused()
    {
        let answer = Admits(DependingCrate("nomos-contracts"), DependedCrate("nomos-rules"));

        assert_eq!(answer, Admissibility::Refused);
        assert!(answer.Is_A_Judgment());
    }

    /// A crate with no declared zone is not judged, and is emphatically not permitted.
    ///
    /// This is the assertion that keeps the verb honest. Every crate of every other
    /// repository is in this state, and an answer of `Permitted` here would be a prospective
    /// check telling its only kind of caller that anything is fine.
    #[test]
    fn Test_A_Crate_With_No_Declared_Zone_Should_Not_Be_Judged()
    {
        let unknown = Admits(DependingCrate("serde_json"), DependedCrate("nomos-contracts"));
        let named = Admits(DependingCrate("nomos-rules"), DependedCrate("serde_json"));

        assert_eq!(unknown, Admissibility::NotJudged);
        assert_eq!(named, Admissibility::NotJudged);
        assert!(!unknown.Is_A_Judgment() && !named.Is_A_Judgment());
    }

    /// The same-zone exception is honoured, which `Permits` alone would refuse.
    #[test]
    fn Test_A_Declared_Same_Zone_Peer_Should_Be_Permitted()
    {
        let Some((depending, depended)) = SAME_ZONE_EDGES.first()
        else
        {
            panic!("SAME_ZONE_EDGES is empty, so this test proved nothing about the exception");
        };

        let answer = Admits(DependingCrate(depending), DependedCrate(depended));

        assert_eq!(answer, Admissibility::Permitted, "{depending} -> {depended}");
    }

    /// Two crates sharing a zone that are not a declared peer pair are refused.
    ///
    /// Named apart from the test above because the two together are what say the same-zone
    /// list is read as a list rather than as "same zone is fine".
    #[test]
    fn Test_An_Undeclared_Same_Zone_Pair_Should_Be_Refused()
    {
        let answer = Admits(DependingCrate("nomos-model"), DependedCrate("nomos-store"));

        assert_eq!(answer, Admissibility::Refused);
    }

    /// The same-zone list is directed, and reading it in both directions would admit a cycle.
    #[test]
    fn Test_A_Declared_Peer_Pair_Should_Not_Be_Permitted_Backwards()
    {
        let Some((depending, depended)) = SAME_ZONE_EDGES.first()
        else
        {
            panic!("SAME_ZONE_EDGES is empty, so this test proved nothing about direction");
        };

        let backwards = Admits(DependingCrate(depended), DependedCrate(depending));

        assert_eq!(backwards, Admissibility::Refused, "{depended} -> {depending}");
    }

    /// A crate naming itself is not an edge.
    #[test]
    fn Test_A_Crate_Naming_Itself_Should_Be_Refused()
    {
        let answer = Admits(DependingCrate("nomos-rules"), DependedCrate("nomos-rules"));

        assert_eq!(answer, Admissibility::Refused);
    }
}
