//! What a resident was holding when a client asked it to stop.

/// What ended when the residency did.
///
/// Every number here is about the resident's own life and the memory it released. There is no
/// field naming a file to clean up, and that is the report rather than an omission: a resident
/// writes nothing to disk at any point, so a stopped one leaves nothing a later cold
/// invocation could read as current state. Persisting a store across processes is
/// `OD-ANALYSIS-009`'s question and its on-disk half is that record's; a resident that wrote
/// one on the way out would be leaving behind exactly the artifact a cold `nomos check` would
/// have to decide whether to believe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopReport
{
    /// Requests this resident answered before it was asked to stop.
    pub answered: usize,
    /// Facts it was serving when it stopped, and released.
    pub released_facts: usize,
}
