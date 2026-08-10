//! Asking the store a question and being told how well it was answered.

use nomos_contracts::Applicability;
use nomos_capability::Requirement;
use nomos_contracts::SubjectId;
use nomos_contracts::CapabilityId;
use crate::dependency::Dependency;
use crate::identity::InputDigest;
use crate::fact_error::FactError;
use crate::materialized_fact::MaterializedFact;
use crate::fact_identity::FactIdentity;
pub trait FactReader
{
    fn Get(&mut self, identity: &FactIdentity) -> Result<&MaterializedFact, FactError>;

    fn Require(
        &mut self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        need: &Requirement,
    ) -> Result<&MaterializedFact, Applicability>;

    /// The best answer any admitted provider has for this subject, and how good it is.
    ///
    /// [`FactReader::Require`] asks the chosen provider and stops. That is right when a
    /// caller wants one provider's answer or none, and it is what leaves a lowered floor
    /// unspent: the offers the floor admitted are reachable and nothing looks at them.
    ///
    /// This walks the selection — the chosen offer, then `Selection::Weaker()` in the order
    /// the registry ranked them — and takes the first that answers. The order is the
    /// registry's rather than this reader's, which is what makes the result single-valued:
    /// one ordered list, one first hit. `OD-CAPABILITY-003` records why that is the rule.
    ///
    /// The returned [`Applicability`] is [`Applicability::SupportedWithFallback`] when the
    /// answer came from anything but the chosen offer. A caller that ignores it is deriving
    /// a fact from an approximation and calling it exact, which is the whole risk of
    /// falling back at all.
    ///
    /// # Errors
    ///
    /// [`Applicability::DependencyUnavailable`] when no admitted provider has an answer,
    /// and whatever the resolution said when nothing was admitted in the first place.
    fn Require_Any(
        &mut self,
        capability: &CapabilityId,
        subject: &SubjectId,
        inputs: InputDigest,
        need: &Requirement,
    ) -> Result<(&MaterializedFact, Applicability), Applicability>;

    fn Dependencies(&self) -> &[Dependency];
}
