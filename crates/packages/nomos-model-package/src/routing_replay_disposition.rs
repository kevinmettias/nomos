//! Whether a routing decision, or a recorded execution's output, can be replayed.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-017`: "Every `ResolvedModelExecution` and replay result shall include
/// a `RoutingReplayDisposition`. Aliases such as latest, executor-controlled model
/// selection, unavailable exact revisions, provider-side nondeterminism, missing
/// native controls, or incomplete context/tool artifacts shall prevent an `Exact`
/// classification and shall be reported as `ConfigurationEquivalent`, `BestEffort`, or
/// `NonReplayable` with reasons."
///
/// Four states, the fourth (`Exact`) implicit as what the named causes *prevent*; the
/// other three each carry `reasons`, per the sentence's own "with reasons" clause.
/// `MODEL-ROUTE-032` confirms this as a real, separately-named sibling of
/// `OutputDeterminismExpectation` ("independently from `RoutingReplayDisposition`")
/// and adds [`ReplayFacts`]' two-value separation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingReplayDisposition
{
    Exact,
    ConfigurationEquivalent
    {
        reasons: Vec<String>,
    },
    BestEffort
    {
        reasons: Vec<String>,
    },
    NonReplayable
    {
        reasons: Vec<String>,
    },
}

/// `MODEL-ROUTE-032`: "Clients, reports, attestations, and replay tooling shall not
/// infer identical output from `Exact` or `ConfigurationEquivalent` routing replay. A
/// recorded executor may replay a retained prior output exactly while the originating
/// live model operation remains `BestEffort` or `NonReplayable`; both facts shall be
/// represented separately."
///
/// Two independently-tracked dispositions, exactly as the sentence's "both facts shall
/// be represented separately" states -- a recorded executor's own ability to reproduce
/// a retained output, and the disposition of the live model operation that originally
/// produced it, kept apart so neither is mistaken for the other.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFacts
{
    pub executor_replay: RoutingReplayDisposition,
    pub live_operation_disposition: RoutingReplayDisposition,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Non_Exact_Disposition_Carries_Its_Reasons()
    {
        let disposition = RoutingReplayDisposition::BestEffort {
            reasons: vec!["provider-side nondeterminism".to_owned()],
        };

        match disposition
        {
            RoutingReplayDisposition::BestEffort { reasons } => assert_eq!(reasons, ["provider-side nondeterminism"]),
            _ => panic!("expected BestEffort"),
        }
    }

    #[test]
    fn Test_Replay_Facts_Track_Executor_And_Live_Operation_Separately()
    {
        let facts = ReplayFacts {
            executor_replay: RoutingReplayDisposition::Exact,
            live_operation_disposition: RoutingReplayDisposition::NonReplayable {
                reasons: vec!["unavailable exact revision".to_owned()],
            },
        };

        assert_eq!(facts.executor_replay, RoutingReplayDisposition::Exact);
        assert_ne!(facts.executor_replay, facts.live_operation_disposition);
    }
}
