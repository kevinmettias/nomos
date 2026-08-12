//! Summing a group's files into one surface fact, and saying what the sum read.
//!
//! A rollup that could not read a member is a degraded answer, and reporting it as an answer
//! is how a corpus with a hole in it reads as a corpus that is fine.

use super::*;

/// What a rollup produced: the summed surface, and the reads it was derived from.
///
/// Named rather than a pair. The dependency edges are a record of what was read, not a
/// second description of the surface, and a name is what keeps the two apart at the call
/// site.
pub(super) struct RolledUp
{
    surface: Surface,
    dependencies: Vec<Dependency>,
}

impl Slice
{
    /// One group's rollup: reused where the store already holds it, computed where it does
    /// not.
    pub(super) fn Surface_For(&mut self, corpus: &Corpus, group: String, report: &mut RunReport)
    {
        let members = corpus.In_Group(&group);
        let Some(subject) = members.first().map(|member| return member.group_subject)
        else
        {
            return;
        };
        let key = self.Surface_Key(subject, &members);
        if self.Held(&key)
        {
            report.surface_reused = report.surface_reused.saturating_add(1);

            return;
        }
        let rolled = self.Roll_Up(&members);

        Self::Note_The_Rollups_Reach(&rolled.surface, &group, report);
        self.Write_The_Surface(&key, &rolled);
        Self::Note_The_Surface_Was_Written(group, report);
    }

    /// What the rollup could not reach, and what it reached only approximately.
    pub(super) fn Note_The_Rollups_Reach(surface: &Surface, group: &str, report: &mut RunReport)
    {
        if surface.unreachable > 0
        {
            report.degraded.push(group.to_owned());
        }
        if surface.approximate > 0
        {
            report.approximated.push(group.to_owned());
        }
    }

    /// The rollup as a fact, stamped with the tree it was computed over.
    pub(super) fn Write_The_Surface(&mut self, key: &FactKey, rolled: &RolledUp)
    {
        self.store
            .Materialize(
                MaterializedFact {
                    identity: key.clone().At(self.generation),
                    // Provenance: the tree this rollup was computed over. Not part of the
                    // key, so the workspace moving re-addresses nothing.
                    snapshot: self.snapshot,
                    // No stronger than what it derived from. A count of verified facts is
                    // derived, and promoting it to Verified would launder the rollup's own
                    // arithmetic into a measurement.
                    evidence: EvidenceClass::Derived,
                    guarantee: surface::Declared_Guarantee(),
                    payload: FactPayload::New(surface::Payload_Schema(), rolled.surface.Encode()),
                },
                &rolled.dependencies,
            )
            .expect("a fact is never written behind the generation it names");
    }

    /// The group's rollup landed, which is one recomputation.
    pub(super) fn Note_The_Surface_Was_Written(group: String, report: &mut RunReport)
    {
        report.surface_materialized = report.surface_materialized.saturating_add(1);
        report.recomputed.push(Recompute {
            capability: surface::CAPABILITY.to_owned(),
            subject: group,
        });
    }

    /// Reads every member's syntax fact through the registry and sums what they declare.
    ///
    /// The dependency edges come from [`Reader`] observing the reads, not from this
    /// function listing them. That distinction is the point: a hand-written edge list is a
    /// claim about what was read, and this is a record of it.
    pub(super) fn Roll_Up(&self, members: &[&SourceFile]) -> RolledUp
    {
        // The run's own requirement, not a second one written here.
        //
        // It was a separate literal until there were two providers, and that was a latent
        // defect rather than a duplication: the rollup looks facts up by rebuilding their
        // key, and a key names the provider that answered. A rollup asking for a floor the
        // run did not ask for would resolve a different provider, rebuild a key nobody
        // wrote, and report every member unreachable — loudly, but for the wrong reason.
        let need = self.Requirement();
        let mut reader = Reader::On(&self.store, &self.registry, self.Context());
        let mut surface = Surface::default();
        for member in members
        {
            Self::Fold_In(&mut surface, &mut reader, member, &need);
        }

        return RolledUp {
            surface,
            dependencies: reader.Into_Dependencies(),
        };
    }

    /// One member read through the registry and folded into the running surface.
    ///
    /// `Require_Any` rather than `Require`, which is what makes the coverage a lowered floor
    /// bought reachable by the thing that derives from it. Reading only the chosen provider
    /// would leave the scanner's answer written into the store and unread, and the rollup
    /// would still report the member missing — the same defect one level in.
    pub(super) fn Fold_In(
        surface: &mut Surface,
        reader: &mut Reader<'_, '_>,
        member: &SourceFile,
        need: &Requirement,
    )
    {
        let read = reader.Require_Any(
            &CapabilityId::New(syntax::CAPABILITY),
            &member.subject,
            Self::Syntax_Inputs(&member.source),
            need,
        );
        let Ok((fact, applicability)) = read
        else
        {
            // The member has no readable fact — every admitted provider refused it, or
            // nothing has computed it. Counted, never skipped: a rollup that silently
            // omits a member reports a smaller surface as if it were a complete one.
            surface.unreachable = surface.unreachable.saturating_add(1);

            return;
        };

        match crate::surface::Public_Items(&fact.payload.bytes)
        {
            Ok((items, public)) => Self::Summed(surface, items, public, applicability),
            Err(_) => surface.unreachable = surface.unreachable.saturating_add(1),
        }
    }

    /// One readable member folded into the rollup.
    ///
    /// The [`Applicability`] is taken whole rather than reduced to a flag by the caller,
    /// so the one place that acts on it is the one place that reads it. It is counted here
    /// at all because of `OD-CAPABILITY-003`'s third condition: without it the scanner's
    /// answer covers the file the parser refused, `unreachable` drops to zero, and a corpus
    /// that was visibly incomplete starts reading as complete and sound.
    pub(super) fn Summed(surface: &mut Surface, items: u32, public: u32, applicability: Applicability)
    {
        surface.files = surface.files.saturating_add(1);
        surface.items = surface.items.saturating_add(items);
        surface.public = surface.public.saturating_add(public);

        if applicability == Applicability::SupportedWithFallback
        {
            surface.approximate = surface.approximate.saturating_add(1);
        }
    }
}
