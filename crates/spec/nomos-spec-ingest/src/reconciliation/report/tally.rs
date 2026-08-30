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

    #[test]
    fn Test_Total_Should_Sum_Every_Fate_Bucket()
    {
        let tally = Tally { preserved: 3, hollowed: 1, mentioned: 2, gone: 4 };

        assert_eq!(tally.Total(), 10);
        assert_eq!(Tally::default().Total(), 0);
    }
}
