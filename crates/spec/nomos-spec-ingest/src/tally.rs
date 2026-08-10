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
