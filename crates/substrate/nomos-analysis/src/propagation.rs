//! Reachability over an opaque dependency graph.
//!
//! [`DependencyPropagation`] is the mechanism `docs/records/D-130` names as an XVPE
//! candidate — dirty propagation over a dependents adjacency map — kept free of
//! fact-identity vocabulary per `docs/records/D-135` and `docs/records/D-138`: its own
//! signatures name only [`Digest128`] and generic sets, never `FactKey`, `FactIdentity`,
//! `Component`, `GenerationCause`, `Broadening`, `Supersession` or `GuaranteeDigest`.
//! `D-138` is why this is a module rather than a crate — Nomos needs it before XVPE can
//! hold it — and why the boundary is drawn here anyway: the later rename-and-move is
//! mechanical only if the thing being moved never learned the vocabulary of what called
//! it.
//!
//! The trait exists so `MemoryFactStore` holds the walk behind a replaceable seam rather
//! than calling one hard-coded implementation: `D-130`'s deferred adoption is only cheap
//! later if something today already proves the mechanism can be swapped without touching
//! `FactStore` or its consumers, not merely that its code reads as generic.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use nomos_contracts::Digest128;

/// Something that can spread an invalidation outward through a dependency graph.
///
/// `roots` are not passed to `on_reach`; the caller has already decided they belong.
/// Every other node an implementation discovers by following `dependents` edges is
/// offered to `on_reach` exactly once, in the order first discovered. Declining a node
/// (`on_reach` returns `false`) stops the walk at that node: nothing reachable only
/// through it is visited. An implementation revisiting an already-seen node, or offering
/// a root to `on_reach`, is a defect in that implementation, not a case a caller must
/// guard against — this module's own tests check `LocalGraphPropagation` against exactly
/// those three properties.
pub(crate) trait DependencyPropagation
{
    fn Spread(
        &self,
        dependents: &BTreeMap<Digest128, BTreeSet<Digest128>>,
        roots: Vec<Digest128>,
        on_reach: &mut dyn FnMut(Digest128) -> bool,
    );
}

/// The one implementation this crate has today: an in-process walk over an owned
/// adjacency snapshot, using a stack so a long chain does not recurse.
///
/// `docs/records/D-122` and `docs/records/D-130` are why there is no second
/// implementation yet — moving this to XVPE needs a second product's materially
/// identical slice as evidence, and XVPE's own package and dependency state has not
/// opened Phase 5 — not a property of this type.
pub(crate) struct LocalGraphPropagation;

impl DependencyPropagation for LocalGraphPropagation
{
    fn Spread(
        &self,
        dependents: &BTreeMap<Digest128, BTreeSet<Digest128>>,
        roots: Vec<Digest128>,
        on_reach: &mut dyn FnMut(Digest128) -> bool,
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
}

#[cfg(test)]
mod tests
{
    use super::DependencyPropagation;
    use super::LocalGraphPropagation;
    use nomos_contracts::Digest128;
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    const SOURCE: &str = include_str!("propagation.rs");

    /// `D-138`'s cost claim — a later rename is mechanical rather than a rewrite — only
    /// holds while this module's actual code never learns what a fact, a key, a
    /// generation, a broadening or a supersession is. The module doc comment above names
    /// those types by way of explaining the rule, so this checks code only: comment
    /// lines, and everything from `#[cfg(test)]` on (this very assertion would otherwise
    /// quote itself into a failure), are excluded before the search.
    ///
    /// The negative control the reviewed correction asked for: a provider-broadening or
    /// supersession rule must not migrate in here merely because it participates in
    /// invalidation, so `Broadening`, `Supersession` and `GuaranteeDigest` are forbidden
    /// alongside the fact-identity types the first version of this test already covered.
    #[test]
    fn Test_This_Modules_Code_Should_Carry_No_Fact_Identity_Vocabulary()
    {
        let code = SOURCE.split("#[cfg(test)]").next().unwrap_or(SOURCE);
        let code: String = code
            .lines()
            .filter(|line| return !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        for forbidden in [
            "FactIdentity",
            "FactKey",
            "GenerationCause",
            "Component",
            "Broadening",
            "Supersession",
            "GuaranteeDigest",
        ]
        {
            assert!(
                !code.contains(forbidden),
                "propagation.rs's code named {forbidden}, which is fact-identity or \
                 invalidation-policy vocabulary this module must stay free of per D-135 \
                 and D-138"
            );
        }
    }

    fn Digest(byte: u8) -> Digest128
    {
        return Digest128::From_Bytes([byte; 16]);
    }

    /// A dependents adjacency map built from `(from, to)` edges, one insertion per edge.
    fn Graph(edges: &[(Digest128, Digest128)]) -> BTreeMap<Digest128, BTreeSet<Digest128>>
    {
        let mut dependents: BTreeMap<Digest128, BTreeSet<Digest128>> = BTreeMap::new();
        for (from, to) in edges.iter().copied()
        {
            dependents.entry(from).or_default().insert(to);
        }

        return dependents;
    }

    fn Spread(
        dependents: &BTreeMap<Digest128, BTreeSet<Digest128>>,
        roots: Vec<Digest128>,
        mut on_reach: impl FnMut(Digest128) -> bool,
    ) -> Vec<Digest128>
    {
        let mut reached: Vec<Digest128> = Vec::new();
        LocalGraphPropagation.Spread(dependents, roots, &mut |digest| {
            let keep_going = on_reach(digest);
            reached.push(digest);

            return keep_going;
        });

        return reached;
    }

    #[test]
    fn Test_Spread_Should_Reach_Every_Downstream_Node_Once()
    {
        let a = Digest(1);
        let b = Digest(2);
        let c = Digest(3);

        let dependents = Graph(&[(a, b), (b, c)]);

        let reached = Spread(&dependents, vec![a], |_| return true);

        assert_eq!(reached, vec![b, c]);
    }

    #[test]
    fn Test_Spread_Should_Terminate_On_A_Cycle()
    {
        let a = Digest(1);
        let b = Digest(2);

        let dependents = Graph(&[(a, b), (b, a)]);

        let reached = Spread(&dependents, vec![a], |_| return true);

        assert_eq!(reached, vec![b]);
    }

    #[test]
    fn Test_Declining_A_Node_Should_Stop_The_Walk_There()
    {
        let a = Digest(1);
        let b = Digest(2);
        let c = Digest(3);

        let dependents = Graph(&[(a, b), (b, c)]);

        let reached = Spread(&dependents, vec![a], |digest| return digest != b);

        assert_eq!(reached, vec![b]);
    }
}
