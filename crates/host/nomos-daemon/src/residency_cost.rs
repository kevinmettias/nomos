//! What one request cost the store it was answered from.

/// What answering one request cost, in the only unit residency can change.
///
/// Not wall clock. A judgment launches `cargo metadata`, `cargo clippy` and `cargo deny`
/// whatever the resident already holds -- `nomos-check-orchestration`'s own currency module
/// states that those providers still run -- so a duration measures the subprocesses more than
/// it measures reuse, and it is not reproducible enough to assert on. What residency actually
/// removes is store writes and the rule invocations that follow them, and
/// [`nomos_analysis::MemoryFactStore::Materializations`]'s delta across one request is exactly
/// that number: it falls to zero over a tree where nothing moved, and to the cost of the moved
/// subjects alone over a tree where something did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResidencyCost
{
    /// Facts this request filed into the store.
    pub materializations: u32,
    /// Facts the store serves once this request is answered.
    pub live_facts: usize,
}
