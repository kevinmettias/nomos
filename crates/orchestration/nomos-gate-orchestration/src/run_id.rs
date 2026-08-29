//! Constructing a fresh [`RunId`] -- `OD-WORKFLOW-001`'s first real increment for the
//! workflow tier, amended.
//!
//! `RunId` (`crates/contracts/nomos-contracts/src/identity.rs`) cannot construct one of
//! itself: band 0 may depend on nothing but `serde`, so it can neither read a
//! `nomos_platform::Timestamp` nor call `nomos_model::Digest_Of_Parts`. `nomos-model` (band
//! 10) cannot either -- `nomos-platform` is band 15, and a lower band may not depend on a
//! higher one. This crate is the lowest band that can reach both, so this is where a fresh
//! identity is built, exported for `Run_Gate`'s callers to construct one with rather than
//! leaving each composition root to invent its own.

use nomos_contracts::RunId;
use nomos_model::Digest_Of_Parts;
use nomos_platform::Timestamp;
use std::sync::atomic::{AtomicU64, Ordering};

/// A process-local sequence, so two calls within the same wall-clock second still diverge.
static NEXT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// How many byte slices [`Fresh_Run_Id`] digests together: the wall-clock seconds, the
/// process id, and the process-local sequence counter.
const DIGEST_PART_COUNT: usize = 3;

/// A `RunId` for an execution starting now.
///
/// Not collision-proof, and this does not pretend otherwise. It combines `now`, this
/// process's id and a process-local monotonic counter: collision-free within one process
/// (the counter alone guarantees that), and only unlikely -- not ruled out -- across two
/// processes that start within the same second and happen to be handed the same
/// operating-system process id by reuse. A real randomness source would close that gap, but
/// none exists in this workspace yet: `nomos-platform` has no `Random` port, and the
/// workspace carries no `rand` or `uuid` dependency, which is its own decision (subject to
/// `cargo-deny`'s advisories/bans/licenses/sources check) rather than one this function
/// makes silently. Naming the limitation plainly is the same honesty this workspace already
/// holds `Applicability::PartiallySupported` to.
#[must_use]
pub fn Fresh_Run_Id(now: Timestamp) -> RunId
{
    // atomic-ordering: allow: only used to give two calls in this process different numbers
    // (the doc comment above already covers the uniqueness this buys); nothing else
    // synchronizes on it or reads memory ordered by this counter.
    let sequence = NEXT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let seconds = now.Unix_Seconds().to_be_bytes();
    let process = std::process::id().to_be_bytes();
    let counter = sequence.to_be_bytes();

    let parts: [&[u8]; DIGEST_PART_COUNT] = [&seconds, &process, &counter];

    return RunId::From_Digest(Digest_Of_Parts(&parts));
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The one guarantee this function actually makes: two calls never collide, regardless
    /// of how close together they land in wall-clock time.
    #[test]
    fn Test_Two_Fresh_Ids_Should_Never_Collide()
    {
        let now = Timestamp::From_Unix_Seconds(0);

        let first = Fresh_Run_Id(now);
        let second = Fresh_Run_Id(now);

        assert_ne!(first, second);
    }

    #[test]
    fn Test_A_Later_Second_Should_Change_The_Id()
    {
        let earlier = Fresh_Run_Id(Timestamp::From_Unix_Seconds(1_000));
        let later = Fresh_Run_Id(Timestamp::From_Unix_Seconds(2_000));

        assert_ne!(earlier, later);
    }
}
