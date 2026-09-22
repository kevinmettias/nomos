//! Everything outside a finding that a walk outward is answered from.

use nomos_cap_architecture::ArchitecturePayload;
use nomos_cap_requirement_trace::Assessment;
use nomos_check_orchestration::SupportingFactTrail;

/// The declarations and the run trail [`crate::WalkOutward`] reads, handed in together.
///
/// One parameter rather than one per answer. `OD-HOST-015` and `OD-HOST-016` each predicted
/// that their building item would add a parameter to [`crate::Diagnostics_For`], and
/// `OD-HOST-016`'s seventh decision resolved the collision the two predictions make: if both
/// land, they hand that function a context struct rather than a third parameter. So a sixth
/// walk-outward answer is a field here and not another change to a signature two records
/// already changed.
///
/// Borrowed rather than owned, and built once per `Diagnose` batch rather than once per
/// finding: the architecture is one file, the assessments are one directory, and the trail is
/// the one value the run already returned. Nothing here is kept past the batch.
pub struct WalkContext<'a>
{
    /// What the repository under check declares about which component each of its crates
    /// belongs to. A repository that declares nothing resolves nothing, rather than resolving
    /// against whatever nomos knows about itself.
    pub architecture: &'a ArchitecturePayload,
    /// Which facts each rule read in the call that produced these findings, exactly as that
    /// call carried it back on its own outcome. Never rebuilt here: `OD-HOST-016` resolved
    /// every answered read against the store inside the run, where the generation was
    /// unambiguous, precisely so no later reader needs a key or a store.
    pub trail: &'a SupportingFactTrail,
    /// The committed assessments of the repository under check, as
    /// `nomos_cap_requirement_trace::Assessments_In` read them. Empty for every repository that
    /// keeps no such registry, which is every repository but this one.
    pub assessments: &'a [Assessment],
}
