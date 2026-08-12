//! A block that survived somewhere else.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relocation
{
    pub to: String,
    pub from: Vec<String>,
}
