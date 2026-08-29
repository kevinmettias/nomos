//! Asking a provider for one file, and recording who answered.
//!
//! The registry decides which provider serves the floor; this decides what to do when the
//! chosen one refuses. A fallback answer is bought visibly — the census the run reports is
//! what makes "one file was approximated" a name rather than a number.

use super::{
    MaterializedFact, Recompute, RunReport, Slice, SourceFile, rust, scan, syntax,
};

/// Who answered for one file, and where in the offer order they came.
#[derive(Clone, Copy)]
pub(super) struct Answered<'a>
{
    provider: &'a str,
    rank: usize,
    file: &'a SourceFile,
}

impl Slice
{
    /// Asks whichever provider the registry resolved.
    ///
    /// # Why the composition root holds the table and not the registry
    ///
    /// A `ProviderOffer` is a claim, not a function pointer, and deliberately so:
    /// `nomos-capability` sits below every provider and must not know what any of them can
    /// be called. So somebody has to turn a resolved `ProviderId` into a call, and the only
    /// thing that legitimately knows every provider is the composition root — the same
    /// place that registered them.
    ///
    /// The `unreachable` arm is the cost of that split, and it is a real one: a provider
    /// registered and not dispatched would resolve and then fail to answer. It is written
    /// as a panic naming the provider rather than as a silent skip, because a run that
    /// quietly produced no facts for a provider it selected would report a clean corpus it
    /// never read.
    ///
    /// # Panics
    ///
    /// If the resolved provider has no dispatch here.
    pub(super) fn Dispatch(&self, file: &SourceFile, offer: &nomos_capability::ProviderOffer)
        -> rust::Materialization
    {
        let provider = offer.provider.As_Str();
        if provider == rust::PROVIDER
        {
            return self.Parsed(file);
        }
        if provider == scan::PROVIDER
        {
            return self.Scanned(file);
        }

        // rust-panic: allow: reached only when a provider was registered with the registry and
        // never given a call here. Those two lists are maintained in different places and this
        // line is the only thing that ever compares them, so a silent skip would produce no
        // facts for a provider the run had already selected and reported as its answer's
        // provenance.
        panic!("{provider} was resolved and this composition cannot call it");
    }

    /// The parser's answer, which may be a refusal.
    pub(super) fn Parsed(&self, file: &SourceFile) -> rust::Materialization
    {
        return rust::Materialize(
            file.subject,
            &file.source,
            rust::FactContext {
                snapshot: self.snapshot,
                variant: self.variant,
                configuration: self.configuration,
                generation: self.generation,
            },
        );
    }

    /// A scan has no refusal case, so this cannot produce `Unparseable`.
    ///
    /// That asymmetry is the interesting half of having two providers: they do not merely
    /// differ in how good the answer is, they differ in which inputs they can answer for at
    /// all.
    pub(super) fn Scanned(&self, file: &SourceFile) -> rust::Materialization
    {
        let materialized = scan::Materialize(
            file.subject,
            &file.source,
            scan::FactContext {
                snapshot: self.snapshot,
                variant: self.variant,
                configuration: self.configuration,
                generation: self.generation,
            },
        );

        return rust::Materialization::Materialized(Box::new(materialized));
    }

    /// Walks the selection for one file until something answers.
    ///
    /// The loop `P8-SELECTION` made possible and nothing wrote. The chosen offer is asked
    /// first; if it refuses the file, the next admitted offer is asked, and so on down the
    /// ranking the registry produced. The first answer is written under *its own* provider's
    /// key, so the store never holds a fact labelled with a provider that refused it.
    ///
    /// A refusal is not cached. The parser is asked again on the next pass over the same
    /// file and refuses again, which is cheap and is the honest reading — nothing has
    /// recorded that the parser cannot answer, only that it did not.
    ///
    /// # Panics
    ///
    /// If a fact would be written behind the generation it names, which is a contradiction
    /// in the run loop rather than a runtime condition.
    pub(super) fn Answer_For(
        &mut self,
        file: &SourceFile,
        candidates: &[nomos_capability::ProviderOffer],
        report: &mut RunReport,
    )
    {
        let mut refusal = None;
        for (rank, offer) in candidates.iter().enumerate()
        {
            let Some(failure) = self.Ask(file, offer, rank, report)
            else
            {
                return;
            };
            // The chosen provider's refusal is the one a caller can act on, so it is the
            // one kept when nobody below can answer either.
            refusal = refusal.or(Some(failure));
        }

        report.refused.push((
            file.path.clone(),
            refusal.unwrap_or_else(|| return "no admitted provider answered".to_owned()),
        ));
    }

    /// One offer asked for one file: nothing if it answered, its refusal if it did not.
    pub(super) fn Ask(
        &mut self,
        file: &SourceFile,
        offer: &nomos_capability::ProviderOffer,
        rank: usize,
        report: &mut RunReport,
    ) -> Option<String>
    {
        let key = self.Syntax_Key_Of(file, offer);
        let provider = offer.provider.As_Str().to_owned();
        let answered = Answered { provider: &provider, rank, file };
        if self.Held(&key)
        {
            report.syntax_reused = report.syntax_reused.saturating_add(1);
            Self::Credit(report, answered);

            return None;
        }

        return match self.Dispatch(file, offer)
        {
            rust::Materialization::Materialized(fact) =>
            {
                self.Kept(*fact, report, answered);

                None
            }
            rust::Materialization::Unparseable(failure) => Some(failure.to_string()),
        };
    }

    /// Records who answered for a subject, and whether that was the chosen offer.
    /// One provider's answer, stored and counted.
    ///
    /// A leaf: computed from the file and from nothing else, so it declares no
    /// dependencies. This is also why the corpus needs the rollup — a graph of leaves has
    /// no descendants to get wrong.
    pub(super) fn Kept(&mut self, fact: MaterializedFact, report: &mut RunReport, answered: Answered<'_>)
    {
        self.store
            .Materialize(fact, &[])
            .expect("a fact is never written behind the generation it names");

        report.syntax_materialized = report.syntax_materialized.saturating_add(1);
        report.recomputed.push(Recompute {
            capability: syntax::CAPABILITY.to_owned(),
            subject: answered.file.path.clone(),
        });
        Self::Credit(report, answered);
    }

    pub(super) fn Credit(report: &mut RunReport, answered: Answered<'_>)
    {
        let Answered { provider, rank, file } = answered;
        let counted = report.answered_by.entry(provider.to_owned()).or_insert(0);
        *counted = counted.saturating_add(1);

        if rank > 0
        {
            report.fell_back.push((file.path.clone(), provider.to_owned()));
        }
    }
}
