//! The transition kind that identity comparison produces.
//!
//! # Nothing in this workspace produces or reads one yet, and it is kept anyway
//!
//! Measured 2026-09-12: no crate outside `nomos-model` names this type, and the only
//! `Confidence::Of` calls anywhere are in `confidence.rs`'s own tests. A complete,
//! documented, tested chain connected to nothing is this repository's most common defect,
//! so keeping it needs a reason rather than inertia.
//!
//! The reason is that the corpus already names its consumer. `SUP-*`'s revalidation
//! triggers — quoted in `OD-GATE-015` — are "a rule upgrade, **an identity transition**,
//! moved code, an expired date, changed evidence, or a removed finding". Suppression and
//! baseline machinery is under active construction in `nomos-gate-orchestration`, and when
//! revalidation lands it needs exactly this: a typed answer to whether the thing a
//! suppression names is still the same thing. Deleting the chain would mean re-deriving it
//! against that requirement later, from the same corpus sentence.
//!
//! What that consumer does *not* settle is `Confidence`'s representation — see its own doc.

use crate::IdentityTransitionKind;

/// A change of identity between two snapshots.
pub type Transition = crate::Transition<IdentityTransitionKind>;
