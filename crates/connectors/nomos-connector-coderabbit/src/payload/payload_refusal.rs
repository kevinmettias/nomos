/// A payload could not be read back out of its canonical encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadRefusal
{
    pub reason: String,
}

impl core::fmt::Display for PayloadRefusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}
