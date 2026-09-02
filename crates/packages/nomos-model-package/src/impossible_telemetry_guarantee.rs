//! What happens when a profile promises a telemetry or replay guarantee its backend
//! cannot actually provide.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-027`: "A profile whose required telemetry completeness, usage
/// reporting, cost evidence, cancellation, structured-result capture, tool-evidence
/// attachment, exact model revision, native-control visibility, or replay guarantee
/// cannot be provided by the selected backend or executor shall produce an
/// `ImpossibleTelemetryGuarantee` or `ImpossibleReplayRequirement` diagnostic."
///
/// Eight named guarantee dimensions, in the corpus's own order -- the "replay
/// guarantee" ninth item maps to [`ImpossibleReplayRequirement`] instead, per the
/// corpus's own "or `ImpossibleReplayRequirement`" split between the two diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TelemetryGuaranteeKind
{
    Completeness,
    UsageReporting,
    CostEvidence,
    Cancellation,
    StructuredResultCapture,
    ToolEvidenceAttachment,
    ExactModelRevision,
    NativeControlVisibility,
}

/// One of `MODEL-ROUTE-027`'s two named diagnostics -- a telemetry-dimension
/// guarantee the selected backend or executor cannot provide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpossibleTelemetryGuarantee
{
    pub guarantee: TelemetryGuaranteeKind,
}

/// The other of `MODEL-ROUTE-027`'s two named diagnostics -- the "replay guarantee"
/// item, which the corpus's own "or" splits into its own diagnostic rather than a
/// ninth [`TelemetryGuaranteeKind`] variant. No further fields are named for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpossibleReplayRequirement;

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_An_Impossible_Telemetry_Guarantee_Names_Its_Dimension()
    {
        let impossible = ImpossibleTelemetryGuarantee {
            guarantee: TelemetryGuaranteeKind::ExactModelRevision,
        };

        assert_eq!(impossible.guarantee, TelemetryGuaranteeKind::ExactModelRevision);
    }

    #[test]
    fn Test_The_Replay_Requirement_Is_A_Distinct_Marker()
    {
        assert_eq!(ImpossibleReplayRequirement, ImpossibleReplayRequirement);
    }
}
