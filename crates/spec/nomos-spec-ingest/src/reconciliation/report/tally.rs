//! How many blocks landed in each fate.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally
{
    pub preserved: u32,
    pub hollowed: u32,
    pub mentioned: u32,
    pub gone: u32,
}

impl Tally
{
    #[must_use]
    pub const fn Total(&self) -> u32
    {
        return self
            .preserved
            .saturating_add(self.hollowed)
            .saturating_add(self.mentioned)
            .saturating_add(self.gone);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// One block in each fate bucket below, each count distinct so a `Total` that dropped a
    /// bucket, added one twice, or swapped two of them lands on a different number than the
    /// sum of all four.
    const PRESERVED_BLOCKS: u32 = 3;
    const HOLLOWED_BLOCKS: u32 = 1;
    const MENTIONED_BLOCKS: u32 = 2;
    const GONE_BLOCKS: u32 = 4;

    #[test]
    fn Test_Total_Should_Sum_Every_Fate_Bucket()
    {
        let tally = Tally {
            preserved: PRESERVED_BLOCKS,
            hollowed: HOLLOWED_BLOCKS,
            mentioned: MENTIONED_BLOCKS,
            gone: GONE_BLOCKS,
        };

        assert_eq!(
            tally.Total(),
            PRESERVED_BLOCKS + HOLLOWED_BLOCKS + MENTIONED_BLOCKS + GONE_BLOCKS
        );
        assert_eq!(Tally::default().Total(), 0);
    }
}
