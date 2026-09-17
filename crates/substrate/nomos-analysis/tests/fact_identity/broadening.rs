//! A provider that cannot refresh at the grain the cause names, and one that can.

use crate::key::{Base, Coarse, Materialized_Fact_From_Key, Memory_Store_For_Key};
use nomos_analysis::{FactStore, GenerationCause, MemoryFactStore};
use nomos_contracts::{GenerationId, IncrementalGranularity};

#[test]
fn Test_A_Coarse_Provider_Should_Have_Its_Broadening_Reported()
{
    let key = Base();
    let mut store = MemoryFactStore::New();
    let mut fact = Materialized_Fact_From_Key(&key, GenerationId::INITIAL);
    fact.guarantee = Coarse();
    store.Materialize(fact, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: IncrementalGranularity::Symbol,
        },
        next,
    );

    assert_eq!(report.broadened.len(), 1, "{report:?}");
    assert_eq!(
        report.broadened.first().map(|broadening| broadening.applied),
        Some(IncrementalGranularity::WholeWorkspace),
        "a symbol-level cause was reported as if the provider could refresh a symbol"
    );
}

#[test]
fn Test_A_Provider_At_The_Requested_Granularity_Should_Not_Report_Broadening()
{
    let key = Base();
    let mut store = Memory_Store_For_Key(&key);

    let report = store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: IncrementalGranularity::File,
        },
        GenerationId::INITIAL.Next(),
    );

    assert!(report.broadened.is_empty(), "{report:?}");
}
