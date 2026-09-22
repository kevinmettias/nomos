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
use nomos_contracts::Guarantee;
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
/// # What is selected, and why it does not cost a scan per offer
///
/// The selection is a chain: start at the earliest offer still unranked, move to the
/// earliest offer strictly stronger than it, and repeat until nothing is stronger. What the
/// chain settles on is a maximal offer — the strict part of a preorder is transitive, so
/// anything stronger than the last link would have been reached from an earlier one.
///
/// A [`Guarantee`] is four small enumerations, so however many providers offer a capability
/// they carry at most a few hundred *distinct* guarantees, and every offer carrying the same
/// one is interchangeable to this chain. So the chain is walked over the distinct guarantees
/// rather than over the offers: each is held with its members in input order, and the
/// earliest unranked member of a class is the only one of it the chain can reach. That makes
/// a round cost the number of distinct guarantees rather than the number of offers, and the
/// whole ranking linear in the offers rather than quadratic.
///
/// It is the same answer, offer for offer and position for position, not merely the same
/// set: `Test_The_Fast_Ranking_Should_Agree_With_A_Scan_Per_Offer` holds it against a
/// literal scan-per-offer implementation over pseudo-random populations built from genuinely
/// incomparable guarantees.
fn Ranked_Offers(offers: Vec<ProviderOffer>) -> Vec<ProviderOffer>
{
    let mut classes = Guarantee_Classes(&offers);
    let mut unranked: Vec<Option<ProviderOffer>> = offers.into_iter().map(Some).collect();
    let mut ranked = Vec::with_capacity(unranked.len());

    while let Some(class) = Chain_From_The_Earliest_Offer(&classes)
    {
        let Some(position) = classes.Take_Earliest(class)
        else
        {
            break;
        };
        if let Some(offer) = unranked.get_mut(position).and_then(Option::take)
        {
            ranked.push(offer);
        }
    }

    return ranked;
}

/// The class the chain settles on: the earliest unranked offer, then whatever is strictly
/// stronger than it, until nothing is.
///
/// Terminating because each step moves strictly up a relation that is transitive and
/// irreflexive, over finitely many classes, so no class can be reached twice.
fn Chain_From_The_Earliest_Offer(classes: &GuaranteeClasses) -> Option<usize>
{
    let mut class = classes.Earliest_Class()?;

    while let Some(stronger) = classes.Earliest_Stronger_Class(class)
    {
        class = stronger;
    }

    return Some(class);
}

/// `offers` grouped by the guarantee they carry, in the order the guarantees first appear.
fn Guarantee_Classes(offers: &[ProviderOffer]) -> GuaranteeClasses
{
    let mut classes = GuaranteeClasses {
        guarantees: Vec::new(),
        members: Vec::new(),
        ranked: Vec::new(),
    };

    for (position, offer) in offers.iter().enumerate()
    {
        classes.Place(position, offer.guarantee);
    }

    return classes;
}

/// The offers of one ranking, grouped by the distinct guarantee they carry.
///
/// Grouped by guarantee *equality*, not by [`Standing::Equivalent`]: two different
/// guarantees can each reach everything the other does, and folding those together would
/// make this structure decide something the guarantee did not. Equal guarantees are
/// interchangeable to the ranking without deciding anything.
struct GuaranteeClasses
{
    /// The distinct guarantees, in the order they first appear among the offers.
    guarantees: Vec<Guarantee>,
    /// Per class, the input positions of the offers carrying that guarantee, in input order.
    members: Vec<Vec<usize>>,
    /// Per class, how many of its members have already been ranked. Its members before that
    /// count are out; the one at it is the earliest this class can still offer.
    ranked: Vec<usize>,
}

impl GuaranteeClasses
{
    /// Files the offer at `position` under `guarantee`, opening a class for a guarantee that
    /// has not appeared before.
    fn Place(&mut self, position: usize, guarantee: Guarantee)
    {
        let known = self.guarantees.iter().position(|seen| return *seen == guarantee);
        let class = if let Some(known) = known
        {
            known
        }
        else
        {
            self.guarantees.push(guarantee);
            self.members.push(Vec::new());
            self.ranked.push(0);
            self.guarantees.len().saturating_sub(1)
        };

        if let Some(members) = self.members.get_mut(class)
        {
            members.push(position);
        }
    }

    /// The input position of `class`'s earliest offer that is not yet ranked.
    fn Earliest(&self, class: usize) -> Option<usize>
    {
        let ranked = *self.ranked.get(class)?;

        return self.members.get(class)?.get(ranked).copied();
    }

    /// The class holding the earliest unranked offer of all, or `None` once every offer has
    /// been ranked.
    fn Earliest_Class(&self) -> Option<usize>
    {
        return (0..self.guarantees.len())
            .filter_map(|class| return Some((self.Earliest(class)?, class)))
            .min()
            .map(|(_, class)| return class);
    }

    /// The class holding the earliest unranked offer among those strictly stronger than
    /// `class`.
    fn Earliest_Stronger_Class(&self, class: usize) -> Option<usize>
    {
        let against = *self.guarantees.get(class)?;

        return (0..self.guarantees.len())
            .filter(|other| return self.Stands_Stronger(*other, against))
            .filter_map(|other| return Some((self.Earliest(other)?, other)))
            .min()
            .map(|(_, other)| return other);
    }

    fn Stands_Stronger(&self, class: usize, against: Guarantee) -> bool
    {
        return self
            .guarantees
            .get(class)
            .is_some_and(|offered| return Standing::Of(offered, &against) == Standing::Stronger);
    }

    /// Ranks `class`'s earliest unranked offer and returns its input position.
    fn Take_Earliest(&mut self, class: usize) -> Option<usize>
    {
        let earliest = self.Earliest(class)?;
        if let Some(ranked) = self.ranked.get_mut(class)
        {
            *ranked = ranked.saturating_add(1);
        }

        return Some(earliest);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{
        Assurance, CapabilityId, ContractVersion, FactVariant, IncrementalGranularity,
    };

    /// How many offers `Test_Ranking_Should_Return_Every_Offer_It_Was_Given` hands the
    /// ranking. The assertion it guards is that the count coming back is this one.
    const OFFERS_GIVEN: usize = 3;

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

        assert_eq!(ranked.len(), OFFERS_GIVEN, "an offer was dropped: {ranked:?}");
        assert_eq!(at("parse"), 0, "nothing is stronger than the parser here");
        assert!(at("scan") > at("parse"), "and the parser is strictly stronger than the scan");
    }

    #[test]
    fn Test_Over_Should_Account_For_Every_Usable_Offer()
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
    fn Test_Has_Passed_Over_Stronger_Should_Be_True_Once_A_Preference_Passed_Over_The_Ranked_Head()
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
    fn Test_Standing_Of_Should_Read_How_An_Alternative_Compares_To_The_Chosen_Offer()
    {
        let selection = Selection::Over(
            vec![Offer_With_Guarantee("a.scan", Scan()), Offer_With_Guarantee("z.parse", Parse())],
            None,
        )
        .expect("two usable offers");

        assert_eq!(
            selection.Standing_Of(&Offer_With_Guarantee("a.scan", Scan())),
            Standing::Weaker,
            "the parser answered, and the scan alternative reaches less than it does"
        );
    }

    #[test]
    fn Test_Weaker_Should_List_Only_The_Alternatives_Standing_Below_The_Chosen_Offer()
    {
        let selection = Selection::Over(
            vec![Offer_With_Guarantee("a.scan", Scan()), Offer_With_Guarantee("z.parse", Parse())],
            None,
        )
        .expect("two usable offers");

        let weaker = selection.Weaker();

        assert_eq!(weaker.len(), 1);
        assert_eq!(weaker.first().expect("the assertion above found exactly one weaker offer").provider, ProviderId::New("a.scan"));
    }

    #[test]
    fn Test_Unranked_Should_List_Alternatives_The_Guarantee_Could_Not_Compare()
    {
        let selection = Selection::Over(
            vec![Offer_With_Guarantee("scan", Scan()), Offer_With_Guarantee("semantic", Coarse_Semantic())],
            None,
        )
        .expect("two usable offers");

        // `scan` and `semantic` are incomparable (see the fixtures' own doc comment), so
        // whichever ranked first leaves the other unranked rather than decided.
        assert_eq!(selection.Unranked().len(), 1);
    }

    #[test]
    fn Test_An_Empty_Set_Should_Not_Produce_A_Selection()
    {
        assert!(Selection::Over(Vec::new(), None).is_none());
    }

    /// How many pseudo-random populations the linear ranking is held against a scan per
    /// offer, the largest of them this many offers.
    const POPULATIONS_COMPARED: usize = 40;

    /// The seed the populations are drawn from. Fixed, because a ranking defect that only
    /// one seed finds is a defect that only one run finds.
    const RANDOM_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

    /// A 64-bit linear congruential generator's constants (Knuth's MMIX). A generator
    /// written out here rather than a dependency: what these populations need is
    /// repeatability across runs, not statistical quality.
    const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;

    /// The increment of the same generator.
    const LCG_INCREMENT: u64 = 1_442_695_040_888_963_407;

    /// A scan per offer: for each place in the ranking, walk everything still unranked and
    /// take what the running-strongest chain settles on.
    ///
    /// This is what `Ranked_Offers` was before it was made linear, kept as the definition
    /// the linear form is held against. Deleting it would leave the faster form checked only
    /// against the handful of populations spelled out by hand above.
    fn Reference_Ranking(mut offers: Vec<ProviderOffer>) -> Vec<ProviderOffer>
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

    fn Next_Random(state: &mut u64) -> u64
    {
        *state = state.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);

        return *state;
    }

    /// One guarantee, each axis drawn independently across its whole enumeration — which is
    /// what makes these populations genuinely partially ordered rather than a chain with
    /// noise in it.
    fn Random_Guarantee(state: &mut u64) -> Guarantee
    {
        use FactVariant::{Approximate, Predicted, RuntimeObserved, SemanticallyResolved, Syntactic};
        use IncrementalGranularity::{File, None, Project, Region, Symbol, WholeWorkspace};

        let variants = [Predicted, Approximate, Syntactic, SemanticallyResolved, RuntimeObserved];
        let assurances = [Assurance::Sound, Assurance::Unsound, Assurance::Unknown];
        let granularities = [None, WholeWorkspace, Project, File, Symbol, Region];

        return Guarantee::New(
            Drawn(state, &variants),
            Drawn(state, &assurances),
            Drawn(state, &assurances),
            Drawn(state, &granularities),
        );
    }

    fn Drawn<Axis: Copy>(state: &mut u64, from: &[Axis]) -> Axis
    {
        let axes = u64::try_from(from.len()).expect("an axis enumeration has a handful of values");
        let at = Next_Random(state).checked_rem(axes).expect("every axis here is non-empty");
        let at = usize::try_from(at).expect("a remainder of the length fits the length's own type");

        return *from.get(at).expect("the index is taken modulo the slice's own length");
    }

    fn Random_Offers(state: &mut u64, population: usize) -> Vec<ProviderOffer>
    {
        return (0..population)
            .map(|provider| {
                return Offer_With_Guarantee(
                    &format!("nomos.provider.test.{provider:04}"),
                    Random_Guarantee(state),
                );
            })
            .collect();
    }

    /// How many pairs in `offers` the guarantee cannot rank against each other. The
    /// populations are worth nothing as evidence if this is zero.
    fn Incomparable_Pairs(offers: &[ProviderOffer]) -> usize
    {
        return offers
            .iter()
            .flat_map(|left| {
                return offers
                    .iter()
                    .map(|right| return Standing::Of(&left.guarantee, &right.guarantee));
            })
            .filter(|standing| return *standing == Standing::Incomparable)
            .count();
    }

    /// The property the ranking exists for, asserted over every population: nothing usable
    /// is strictly stronger than what the ranking put first.
    fn Assert_The_Head_Is_Maximal(ranked: &[ProviderOffer], population: usize)
    {
        let Some(head) = ranked.first()
        else
        {
            return;
        };

        for offer in ranked
        {
            assert_ne!(
                Standing::Of(&offer.guarantee, &head.guarantee),
                Standing::Stronger,
                "over {population} offers, {} was ranked first though {} is strictly stronger",
                head.provider.As_Str(),
                offer.provider.As_Str()
            );
        }
    }

    /// The linear ranking against a literal scan per offer, offer for offer and position for
    /// position, over populations the guarantee genuinely cannot totally order.
    #[test]
    fn Test_The_Fast_Ranking_Should_Agree_With_A_Scan_Per_Offer()
    {
        let mut state = RANDOM_SEED;
        let mut incomparable = 0_usize;

        for population in 1..=POPULATIONS_COMPARED
        {
            let offers = Random_Offers(&mut state, population);
            incomparable = incomparable.saturating_add(Incomparable_Pairs(&offers));
            let ranked = Ranked_Offers(offers.clone());

            Assert_The_Head_Is_Maximal(&ranked, population);
            assert_eq!(
                Named_Providers(&ranked),
                Named_Providers(&Reference_Ranking(offers)),
                "the linear ranking and a scan per offer disagreed over {population} offers"
            );
        }

        assert!(
            incomparable > 0,
            "every population drawn was totally ordered, so this compared nothing a partial \
             order does that a sort does not"
        );
    }

    /// The offer counts the two rankings are timed at. Four times the offers is sixteen
    /// times the work under a scan per offer and four times under the linear form, so two
    /// steps of four are enough to tell them apart without reading either number alone.
    const TIMED_POPULATIONS: [usize; 3] = [1_000, 4_000, 16_000];

    /// How many times each population is ranked, so one scheduling hiccup does not become
    /// the number reported. The best of them is taken: the thing being measured is the work
    /// the code does, and every deviation upward belongs to the machine.
    const TIMED_REPEATS: u32 = 9;

    /// The nanoseconds `rank` takes over `offers`, best of [`TIMED_REPEATS`].
    fn Nanoseconds_Of(rank: fn(Vec<ProviderOffer>) -> Vec<ProviderOffer>, offers: &[ProviderOffer]) -> u128
    {
        let mut best = u128::MAX;

        for _ in 0..TIMED_REPEATS
        {
            let population = offers.to_vec();
            let started = std::time::Instant::now();
            let ranked = rank(population);
            best = best.min(started.elapsed().as_nanos());

            assert_eq!(ranked.len(), offers.len(), "a ranking dropped an offer");
        }

        return best;
    }

    /// What the change to [`Ranked_Offers`] cost and bought, measured side by side in one
    /// process so the two numbers are taken on one machine under one load.
    ///
    /// `#[ignore]`d because a timing figure is evidence for a commit message and not a gate:
    /// a shared runner under load would fail a threshold that says nothing about the code.
    /// Run it with
    /// `cargo test --release -p nomos-capability --lib -- --ignored --nocapture ranking_cost`.
    #[test]
    #[ignore = "a timing measurement, reported into a commit message rather than gated on"]
    fn Test_Ranking_Cost_Should_Be_Reported_Beside_A_Scan_Per_Offer()
    {
        let mut state = RANDOM_SEED;

        for population in TIMED_POPULATIONS
        {
            let offers = Random_Offers(&mut state, population);
            let scanned = Nanoseconds_Of(Reference_Ranking, &offers);
            let linear = Nanoseconds_Of(Ranked_Offers, &offers);

            println!("ranking {population} offers: scan-per-offer {scanned} ns, linear {linear} ns");
        }
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
