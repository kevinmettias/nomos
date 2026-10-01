//! A block a reader gains nothing from.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillerBlock
{
    pub document: String,
    pub ordinal: u32,
    pub pattern: &'static str,
    /// The v14 heading this block's document corresponds to, when one does.
    pub displaced: Option<String>,
}
