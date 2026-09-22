//! What residency actually buys, and what it must not cost.
//!
//! Every assertion here is about one of four claims the crate makes: a second request over an
//! unchanged tree files nothing, a resident's judgment is a cold invocation's judgment, a
//! resident notices what moved and refuses what it cannot see, and a stopped or forgotten
//! resident is indistinguishable from one that never ran.
//!
//! # Why the cost assertions are relations and never a fact count
//!
//! `nomos_analysis::MemoryFactStore::Materializations` counts every store write, and one
//! judgment of this fixture writes several kinds -- one fact per source per per-source family,
//! and one per whole-workspace family. How many of each there are is a property of what this
//! build composes, not of reuse, so every assertion below is a relation between the deltas and
//! none of them names a total. `nomos-lsp`'s own equivalent suite records what a version of
//! this test that named one cost when the composition moved under it.

use super::*;

/// A fresh, empty directory under the system temporary directory.
fn Root(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a fresh directory under the system temporary directory is creatable");

    return root;
}

/// A root holding the two sources every fixture below starts from.
///
/// `a.rs` carries a public list with a doc comment that claims no mirror, which is what
/// `check-completeness-mirrors` reports. The fixture has it so that an assertion comparing two
/// finding lists is not two empty lists agreeing -- the vacuity every comparison in this
/// workspace is asked about.
fn Fixture(name: &str) -> PathBuf
{
    let root = Root(name);

    Write_Source(&root, "a.rs", "/// A list of things.\npub const THINGS: &[&str] = &[\"a\"];\n");
    Write_Source(&root, "b.rs", "pub fn Ok() {}\n");

    return root;
}

/// One source written into a root this module just created.
fn Write_Source(root: &Path, name: &str, text: &str)
{
    std::fs::write(root.join(name), text).expect("the fixture root was created by this module a moment ago");
}

/// The three halves of a judging answer, or a panic naming what came back instead.
fn Judged(answer: ResidentAnswer) -> (CheckOutcome, TreeReading, ResidencyCost)
{
    let ResidentAnswer::Judged { outcome, reading, cost } = answer
    else
    {
        panic!("a Judge request answers with Judged: {answer:?}");
    };

    return (outcome, reading, cost);
}

/// One judgment from a resident over `root`, with everything it was already holding.
fn Judgment(resident: &mut Resident) -> (CheckOutcome, TreeReading, ResidencyCost)
{
    let answer = resident.Answer(ResidentRequest::Judge).expect("a fixture root stays readable for the whole of one test");

    return Judged(answer);
}

/// The findings a cold invocation reaches over `root` -- a fresh workspace, a fresh store and
/// a cache built for the call, which is exactly what `nomos-cli::check` constructs per run.
///
/// It calls `Run` and not `Run_Reassessing`: a cold invocation has no cache to keep, and
/// asking the seam for the same entry point a shell invocation uses is what makes this a
/// comparison against the product rather than against a second spelling of the resident.
fn Cold_Judgment(root: &Path) -> CheckOutcome
{
    let sources = crate::sources::Walked_Sources(root).expect("a fixture root is a directory this process created");
    let mut workspace = None;
    let mut store = MemoryFactStore::New();

    return nomos_check_orchestration::Run(
        &sources,
        RunContext {
            variant: Host_Variant(),
            root,
            launcher: &LAUNCHER,
            filesystem: &FILE_SYSTEM,
            environment: &ENVIRONMENT,
            workspace: &mut workspace,
            store: &mut store,
        },
        &[],
    );
}

/// Every path under a directory, recursively, sorted -- what a tree looks like from outside.
///
/// Recursive rather than one level, because "leaves nothing behind" has to cover a store
/// written into a subdirectory as much as a marker dropped beside the sources.
fn Tree_Under(root: &Path) -> Vec<String>
{
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root)
    else
    {
        return found;
    };
    for entry in entries.flatten()
    {
        let path = entry.path();
        found.push(path.display().to_string());
        found.extend(Tree_Under(&path));
    }
    found.sort();

    return found;
}

/// How many findings a judged outcome carries, and none when it did not judge.
///
/// A count rather than the findings themselves: naming `nomos_contracts::Finding` here would
/// give this crate a dependency only a test reaches, and every comparison below that cares
/// about the findings compares the whole outcome instead, which carries them.
fn Findings_In(outcome: &CheckOutcome) -> usize
{
    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return 0;
    };

    return findings.len();
}

/// The claim this whole crate exists for: a second request over an unchanged tree files
/// nothing, and a request after one edit files less than the cold one did.
///
/// Four judgments rather than two, for `nomos-lsp`'s own reason: a second unchanged judgment
/// answering what the first answered is consistent with a resident that reused everything and
/// with one that recomputed everything. The third separates them, because only a resident that
/// really kept the store can charge for one source and not the other, and the fourth moves
/// both sources so the third can be read as a per-source cost without this test declaring one.
#[test]
fn Test_A_Second_Request_Over_An_Unchanged_Tree_Should_File_Nothing_And_An_Edit_Should_Cost_Less_Than_The_Cold_One()
{
    let root = Fixture("nomos-daemon-resident-cost-profile");
    let mut resident = Resident::Start(&root).expect("a fixture root is a directory this process created");

    let (_cold_outcome, cold_reading, cold) = Judgment(&mut resident);
    let (_unchanged_outcome, unchanged_reading, unchanged) = Judgment(&mut resident);

    Write_Source(&root, "a.rs", "/// A list of things.\npub const THINGS: &[&str] = &[\"a\", \"b\"];\n");
    let (_edited_outcome, edited_reading, edited) = Judgment(&mut resident);

    Write_Source(&root, "a.rs", "/// A list of things.\npub const THINGS: &[&str] = &[\"a\", \"b\", \"c\"];\n");
    Write_Source(&root, "b.rs", "pub fn Ok() {}\npub fn Also_Ok() {}\n");
    let (_both_outcome, both_reading, both) = Judgment(&mut resident);

    let _ignored = std::fs::remove_dir_all(&root);
    println!("materializations: cold {} unchanged {} one-edit {} both-edited {}", cold.materializations, unchanged.materializations, edited.materializations, both.materializations);
    assert!(cold.materializations > 0, "a cold request must file the facts it judged over");
    assert_eq!(unchanged.materializations, 0, "an unchanged tree must file nothing a second time");
    assert!(edited.materializations > 0, "an edited source must file again");
    assert!(edited.materializations < cold.materializations, "one moved source must cost less than the whole tree: {edited:?} against {cold:?}");
    assert!(edited.materializations < both.materializations, "one moved source must cost less than two: {edited:?} against {both:?}");
    assert_eq!(both_reading.changed, vec!["a.rs".to_owned(), "b.rs".to_owned()], "both sources moved: {both_reading:?}");
    assert_eq!(cold_reading.changed.len(), cold_reading.watched, "a resident holding nothing has compared nothing, so everything it walked has moved");
    assert!(!unchanged_reading.Moved(), "an untouched tree has moved nothing: {unchanged_reading:?}");
    assert_eq!(edited_reading.changed, vec!["a.rs".to_owned()], "only the edited source moved: {edited_reading:?}");
}

/// The agreement proof: one tree, two paths to a verdict, one verdict.
///
/// The resident judges twice before the comparison deliberately. A first judgment is a cold
/// one wearing a different name; the second is the one that had a workspace, a store and a
/// reassessment cache to answer from, and is therefore the only one where reuse could have
/// changed an answer.
#[test]
fn Test_A_Reused_Judgment_Should_Be_The_Judgment_A_Cold_Invocation_Reaches()
{
    let root = Fixture("nomos-daemon-resident-agrees-with-cold");
    let mut resident = Resident::Start(&root).expect("a fixture root is a directory this process created");

    let (_first, _reading, _cost) = Judgment(&mut resident);
    let (reused, _second_reading, _second_cost) = Judgment(&mut resident);
    let cold = Cold_Judgment(&root);

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(Findings_In(&cold) > 0, "the fixture must produce a finding, or two empty outcomes agree having judged nothing");
    assert_eq!(reused, cold, "a reused judgment and a cold one must reach the same outcome, examined count and claim");
}

/// A root that goes away is refused, not answered from what is still held.
///
/// The resident is asked to judge first, so it is holding a complete judgment of that tree
/// when the tree is removed. That is the only state in which a cache could be passed off as an
/// observation, which is why the removal happens after a judgment and not before one.
#[test]
fn Test_A_Root_That_Stopped_Being_Readable_Should_Be_Refused_Rather_Than_Answered_From_What_Is_Held()
{
    let root = Fixture("nomos-daemon-resident-root-taken-away");
    let mut resident = Resident::Start(&root).expect("a fixture root is a directory this process created");

    let (judged, _reading, _cost) = Judgment(&mut resident);
    let _ignored = std::fs::remove_dir_all(&root);
    let after = resident.Answer(ResidentRequest::Judge);
    let observed = resident.Answer(ResidentRequest::Observe);

    assert!(Findings_In(&judged) > 0, "the resident must be holding a real judgment when the root is taken away");
    assert_eq!(after, Err(ResidentRefusal::RootIsNotWalkable { root: root.clone() }), "a judge over a root that is gone must be refused");
    assert_eq!(observed, Err(ResidentRefusal::RootIsNotWalkable { root }), "an observe over a root that is gone must be refused");
}

/// Stopping releases what was held, refuses everything after it, and leaves the tree alone.
///
/// The tree comparison is what makes "leaves nothing a later cold invocation would mistake for
/// current state" checkable rather than asserted: a resident that had written a store, a lock
/// or a marker anywhere under the root would show it here.
#[test]
fn Test_A_Stopped_Resident_Should_Release_What_It_Held_Refuse_Every_Later_Request_And_Leave_The_Tree_Alone()
{
    let root = Fixture("nomos-daemon-resident-stop");
    let before = Tree_Under(&root);
    let mut resident = Resident::Start(&root).expect("a fixture root is a directory this process created");

    let (_judged, _reading, cost) = Judgment(&mut resident);
    let stopped = resident.Answer(ResidentRequest::Stop).expect("a resident that has not been stopped answers a stop");
    let after_stop = resident.Answer(ResidentRequest::Judge);
    let after_tree = Tree_Under(&root);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(stopped, ResidentAnswer::Stopped(StopReport { answered: 1, released_facts: cost.live_facts }), "a stop reports what it answered and what it released");
    assert_eq!(after_stop, Err(ResidentRefusal::AlreadyStopped), "a stopped resident refuses every later request");
    assert_eq!(before, after_tree, "a resident writes nothing under its root, so stopping leaves the tree exactly as it was found");
}

/// `OD-HOST-002`'s own test, run rather than argued: dropping everything held costs latency
/// and not information.
///
/// The forgotten resident pays a cold request's price again and reaches a cold request's
/// answer. Both halves matter -- the price is what proves it really let go, and the answer is
/// what proves nothing it was holding was information nobody else had.
#[test]
fn Test_A_Forgotten_Resident_Should_Pay_A_First_Requests_Price_Again_And_Reach_The_Same_Answer()
{
    let root = Fixture("nomos-daemon-resident-forget");
    let mut resident = Resident::Start(&root).expect("a fixture root is a directory this process created");

    let (first, _first_reading, cold) = Judgment(&mut resident);
    let forgotten = resident.Answer(ResidentRequest::Forget).expect("a running resident answers a forget");
    let (after, after_reading, after_cost) = Judgment(&mut resident);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(forgotten, ResidentAnswer::Forgotten { released_facts: cold.live_facts }, "a forget reports the facts it released");
    assert_eq!(after_cost.materializations, cold.materializations, "a resident that let go pays what a first request pays");
    assert_eq!(after_reading.changed.len(), after_reading.watched, "a resident that let go is holding nothing to compare against");
    assert_eq!(after, first, "a resident that let go reaches the answer it reached before it held anything");
}

/// An observation reads and compares without judging, and without advancing what it compares
/// against.
#[test]
fn Test_An_Observation_Should_Report_What_Moved_Without_Judging_Or_Advancing_What_It_Compares_Against()
{
    let root = Fixture("nomos-daemon-resident-observe");
    let mut resident = Resident::Start(&root).expect("a fixture root is a directory this process created");

    let (_judged, _reading, _cost) = Judgment(&mut resident);
    Write_Source(&root, "b.rs", "pub fn Ok() {}\npub fn Also_Ok() {}\n");
    let first = resident.Answer(ResidentRequest::Observe).expect("a fixture root stays readable for the whole of one test");
    let second = resident.Answer(ResidentRequest::Observe).expect("a fixture root stays readable for the whole of one test");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(first, ResidentAnswer::Observed { reading: TreeReading { watched: 2, changed: vec!["b.rs".to_owned()] } }, "an observation names the moved source");
    assert_eq!(first, second, "an observation ingests nothing, so a second one is measured against the same held state");
}

/// Two residents over one root share it safely, because neither writes to it.
///
/// Run on two threads rather than one after the other: what is being asserted is that there is
/// no shared mutable state between two residencies over one tree, and a sequential pair cannot
/// distinguish that from luck.
#[test]
fn Test_Two_Residents_Over_One_Root_Should_Reach_The_Same_Answer_At_The_Same_Time()
{
    let root = Fixture("nomos-daemon-two-residents-one-root");

    let left = std::thread::scope(|scope| {
        let other = scope.spawn(|| return Answered_Alone(&root));
        let mine = Answered_Alone(&root);

        return (mine, other.join());
    });

    let _ignored = std::fs::remove_dir_all(&root);
    let (mine, other) = left;
    let other = other.expect("a thread judging a fixture root does not panic");
    assert!(Findings_In(&mine) > 0, "each residency must reach a real judgment, or two empty lists agree having judged nothing");
    assert_eq!(mine, other, "two residents over one root must reach the same answer");
}

/// One judgment from a residency of its own over `root`.
fn Answered_Alone(root: &Path) -> CheckOutcome
{
    let mut resident = Resident::Start(root).expect("a fixture root is a directory this process created");
    let (outcome, _reading, _cost) = Judgment(&mut resident);

    return outcome;
}

/// Two requests to one resident cannot be concurrent, and each is answered with the state the
/// one before it left behind.
///
/// This is the whole of what "serialize" means here, because a resident cannot be shared
/// across threads at all -- the crate doc records the measurement and what in
/// `nomos-analysis` causes it. So the assertion is about the sequence rather than about a
/// lock: both requests are counted, and the second one is measured against what the first
/// ingested rather than against an empty workspace.
#[test]
fn Test_Two_Requests_To_One_Resident_Should_Each_See_What_The_One_Before_It_Left()
{
    let root = Fixture("nomos-daemon-one-resident-two-requests");
    let mut resident = Resident::Start(&root).expect("a fixture root is a directory this process created");

    let (first, _first_reading, _first_cost) = Judgment(&mut resident);
    let (second, second_reading, second_cost) = Judgment(&mut resident);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(resident.Answered(), 2, "both requests must have been answered");
    assert_eq!(first, second, "two requests over an unmoved tree must reach the same answer");
    assert_eq!(second_cost.materializations, 0, "the second request must be answered from what the first left");
    assert!(!second_reading.Moved(), "the second request must be measured against what the first ingested: {second_reading:?}");
}

/// What crosses a thread boundary is the answer, not the resident.
///
/// A residency is confined to the thread that started it, so the shape a concurrent caller
/// actually has is a resident per thread and an answer handed back -- which is what this
/// asserts, by taking a `CheckOutcome` out of a thread that built its own residency.
#[test]
fn Test_An_Answer_Should_Cross_A_Thread_Boundary_Even_Though_A_Resident_Cannot()
{
    let root = Fixture("nomos-daemon-answer-crosses-a-thread");

    let handle = std::thread::scope(|scope| {
        let spawned = scope.spawn(|| return Answered_Alone(&root));

        return spawned.join();
    });

    let _ignored = std::fs::remove_dir_all(&root);
    let answered = handle.expect("a thread judging a fixture root does not panic");
    assert!(Findings_In(&answered) > 0, "the answer that crossed must be a real judgment, or nothing was carried");
}

#[test]
fn Test_Starting_Against_Something_That_Is_Not_A_Directory_Should_Be_Refused()
{
    let root = Root("nomos-daemon-resident-start-refusal");
    let file = root.join("a.rs");
    Write_Source(&root, "a.rs", "pub fn Ok() {}\n");

    let refused = Resident::Start(&file);
    let missing = Resident::Start(&root.join("nowhere"));

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(refused.err(), Some(ResidentRefusal::RootIsNotWalkable { root: file }), "a file is not a root");
    assert_eq!(missing.err(), Some(ResidentRefusal::RootIsNotWalkable { root: root.join("nowhere") }), "a path that does not exist is not a root");
}
