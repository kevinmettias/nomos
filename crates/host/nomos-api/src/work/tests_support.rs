//! Scratch-board fixtures shared by more than one `Handle_Work_*` test module. Declared
//! `#[cfg(test)]` by `crate::work`'s own `mod tests_support;`, so nothing here compiles into
//! a real build.

use nomos_ledger::ItemId;

/// A scratch directory of this test's own -- never the real shared `work/` directory, which
/// live sessions write to concurrently. The same `std::env::temp_dir()` / `std::process::id()`
/// scoping `crates/host/nomos-cli/tests/scratch_ledger` already uses, plus a call-local
/// counter: several fixtures below share one `label` (`Scratch_Board` backs `List`,
/// `Validate` and `Audit` tests across three different files, for instance), and the default
/// test runner's threads would otherwise race on one directory a bare pid gave them.
/// `process::id()` alone tells two runs of the whole suite apart; it says nothing about two
/// calls inside one.
pub(crate) fn Unique_Scratch_Directory(label: &str) -> std::path::PathBuf
{
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let directory = std::env::temp_dir().join(format!("nomos-api-work-{label}-{}-{unique}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("creates a scratch directory");

    return directory;
}

/// A scratch board with no items, for the `List`, `Validate` and `Audit` tests that need
/// only a well-formed, empty ledger -- the same minimal two-key shape (`schema_version`,
/// `items`) `work/ledger.json` itself has.
pub(crate) fn Scratch_Board() -> std::path::PathBuf
{
    let directory = Unique_Scratch_Directory("list");
    std::fs::write(directory.join("ledger.json"), "{\"schema_version\": 5, \"items\": []}\n")
        .expect("writes a minimal valid ledger");

    return directory;
}

/// A scratch board carrying one real item, for the `Show` tests -- `Scratch_Board`'s own
/// empty board proves nothing about a found item.
pub(crate) fn Scratch_Board_With_One_Item() -> (std::path::PathBuf, ItemId)
{
    let directory = Unique_Scratch_Directory("show");
    let id = ItemId::New("SCRATCH-ITEM");
    let ledger = format!(
        "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
         \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [], \"patterns\": []}}, \"state\": \"Ready\"}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

    return (directory, id);
}

/// A scratch board carrying two real items, one blocking the other -- for the `Audit` tests.
/// `dependency` is `Ready` and unclaimed, so `blocked` earns a real `ClaimRefusal::
/// DependencyUnmet` from `nomos_ledger::Claim_Refusal` rather than a fixture-only refusal
/// this crate invented.
pub(crate) fn Scratch_Board_With_A_Blocked_Item() -> (std::path::PathBuf, ItemId, ItemId)
{
    let directory = Unique_Scratch_Directory("audit");
    let dependency = ItemId::New("SCRATCH-DEPENDENCY");
    let blocked = ItemId::New("SCRATCH-BLOCKED");
    let ledger = format!(
        "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{dependency}\", \"title\": \"t\", \
         \"why\": \"w\", \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \
         \"Proposed\", \"territory\": {{\"resolution\": \"File\", \"paths\": [\"a\"], \
         \"patterns\": []}}, \"state\": \"Ready\"}}, {{\"id\": \"{blocked}\", \"title\": \"t\", \
         \"why\": \"w\", \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \
         \"Proposed\", \"territory\": {{\"resolution\": \"File\", \"paths\": [\"b\"], \
         \"patterns\": []}}, \"state\": \"Ready\", \"depends_on\": [\"{dependency}\"]}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

    return (directory, dependency, blocked);
}

/// A scratch board carrying one real `Ready` item with a real, non-empty territory --
/// unlike `Scratch_Board_With_One_Item`'s deliberately territory-less fixture (built for
/// `Validate`'s own violation test), a document `Claim`/`Renew`/`TakeOver` write back
/// must itself stay valid, so every fixture below reserves something real.
pub(crate) fn Scratch_Board_With_A_Claimable_Item() -> (std::path::PathBuf, ItemId)
{
    let directory = Unique_Scratch_Directory("claimable");
    let id = ItemId::New("SCRATCH-CLAIMABLE");
    let ledger = format!(
        "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
         \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"a\"], \"patterns\": []}}, \"state\": \"Ready\"}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

    return (directory, id);
}

/// A scratch board carrying one real item already `Claimed`, with `holder` and
/// `expires_at_unix` chosen by the caller -- an active lease far in the future for a
/// `Renew` fixture, or one far in the past (lapsed) for a `TakeOver` fixture.
pub(crate) fn Scratch_Board_With_A_Claimed_Item(holder: &str, expires_at_unix: i64) -> (std::path::PathBuf, ItemId)
{
    let directory = Unique_Scratch_Directory("claimed");
    let id = ItemId::New("SCRATCH-CLAIMED");
    let ledger = format!(
        "{{\"schema_version\": 5, \"items\": [{{\"id\": \"{id}\", \"title\": \"t\", \"why\": \"w\", \
         \"done_when\": \"d\", \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"a\"], \"patterns\": []}}, \"state\": \"Claimed\", \
         \"claim\": {{\"holder\": \"{holder}\", \"acquired_at\": 1, \"lease_expires_at\": \
         {expires_at_unix}}}}}]}}\n"
    );
    std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

    return (directory, id);
}

/// A scratch board carrying two real items over the same territory: one already
/// `Claimed` with an active lease, the other `Ready` and unclaimed -- the real
/// `ClaimRefusal::HeldBy` trigger `crates/substrate/nomos-ledger/src/store/refusal.rs`'s
/// own `Held_Ground` looks for (another item's *live claim* over overlapping ground),
/// not the `ClaimRefusal::NotClaimable` a second `claim` of the same already-`Claimed`
/// item id would hit instead.
pub(crate) fn Scratch_Board_With_A_Held_Territory_Conflict() -> (std::path::PathBuf, ItemId)
{
    let directory = Unique_Scratch_Directory("held");
    let held = ItemId::New("SCRATCH-HELD");
    let contested = ItemId::New("SCRATCH-CONTESTED");
    let ledger = format!(
        "{{\"schema_version\": 5, \"items\": [\
         {{\"id\": \"{held}\", \"title\": \"t\", \"why\": \"w\", \"done_when\": \"d\", \
         \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"shared\"], \"patterns\": []}}, \"state\": \"Claimed\", \
         \"claim\": {{\"holder\": \"someone-else\", \"acquired_at\": 1, \"lease_expires_at\": \
         {far_future}}}}}, \
         {{\"id\": \"{contested}\", \"title\": \"t\", \"why\": \"w\", \"done_when\": \"d\", \
         \"kind\": \"Capability\", \"origin\": \"Proposed\", \"territory\": \
         {{\"resolution\": \"File\", \"paths\": [\"shared\"], \"patterns\": []}}, \"state\": \"Ready\"}}\
         ]}}\n",
        far_future = i64::from(u32::MAX)
    );
    std::fs::write(directory.join("ledger.json"), ledger).expect("writes a minimal valid ledger");

    return (directory, contested);
}
