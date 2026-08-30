//! Which of several usable offers answers.
//!
//! A floor decides which offers are *usable*. This decides which usable offer is *chosen*,
//! and it is a separate question: with one provider it never arose, and until this module
//! existed it was answered by whichever provider's name sorted first.
//!
//! # The rule
//!
//! The offer that no usable offer is strictly stronger than answers. A named preference
//! outranks that, because a caller saying what it wants is not something the registry is in
//! a position to overrule. Every other usable offer comes back alongside the chosen one, so
//! a caller can always see what it was chosen over.
//!
//! # Why the guarantee decides it rather than a cost
//!
//! [`Guarantee`] says how good an answer is and nothing about what it costs, and the
//! reflex objection to strongest-wins is that it therefore turns "I will accept an
//! approximation" into "give me the best you have". That objection is about *spending* a
//! weaker offer, not about *ranking* one, and the two are separable: the guarantee ranks,
//! and the caller decides how far down the ranking to spend. [`Selection::Weaker`] is what
//! it spends. Adding a cost axis to `Guarantee` would put a scheduling concern in the crate
//! every non-Rust peer reimplements, to answer a question the caller was already answering.
//!
//! # Why there is no total order
//!
//! [`Guarantee::Satisfies`] is deliberately not a score — "every axis, not a score", because
//! a weighted average would let a very sound syntactic provider answer a question that needs
//! name resolution. What it gives is a partial order, and a partial order has maximal
//! elements rather than a maximum. Where two offers are equivalent or incomparable this
//! module does not invent a ranking; it reports that it did not decide, and the caller that
//! needs determinism can check.

use super::Standing;
use crate::ProviderOffer;
use nomos_contracts::ProviderId;

/// Which usable offer answers, and every usable offer it was chosen over.
///
/// `chosen` together with `alternatives` is exactly the set of offers that cleared the
/// floor — no usable offer is dropped on the way out. That is the property that makes a
/// lowered floor mean something: a caller that widened what it would accept in order to buy
/// coverage can reach what it bought, rather than being handed the weakest thing it said it
/// would tolerate and told the requirement was met.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection
{
    /// The offer that answers.
    pub chosen: ProviderOffer,
    /// Every other usable offer, strongest first.
    ///
    /// "Strongest first" only where the guarantee ranks them. Where it does not — see
    /// [`Standing::Equivalent`] and [`Standing::Incomparable`] — the order here is the
    /// order the registry holds its offers in, which is by provider name, and that is a
    /// deterministic tiebreak rather than a judgement. [`Selection::Unranked`] is how a
    /// caller tells the two apart.
    pub alternatives: Vec<ProviderOffer>,
}

impl Selection
{
    /// Picks from the offers that cleared the floor.
    ///
    /// Returns `None` for an empty set, because a selection with nothing chosen is not a
    /// selection and the caller has a different thing to report.
    pub(crate) fn Over(usable: Vec<ProviderOffer>, preferred: Option<&ProviderId>) -> Option<Self>
    {
        let mut ranked = Ranked_Offers(usable);

        if ranked.is_empty()
        {
            return None;
        }

        // The head is the strongest unless the caller named someone. Removing one element
        // from a ranked list leaves the rest ranked, so an honoured preference costs the
        // alternatives nothing — including the stronger offer it was honoured over, which
        // stays visible rather than being dropped as no longer relevant.
        let head = preferred
            .and_then(|name| {
                return ranked.iter().position(|offer| return &offer.provider == name);
            })
            .unwrap_or(0);
        let chosen = ranked.remove(head);

        return Some(Self {
            chosen,
            alternatives: ranked,
        });
    }

    /// How `other` stands against the offer that answered.
    #[must_use]
    pub fn Standing_Of(&self, other: &ProviderOffer) -> Standing
    {
        return Standing::Of(&other.guarantee, &self.chosen.guarantee);
    }

    /// Whether something usable and strictly stronger than the answer was passed over.
    ///
    /// Only ever true when a preference was honoured, because that is the only way the
    /// rule chooses anything but a maximal offer. A caller that named no preference and
    /// reads `true` here is reading a defect in this module.
    #[must_use]
    pub fn Has_Passed_Over_Stronger(&self) -> bool
    {
        return self
            .alternatives
            .iter()
            .any(|offer| return self.Standing_Of(offer) == Standing::Stronger);
    }

    /// The usable offers strictly weaker than the answer, strongest first.
    ///
    /// What a caller that lowered its floor for coverage actually spends: the answer is
    /// the best available, and these are what remain when the best cannot answer for some
    /// particular subject. The registry has no subject and so cannot make that call.
    #[must_use]
    pub fn Weaker(&self) -> Vec<&ProviderOffer>
    {
        return self
            .alternatives
            .iter()
            .filter(|offer| return self.Standing_Of(offer) == Standing::Weaker)
            .collect();
    }

    /// The usable offers the guarantee could not rank against the answer.
    ///
    /// Non-empty means this selection reports a choice the rule did not make. The offer
    /// that answers is still the first by provider name among them, which is deterministic
    /// and is not a reason — a composition root that needs its provider choice to be a
    /// decision asserts this is empty.
    #[must_use]
    pub fn Unranked(&self) -> Vec<&ProviderOffer>
    {
        return self
            .alternatives
            .iter()
            .filter(|offer| return !self.Standing_Of(offer).Is_Decided())
            .collect();
    }
}

/// `offers`, strongest first, keeping input order where the guarantee does not rank.
///
/// Repeated selection of a maximal element rather than `sort_by`. A comparator built from a
/// partial order is not transitive — A stronger than B, B incomparable to C, C incomparable
/// to A admits no total order — and a sort handed one produces an order anyway, which is the
/// same defect as name order with more machinery in front of it.
///
/// The inner scan is correct because the strict part of a preorder is transitive: the
/// running best only ever moves up, so anything strictly stronger than the final best would
/// have displaced it when it was scanned.
fn Ranked_Offers(mut offers: Vec<ProviderOffer>) -> Vec<ProviderOffer>
{
    let mut ranked = Vec::with_capacity(offers.len());

    while let Some(first) = offers.first()
    {
        let mut best = 0;
        let mut strongest = first.guarantee;

        for (candidate, offer) in offers.iter().enumerate().skip(1)
        {
            if Standing::Of(&offer.guarantee, &strongest) == Standing::Stronger
            {
                best = candidate;
                strongest = offer.guarantee;
            }
        }

        ranked.push(offers.remove(best));
    }

    return ranked;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::Guarantee;
    use nomos_contracts::{
        Assurance, CapabilityId, ContractVersion, FactVariant, IncrementalGranularity,
    };

    fn Offer_With_Guarantee(provider: &str, guarantee: Guarantee) -> ProviderOffer
    {
        return ProviderOffer {
            provider: ProviderId::New(provider),
            capability: CapabilityId::New("nomos.cap.test.items"),
            version: ContractVersion::New(1, 0),
            guarantee,
        };
    }

    fn Parse() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    fn Scan() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Approximate,
            Assurance::Unsound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    /// Incomparable to both of the above: stronger than a parse on variant, weaker on
    /// soundness and on the granularity it can refresh at.
    fn Coarse_Semantic() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Unknown,
            Assurance::Unknown,
            IncrementalGranularity::Project,
        );
    }

    #[test]
    fn Test_Standing_Should_Separate_Equivalent_From_Incomparable()
    {
        assert_eq!(Standing::Of(&Parse(), &Scan()), Standing::Stronger);
        assert_eq!(Standing::Of(&Scan(), &Parse()), Standing::Weaker);
        assert_eq!(Standing::Of(&Parse(), &Parse()), Standing::Equivalent);
        assert_eq!(
            Standing::Of(&Coarse_Semantic(), &Parse()),
            Standing::Incomparable,
            "it resolves names and cannot refresh a file on its own; neither reaches the \
             other"
        );

        assert!(Standing::Of(&Parse(), &Scan()).Is_Decided());
        assert!(!Standing::Of(&Parse(), &Parse()).Is_Decided());
        assert!(!Standing::Of(&Coarse_Semantic(), &Parse()).Is_Decided());
    }

    /// The name that sorts first is the weaker one, which is the shape the real registry
    /// has and the reason this rule exists.
    #[test]
    fn Test_Ranking_Should_Put_The_Strongest_First_Whatever_The_Names_Are()
    {
        let ranked = Named_Providers(&Ranked_Offers(vec![
            Offer_With_Guarantee("a.scan", Scan()),
            Offer_With_Guarantee("z.parse", Parse()),
        ]));

        assert_eq!(ranked, vec!["z.parse", "a.scan"]);
    }

    /// Where the guarantee decides nothing, input order survives — deterministic, and not
    /// claimed as a ranking.
    #[test]
    fn Test_Ranking_Should_Keep_Input_Order_Where_The_Guarantee_Does_Not_Rank()
    {
        let ranked = Named_Providers(&Ranked_Offers(vec![
            Offer_With_Guarantee("first", Parse()),
            Offer_With_Guarantee("second", Coarse_Semantic()),
        ]));
        let reversed = Named_Providers(&Ranked_Offers(vec![
            Offer_With_Guarantee("second", Coarse_Semantic()),
            Offer_With_Guarantee("first", Parse()),
        ]));

        assert_eq!(ranked, vec!["first", "second"]);
        assert_eq!(
            reversed,
            vec!["second", "first"],
            "two offers the guarantee cannot rank come back in the order they arrived; the \
             registry's input is name-ordered, which is what makes that deterministic"
        );
    }

    /// A maximal element exists even when nothing is comparable to everything, and the
    /// rank never leaves one behind.
    ///
    /// Note what is *not* claimed: `scan` outranks `semantic` here only because it arrived
    /// first, and the two are incomparable. The claim is about the pairs the guarantee
    /// ranks, and `parse` above `scan` is the whole of it.
    #[test]
    fn Test_Ranking_Should_Return_Every_Offer_It_Was_Given()
    {
        let ranked = Named_Providers(&Ranked_Offers(vec![
            Offer_With_Guarantee("scan", Scan()),
            Offer_With_Guarantee("parse", Parse()),
            Offer_With_Guarantee("semantic", Coarse_Semantic()),
        ]));

        let at = |provider: &str| {
            return ranked
                .iter()
                .position(|name| return name == provider)
                .expect("every offer given is returned");
        };

        assert_eq!(ranked.len(), 3, "an offer was dropped: {ranked:?}");
        assert_eq!(at("parse"), 0, "nothing is stronger than the parser here");
        assert!(at("scan") > at("parse"), "and the parser is strictly stronger than the scan");
    }

    #[test]
    fn Test_A_Selection_Should_Account_For_Every_Usable_Offer()
    {
        let selection = Selection::Over(
            vec![Offer_With_Guarantee("a.scan", Scan()), Offer_With_Guarantee("z.parse", Parse())],
            None,
        )
        .expect("two usable offers");

        assert_eq!(selection.chosen.provider, ProviderId::New("z.parse"));
        assert_eq!(selection.alternatives.len(), 1);
        assert!(!selection.Has_Passed_Over_Stronger());
        assert_eq!(selection.Weaker().len(), 1);
        assert!(selection.Unranked().is_empty());
    }

    /// A preference is the caller's, and the registry does not overrule it — but the
    /// stronger offer it was honoured over stays visible.
    #[test]
    fn Test_An_Honoured_Preference_Should_Keep_The_Stronger_Offer_Visible()
    {
        let selection = Selection::Over(
            vec![Offer_With_Guarantee("a.scan", Scan()), Offer_With_Guarantee("z.parse", Parse())],
            Some(&ProviderId::New("a.scan")),
        )
        .expect("two usable offers");

        assert_eq!(selection.chosen.provider, ProviderId::New("a.scan"));
        assert!(
            selection.Has_Passed_Over_Stronger(),
            "the parser was available and stronger, and a caller that cannot see that \
             cannot record what its preference cost"
        );
        assert!(selection.Weaker().is_empty());
    }

    #[test]
    fn Test_An_Empty_Set_Should_Not_Produce_A_Selection()
    {
        assert!(Selection::Over(Vec::new(), None).is_none());
    }

    /// Provider names in order, which is what every ranking assertion here is about.
    fn Named_Providers(offers: &[ProviderOffer]) -> Vec<String>
    {
        return offers
            .iter()
            .map(|offer| return offer.provider.As_Str().to_owned())
            .collect();
    }
}
