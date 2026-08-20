//! Reading a child's stdout and stderr while it runs.

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// How much to take from a pipe at a time.
const CHUNK_SIZE: usize = 8_192;

/// A stream being read while the process writing it is still running.
///
/// This is a thread rather than a read after the wait because a pipe holds a fixed
/// number of bytes — 64 KiB as measured on this platform — and a child that fills one
/// blocks in `write` until somebody reads. Nothing did. A predicate loud enough to fill
/// the buffer therefore never exited, was killed at its timeout, and a run that had
/// answered was recorded as a run nobody got an answer from. The failing direction is
/// the worse one: failure detail is exactly what makes output large, so the louder the
/// failure the likelier it was reported as a timeout instead.
pub(super) struct Drain
{
    /// What has been read so far.
    ///
    /// Shared with the reader rather than owned by it, so that a reader still blocked on
    /// a pipe somebody else is holding open does not take the output with it.
    collected: Arc<Mutex<Vec<u8>>>,
    /// Whether the reader reached end of file.
    finished: Arc<AtomicBool>,
}

/// Reads a stream to its end, keeping everything it yields.
///
/// A read error ends the drain exactly as end of file does. There is nothing useful to
/// report from here — the outcome belongs to the process, not to its pipe — and what was
/// read before the error is still worth keeping.
fn Drain_Into<R: Read>(source: &mut R, sink: &Mutex<Vec<u8>>)
{
    let mut chunk = [0_u8; CHUNK_SIZE];

    while let Ok(taken) = source.read(&mut chunk)
    {
        if taken == 0
        {
            break;
        }
        let Ok(mut buffer) = sink.lock()
        else
        {
            break;
        };
        let Some(slice) = chunk.get(..taken)
        else
        {
            break;
        };
        buffer.extend_from_slice(slice);
    }
}

impl Drain
{
    /// Starts reading `source` on a thread of its own.
    // rust-lifetime: allow: `std::thread::spawn` requires it. The reader is moved onto a
    // detached thread that outlives this call, so no borrow of the caller can reach it and
    // `'static` is the bound the standard library demands rather than one chosen here.
    pub(super) fn Reading<R: Read + Send + 'static>(mut source: R) -> Self
    {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let finished = Arc::new(AtomicBool::new(false));
        let sink = Arc::clone(&collected);
        let reached_end = Arc::clone(&finished);

        std::thread::spawn(move || {
            Drain_Into(&mut source, &sink);
            // atomic-ordering: allow: pairs with the `Acquire` load in `Finished`. Every write
            // `Drain_Into` made into `collected` through the mutex happens before this store, so a
            // reader that observes `true` is guaranteed to observe the drained bytes as well. A
            // `Relaxed` store would publish the flag without the buffer behind it.
            reached_end.store(true, Ordering::Release);
        });

        return Self {
            collected,
            finished,
        };
    }

    /// Whether the reader reached end of file.
    pub(super) fn Finished(&self) -> bool
    {
        // atomic-ordering: allow: pairs with the `Release` store in `Reading`. Reading `true` here
        // establishes happens-before against the draining thread, so the caller that then takes
        // `collected` sees the complete buffer rather than a prefix of it.
        return self.finished.load(Ordering::Acquire);
    }

    /// How many bytes have been read so far, without copying them.
    ///
    /// Cheap on purpose: this is polled once per [`super::POLL_INTERVAL`] for as long as a
    /// process runs, to notice whether it is still producing anything. Copying the whole
    /// buffer that often to answer a question only its length can answer would make the
    /// idle check itself the thing slowing a loud process down.
    pub(super) fn Len(&self) -> usize
    {
        return self.collected.lock().map_or(0, |buffer| return buffer.len());
    }

    /// What has been read, as text.
    ///
    /// Lossy rather than strict, which is a change in what gets recorded. `read_to_string`
    /// refuses the entire stream on one byte that is not UTF-8 and the caller discarded
    /// the error, so a predicate that printed a stray byte had the whole of its output
    /// recorded as nothing at all — the same shape of loss this type exists to end.
    pub(super) fn Text(&self) -> String
    {
        return self.collected.lock().map_or_else(
            |_| String::new(),
            |buffer| String::from_utf8_lossy(&buffer).into_owned(),
        );
    }
}
