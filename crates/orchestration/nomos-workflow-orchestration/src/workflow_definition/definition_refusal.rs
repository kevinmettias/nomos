//! Why a workflow definition could not be published.

/// Why a [`super::WorkflowDefinition`] could not be published.
///
/// Every one of these is refused at [`super::WorkflowDefinition::Publish`], which is
/// before a definition exists at all and therefore before any body could dispatch --
/// the same guarantee `WorkflowStep::Is_Coherent` already gives the sequential
/// [`crate::Run`], moved one step earlier. A definition that exists is a definition whose
/// topology and whose every step declaration were already checked, so
/// [`crate::WorkflowOutcome::Refused`] is unreachable from a definition run: there is no
/// way to hand the runner one that was never checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefinitionRefusal
{
    /// Two nodes are named the same, so a reference to that name names neither.
    DuplicateNodeName
    {
        /// The spelling two nodes share.
        name: String,
    },
    /// A branch chooses on a value no earlier node produces, so nothing could ever decide
    /// which arm runs.
    ///
    /// The refusal the whole publish-time check exists for. A branch condition is a read
    /// of what earlier steps produced, and a read of a name nothing writes is a defect in
    /// the definition rather than a run that happens to take the default arm -- which is
    /// exactly the shape that would otherwise dispatch half a workflow before discovering
    /// it could not decide the other half.
    BranchesOnUnproducedValue
    {
        /// The branch node that cannot decide.
        branch: String,
        /// The value it reads, which no earlier node publishes.
        value: String,
    },
    /// A step declares a dependence on a value no earlier node produces.
    RequiresUnproducedValue
    {
        /// The step node with the unsatisfiable dependence.
        node: String,
        /// The value it depends on, which no earlier node publishes.
        value: String,
    },
    /// A branch arm names a node that is not declared after it, or a join names a node
    /// that is not declared before it.
    ///
    /// One refusal for both directions because it is one defect: a reference that does not
    /// resolve in the direction its node reads. An arm reaching backward would name a node
    /// that already ran; a join reaching forward would name one that has not.
    NamesUnknownNode
    {
        /// The node holding the reference.
        node: String,
        /// The name it reaches for.
        named: String,
    },
    /// A step's own `WorkflowStep::Is_Coherent` refuses its declaration.
    ///
    /// Checked here rather than left to the run, so an incoherent declaration anywhere in
    /// a definition is refused before the *first* body dispatches rather than after the
    /// steps ahead of it already have. The sequential [`crate::Run`] keeps its own
    /// per-step behaviour unchanged; this is a stricter promise a definition can make
    /// because it has the whole plan before anything runs.
    IncoherentStep
    {
        /// The step node whose declaration is incoherent.
        node: String,
    },
}
