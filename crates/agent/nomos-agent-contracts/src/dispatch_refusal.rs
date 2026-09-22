//! Why a port produced no answer at all.

/// Why an [`crate::AgentExecutor`] or a [`crate::ModelBackend`] produced nothing.
///
/// One shape for both ports, carrying the text the refusing adapter's own `Display` already
/// produces. That is not a flattening of the two outcome shapes -- those stay apart, which
/// is the whole of why there are two ports -- it is the fold the generic path already made:
/// `AgentDispatchOutcome::Unavailable(String)` collapsed both adapters' error types to text
/// before this type existed, because a caller that reached it already knows which backend it
/// asked for.
///
/// Each adapter keeps its own structured error type on its own public surface, so a caller
/// that wants the structure calls the adapter directly rather than through the port. The
/// port is for the generic path, which has nothing to do with the difference between "the
/// process could not start" and "its stdout was not the JSON it promised".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchRefusal
{
    reason: String,
}

impl DispatchRefusal
{
    /// A refusal carrying `reason` as the refusing adapter stated it.
    #[must_use]
    pub fn Of(reason: impl Into<String>) -> Self
    {
        return Self { reason: reason.into() };
    }

    /// The reason, as the refusing adapter stated it.
    #[must_use]
    pub fn Reason(&self) -> &str
    {
        return &self.reason;
    }
}

impl core::fmt::Display for DispatchRefusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}
