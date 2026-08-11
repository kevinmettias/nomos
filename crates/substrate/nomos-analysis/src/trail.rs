//! Every fact a read depended on, in the order it was read.

use crate::dependency::Dependency;
use crate::fact_key::FactKey;
use crate::reader::ReadOutcome;
use nomos_contracts::Applicability;

/// What one reader has read, kept apart from what it reads *through*.
///
/// A [`Reader`](crate::Reader) holds a store, a registry and a build; none of those change
/// while it reads. This is the half that does, and it is the half the invalidation engine
/// consumes afterwards. Separating them is what stops "which offers were admitted" and
/// "what did we end up asking for" from being one thing with two lifetimes.
///
/// A miss is recorded as readily as a hit. "The parser had nothing here" is a real read and
/// a real dependency, and a fact that must be invalidated once the parser *does* have
/// something needs that edge to exist.
#[derive(Default)]
pub(crate) struct Trail
{
    recorded: Vec<Dependency>,
}

impl Trail
{
    /// An empty trail, before anything has been read.
    #[must_use]
    pub(crate) const fn New() -> Self
    {
        return Self {
            recorded: Vec::new(),
        };
    }

    /// Records one read.
    pub(crate) fn Note(&mut self, key: &FactKey, outcome: ReadOutcome)
    {
        self.recorded.push(Dependency {
            key: key.clone(),
            outcome,
        });
    }

    /// Records a miss at the applicability every unanswered candidate is recorded at.
    pub(crate) fn Note_Miss(&mut self, key: &FactKey)
    {
        self.Note(
            key,
            ReadOutcome::Degraded(Applicability::DependencyUnavailable),
        );
    }

    /// What has been read so far.
    #[must_use]
    pub(crate) fn Recorded(&self) -> &[Dependency]
    {
        return &self.recorded;
    }

    /// The trail, once the reader that produced it is finished with.
    #[must_use]
    pub(crate) fn Into_Dependencies(self) -> Vec<Dependency>
    {
        return self.recorded;
    }
}
