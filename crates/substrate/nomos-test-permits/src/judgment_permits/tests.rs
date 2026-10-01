//! The two tests that falsify [`JudgmentPermits`], moved with it unchanged.

use super::JudgmentPermits;

/// How long the tests below wait before concluding a judgment was held back.
///
/// Only the passing direction depends on it. A permit that wrongly admits a judgment admits it at
/// once, which this reads as a failure well inside the window; a correct bound never admits it at
/// all, so no length of window can make a correct bound fail.
const HELD_BACK_WINDOW: std::time::Duration = std::time::Duration::from_millis(500);

/// How long the tests below wait for a returned permit to admit the judgment waiting on it.
///
/// A bound that never admits again would otherwise hang the test rather than fail it, and a hang
/// is the one outcome nobody can read. A correct bound admits within microseconds of the return,
/// so this window is generous by four orders of magnitude and bounds only the failure.
const ADMITTED_WINDOW: std::time::Duration = std::time::Duration::from_secs(30);

/// With every permit taken, the next judgment waits -- and runs as soon as one is returned.
///
/// The second half is the one that keeps the first honest: a bound that never admitted anyone
/// again would pass the first assertion forever. Both judgments run on detached threads rather
/// than scoped ones, so a bound that never admits the second fails this test at
/// [`ADMITTED_WINDOW`] instead of hanging it on a join that cannot return.
#[test]
fn Test_A_Judgment_Beyond_The_Limit_Should_Wait_Until_A_Permit_Is_Returned()
{
    let permits = std::sync::Arc::new(JudgmentPermits::New(1));
    let (holding, first_holds) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel::<()>();
    let (admitted, second_admitted) = std::sync::mpsc::channel();

    let first = std::sync::Arc::clone(&permits);
    std::thread::spawn(move || first.Holding(|| {
        holding.send(()).expect("the test body is still receiving");
        // Returns either way: released, or the body gave up and dropped the sender.
        let _released = released.recv();
    }));
    first_holds.recv_timeout(ADMITTED_WINDOW).expect("the first judgment takes the only permit");
    let second = std::sync::Arc::clone(&permits);
    std::thread::spawn(move || second.Holding(|| admitted.send(()).expect("the test body is still receiving")));

    assert!(second_admitted.recv_timeout(HELD_BACK_WINDOW).is_err(), "a second judgment ran while the only permit was held");
    release.send(()).expect("the first judgment is still waiting for this");
    second_admitted.recv_timeout(ADMITTED_WINDOW).expect("the returned permit must admit the judgment that was waiting for it");
}

/// A judgment that panics still returns its permit.
///
/// Without this, one failing root-judging test would leave every later one waiting forever, and
/// the suite would report a hang rather than the one assertion that actually failed. The judgment
/// after the panic is asked for on a thread of its own, so a permit that was never returned fails
/// this test at [`ADMITTED_WINDOW`] rather than hanging it.
#[test]
fn Test_A_Judgment_That_Panics_Should_Still_Return_Its_Permit()
{
    let permits = std::sync::Arc::new(JudgmentPermits::New(1));
    let (admitted, next_admitted) = std::sync::mpsc::channel();

    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| permits.Holding(|| panic!("a judgment that fails its own test"))));
    let waiting = std::sync::Arc::clone(&permits);
    std::thread::spawn(move || waiting.Holding(|| admitted.send(()).expect("the test body is still receiving")));

    assert!(unwound.is_err());
    next_admitted.recv_timeout(ADMITTED_WINDOW).expect("the panicked judgment's permit must have been returned");
}
