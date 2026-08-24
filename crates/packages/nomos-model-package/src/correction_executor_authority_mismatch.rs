//! Whether a correction route landed on read-only ground on purpose.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-025`: "Validation shall reject or diagnose an operation-stage and
/// executor-authority mismatch, including a `CorrectionOrFix` profile that resolves
/// only to a read-only, review-only, non-mutating, or non-transactional executor when
/// mutation is required. The diagnostic shall distinguish an intentionally
/// proposal-only correction from an accidentally non-executable correction route."
///
/// A marker wrapping the corpus's own two-way classification -- "the diagnostic shall
/// distinguish A from B" is an exhaustive binary, equivalent in force to a "shall be
/// X or Y" enumeration even without that exact phrasing. The four-adjective list
/// (read-only, review-only, non-mutating, non-transactional) is illustrative context
/// for when the mismatch fires, not itself enumerated as a field the way the
/// classification sentence explicitly is, so it stays out of this shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectionExecutorAuthorityMismatch
{
    pub classification: CorrectionRouteClassification,
}

/// `MODEL-ROUTE-025`'s own two named conditions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrectionRouteClassification
{
    /// "an intentionally proposal-only correction"
    IntentionallyProposalOnly,
    /// "an accidentally non-executable correction route"
    AccidentallyNonExecutable,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Mismatch_Names_Its_Classification()
    {
        let mismatch = CorrectionExecutorAuthorityMismatch {
            classification: CorrectionRouteClassification::AccidentallyNonExecutable,
        };

        assert_eq!(mismatch.classification, CorrectionRouteClassification::AccidentallyNonExecutable);
    }

    #[test]
    fn Test_The_Two_Classifications_Are_Distinct()
    {
        assert_ne!(
            CorrectionRouteClassification::IntentionallyProposalOnly,
            CorrectionRouteClassification::AccidentallyNonExecutable
        );
    }
}
