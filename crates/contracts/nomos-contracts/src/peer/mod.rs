//! Talking to another system without letting its silence read as agreement.
//!
//! The two types here answer the two ways a remote system fails to agree with us.
//! [`PeerAvailability`] is about one question and one answer: did the peer speak at all?
//! [`SynchronizationState`] is about two records that both spoke and said different
//! things. A peer that never answered has not diverged from us, and two records that
//! disagree are not an outage — collapsing the two would let a network failure resolve
//! itself as a decision about content.

mod peer_availability;
mod synchronization_state;

pub use peer_availability::PeerAvailability;
pub use synchronization_state::SynchronizationState;
