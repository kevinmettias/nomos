//! Four items, two of them public.

pub struct Anchor
{
    depth: u8,
}

impl Anchor
{
    pub fn Depth(&self) -> u8
    {
        return self.depth;
    }

    fn Hidden(&self) -> u8
    {
        return 0;
    }
}
