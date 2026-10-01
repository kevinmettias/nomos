//! One answer per distinct set of inputs, computed once and handed to every caller that asks
//! for exactly those inputs.

use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// A memo from the complete inputs of a judgment to the one answer they produce.
///
/// The key is the whole input and it is compared by equality, never by a digest or by a part of
/// it: two callers share an answer exactly when nothing they handed in differs. The judgment is
/// handed the stored key and nothing else, so what is judged cannot drift from what the answer is
/// filed under -- a caller cannot compute from one input and file the result under another.
///
/// What equality cannot see is anything the judgment reads that is not in the key. That is why
/// callers share only judgments over a tree no test writes to; a tree a test rewrites between two
/// equal-looking calls would be two different questions under one key.
pub(crate) struct SharedJudgments<Key, Answer>
{
    entries: Mutex<Vec<Arc<Entry<Key, Answer>>>>,
}

/// One set of inputs and the answer, once some caller has computed it.
struct Entry<Key, Answer>
{
    key: Key,
    answer: OnceLock<Answer>,
}

impl<Key: PartialEq, Answer: Clone> SharedJudgments<Key, Answer>
{
    /// A memo holding nothing yet.
    pub(crate) const fn New() -> Self
    {
        return Self { entries: Mutex::new(Vec::new()) };
    }

    /// The answer `judge` gives for `key`, computed by the first caller to ask and shared with
    /// every later one.
    ///
    /// A second caller asking while the first is still judging waits for that answer rather
    /// than computing its own, which is the point: two concurrent callers are exactly the case
    /// where a duplicate judgment would have cost the most. The list lock is held only to find
    /// or file the entry, never across a judgment, so callers with different keys never wait for
    /// each other here.
    pub(crate) fn Answer_For(&self, key: Key, judge: impl FnOnce(&Key) -> Answer) -> Answer
    {
        let entry = self.Entry_For(key);

        return entry.answer.get_or_init(|| return judge(&entry.key)).clone();
    }

    fn Entry_For(&self, key: Key) -> Arc<Entry<Key, Answer>>
    {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = entries.iter().find(|entry| return entry.key == key)
        {
            return Arc::clone(existing);
        }
        let entry = Arc::new(Entry { key, answer: OnceLock::new() });
        entries.push(Arc::clone(&entry));

        return entry;
    }
}

/// Equal inputs are judged once; each distinct input is judged for itself and gets its own answer.
///
/// Both halves are asserted, because each is satisfied by a memo that gets the other one wrong:
/// a memo keyed on nothing shares everything and passes the first, and a memo that never shares
/// passes the second.
#[test]
fn Test_Equal_Inputs_Should_Share_One_Judgment_And_Distinct_Inputs_Should_Not()
{
    let memo: SharedJudgments<(u8, u8), String> = SharedJudgments::New();
    let judged = std::sync::atomic::AtomicUsize::new(0);
    let judge = |key: &(u8, u8)| {
        judged.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        return format!("{key:?}");
    };

    let first = memo.Answer_For((1, 2), judge);
    let again = memo.Answer_For((1, 2), judge);
    let other_first = memo.Answer_For((3, 2), judge);
    let other_second = memo.Answer_For((1, 3), judge);

    assert_eq!((first.as_str(), again.as_str()), ("(1, 2)", "(1, 2)"));
    assert_eq!((other_first.as_str(), other_second.as_str()), ("(3, 2)", "(1, 3)"), "an input differing in either half is a different judgment");
    assert_eq!(judged.load(std::sync::atomic::Ordering::SeqCst), 3, "the repeated input was judged again");
}
