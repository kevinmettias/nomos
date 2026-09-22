//! What a node published when it settled, reduced to the one distinction a branch acts on.

/// What a node published when it settled, reduced to the one distinction a branch can act
/// on: whether its own dispatch target reported something to act on.
///
/// Two states rather than the four dispatch outcomes, because a branch is a choice and a
/// choice needs a decidable value. This is deliberately *not* a second judgment about a
/// run: every arm below defers to the vocabulary the answering seam already published, and
/// none of them invents a threshold. Where a seam's own vocabulary has no clean answer --
/// a check that never reached `Judged`, a correction that could not read the tree -- the
/// state is [`Self::Flagged`], because an absence of judgment is not a statement that all
/// is well and collapsing the two is the failure `nomos_contracts`' honesty vocabularies
/// exist to prevent.
///
/// A step that *failed* to dispatch produces neither state: a dispatch failure ends the
/// run through [`crate::WorkflowOutcome::Failed`] and no branch downstream of it is ever
/// reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducedState
{
    /// The node's own dispatch target reported nothing to act on.
    Clean,
    /// The node's own dispatch target reported something to act on, or did not reach a
    /// judgment at all.
    Flagged,
}
