//! The ways a block can be present and say nothing.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hollow
{
    Template
    {
        shared_with: u32,
        declared: Option<&'static str>,
    },
    NoBody,
}
