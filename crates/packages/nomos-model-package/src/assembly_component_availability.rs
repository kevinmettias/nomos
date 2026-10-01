//! Why one component of a [`crate::ModelInputAssemblyIdentity`] could not be pinned
//! exactly.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-030`: "Replay classification shall be computed over the complete
/// `ModelInputAssemblyIdentity` as well as backend, model revision, native controls,
/// tools, policy, and environment. A change to any semantically relevant assembly
/// component shall prevent `Exact` replay unless the component is proven
/// byte-for-byte or contract-equivalent under an explicitly versioned equivalence
/// rule. Missing, redacted, provider-hidden, or executor-controlled assembly
/// components shall lower the disposition accordingly."
///
/// Four named states, directly tied to the "shall lower the disposition accordingly"
/// clause -- attaches per-component to a [`crate::ModelInputAssemblyIdentity`], for
/// whichever of that type's twelve pinned fields could not be captured exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssemblyComponentAvailability
{
    Missing,
    Redacted,
    ProviderHidden,
    ExecutorControlled,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_The_Four_States_Are_Distinct()
    {
        let all = [
            AssemblyComponentAvailability::Missing,
            AssemblyComponentAvailability::Redacted,
            AssemblyComponentAvailability::ProviderHidden,
            AssemblyComponentAvailability::ExecutorControlled,
        ];

        for (index, state) in all.iter().enumerate()
        {
            for (other_index, other) in all.iter().enumerate()
            {
                if index != other_index
                {
                    assert_ne!(state, other);
                }
            }
        }
    }
}
