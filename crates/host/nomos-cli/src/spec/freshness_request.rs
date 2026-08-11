//! What `nomos spec freshness` was asked for.

use std::path::PathBuf;
/// Which outputs are being checked, and which of them were promised.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FreshnessRequest
{
    /// The build root the profiles' own relative outputs are read from.
    pub into: PathBuf,
    /// Only this profile. Without it, every shipped profile is looked for.
    pub profile: Option<String>,
    /// The profiles this run requires to be there, whose absence is a failure.
    ///
    /// Empty by default, which is the question this command already answered: what is
    /// here, and is what is here current. A build root legitimately holds a subset, so
    /// absence is only a finding when a caller says which outputs it was promised — and
    /// that promise belongs to the repository asking, not to the profile, which describes
    /// how a projection is built and not whether anyone ships it.
    pub require: Vec<String>,
}
