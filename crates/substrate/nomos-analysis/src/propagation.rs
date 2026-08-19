//! Reachability over an opaque dependency graph.
//!
//! `Spread` is the mechanism `docs/records/D-130` names as an XVPE candidate — dirty
//! propagation over a dependents adjacency map — kept free of fact-identity vocabulary
//! per `docs/records/D-135` and `docs/records/D-138`: its own signatures name only
//! [`Digest128`] and generic sets, never [`crate::FactKey`], [`crate::FactIdentity`],
//! [`crate::component::Component`] or [`crate::GenerationCause`]. `D-138` is why this is
//! a module rather than a crate — Nomos needs it before XVPE can hold it — and why the
//! boundary is drawn here anyway: the later rename-and-move is mechanical only if the
//! thing being moved never learned the vocabulary of what called it.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use nomos_contracts::Digest128;

/// Visits every node reachable from `roots` by following `dependents` edges outward,
/// each at most once — the "seen" set makes a cycle in the graph terminate the walk
/// rather than loop it.
///
/// `roots` are not passed to `on_reach`; the caller has already decided they belong.
/// Every other node the walk discovers is offered to `on_reach` exactly once, in the
/// order first discovered. Declining a node (`on_reach` returns `false`) stops the walk
/// at that node: nothing reachable only through it is visited.
pub(crate) fn Spread(
    dependents: &BTreeMap<Digest128, BTreeSet<Digest128>>,
    roots: Vec<Digest128>,
    mut on_reach: impl FnMut(Digest128) -> bool,
)
{
    let mut seen: BTreeSet<Digest128> = roots.iter().copied().collect();
    let mut frontier = roots;

    while let Some(digest) = frontier.pop()
    {
        let Some(downstream) = dependents.get(&digest)
        else
        {
            continue;
        };

        for consumer in downstream.iter().copied().collect::<Vec<_>>()
        {
            if seen.insert(consumer) && on_reach(consumer)
            {
                frontier.push(consumer);
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::Spread;
    use nomos_contracts::Digest128;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    const SOURCE: &str = include_str!("propagation.rs");

    /// `D-138`'s cost claim — a later rename is mechanical rather than a rewrite — only
    /// holds while this module's actual code never learns what a fact, a key or a
    /// generation is. The module doc comment above names those types by way of
    /// explaining the rule, so this checks code only: comment lines, and everything
    /// from `#[cfg(test)]` on (this very assertion would otherwise quote itself into a
    /// failure), are excluded before the search.
    #[test]
    fn Test_This_Modules_Code_Should_Carry_No_Fact_Identity_Vocabulary()
    {
        let code = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);
        let code: String = code
            .lines()
            .filter(|line| return !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        for forbidden in ["FactIdentity", "FactKey", "GenerationCause", "Component"]
        {
            assert!(
                !code.contains(forbidden),
                "propagation.rs's code named {forbidden}, which is fact-identity vocabulary \
                 this module must stay free of per D-135 and D-138"
            );
        }
    }

    fn Digest(byte: u8) -> Digest128
    {
        return Digest128::From_Bytes([byte; 16]);
    }

    #[test]
    fn Test_Spread_Should_Reach_Every_Downstream_Node_Once()
    {
        let a = Digest(1);
        let b = Digest(2);
        let c = Digest(3);

        let mut dependents: BTreeMap<Digest128, BTreeSet<Digest128>> = BTreeMap::new();
        dependents.insert(a, BTreeSet::from([b]));
        dependents.insert(b, BTreeSet::from([c]));

        let mut reached: Vec<Digest128> = Vec::new();
        Spread(&dependents, vec![a], |digest| {
            reached.push(digest);
            return true;
        });

        assert_eq!(reached, vec![b, c]);
    }

    #[test]
    fn Test_Spread_Should_Terminate_On_A_Cycle()
    {
        let a = Digest(1);
        let b = Digest(2);

        let mut dependents: BTreeMap<Digest128, BTreeSet<Digest128>> = BTreeMap::new();
        dependents.insert(a, BTreeSet::from([b]));
        dependents.insert(b, BTreeSet::from([a]));

        let mut reached: Vec<Digest128> = Vec::new();
        Spread(&dependents, vec![a], |digest| {
            reached.push(digest);
            return true;
        });

        assert_eq!(reached, vec![b]);
    }

    #[test]
    fn Test_Declining_A_Node_Should_Stop_The_Walk_There()
    {
        let a = Digest(1);
        let b = Digest(2);
        let c = Digest(3);

        let mut dependents: BTreeMap<Digest128, BTreeSet<Digest128>> = BTreeMap::new();
        dependents.insert(a, BTreeSet::from([b]));
        dependents.insert(b, BTreeSet::from([c]));

        let mut reached: Vec<Digest128> = Vec::new();
        Spread(&dependents, vec![a], |digest| {
            reached.push(digest);
            return digest != b;
        });

        assert_eq!(reached, vec![b]);
    }
}
