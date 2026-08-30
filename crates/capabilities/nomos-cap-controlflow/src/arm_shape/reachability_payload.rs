//! One file's full set of flagged reachability sites.

use super::reachability_site::ReachabilitySite;

/// One file's full set of flagged reachability sites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachabilityPayload
{
    /// The flagged sites this payload carries.
    pub sites: Vec<ReachabilitySite>,
}
