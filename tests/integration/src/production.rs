//! What one domain produced, in the two projections a strength claim can be about.

use nomos_contracts::DeterminismStrength;
use nomos_model::{Content_Digest, Digest_Of_Parts};

/// What one domain produced, in the two projections a strength claim can be about.
///
/// Every payload this workspace produces is line-oriented, and deliberately so — the
/// encodings are hand-written precisely so that a derive cannot re-address them. That
/// makes the set projection well defined without any domain having to supply one: the
/// lines, sorted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Production
{
    /// The bytes as produced, in the order produced.
    pub trace: Vec<u8>,
}

impl Production
{
    /// The bytes, with the sequence discarded.
    ///
    /// What `DeterminismStrength::State` promises and `StateTemporal` promises on top of.
    #[must_use]
    pub fn State(&self) -> Vec<Vec<u8>>
    {
        let mut lines: Vec<Vec<u8>> = self
            .trace
            .split(|byte| return *byte == b'\n')
            .map(<[u8]>::to_vec)
            .collect();
        lines.sort();
        return lines;
    }

    /// The digest of what was produced, taken at the strength that was declared.
    ///
    /// The unit a cross-process and cross-platform comparison travels in: a child process
    /// prints this, and a committed golden is one of these.
    ///
    /// # Why the strength has to reach this
    ///
    /// A cross-run comparison of raw bytes would hold a `State` domain to byte-stable
    /// ordering across processes — which is `StateTemporal`, one step above what it
    /// declared. The check would pass today and would fail the first time a `State`
    /// domain legitimately reordered its output, and the failure would name a promise
    /// nobody made. Taking the digest at the declared strength is what keeps the scope
    /// axis and the strength axis independent, which is the entire reason the triple has
    /// three axes instead of one bit.
    #[must_use]
    pub fn Digest_At(&self, strength: DeterminismStrength) -> String
    {
        return match strength
        {
            DeterminismStrength::StateTemporal | DeterminismStrength::None =>
            {
                Content_Digest(&self.trace).to_string()
            }
            DeterminismStrength::State =>
            {
                let lines = self.State();
                let parts: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
                // Length-framed by `Digest_Of_Parts`, so that a set of lines cannot
                // collide with a differently-split set of the same bytes.
                Digest_Of_Parts(&parts).to_string()
            }
        };
    }
}
