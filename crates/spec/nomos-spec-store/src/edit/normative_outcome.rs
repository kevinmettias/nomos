//! What became of the statement: the outcome itself.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormativeOutcome
{
    /// Still in the block it was in.
    Held
    {
        block: u32,
    },
    /// Still present, in a different block.
    Moved
    {
        from: u32,
        to: u32,
    },
    /// Was in the record and is not in the staged text.
    Gone
    {
        from: u32,
    },
    /// Recorded against the record and not found in it before the edit either, so this edit
    /// cannot be what moved it. Reported rather than skipped: a statement the store cannot
    /// locate is a finding about the store.
    Unlocatable,
}

impl NormativeOutcome
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::Held { block } => format!("held in block {block}"),
            Self::Moved { from, to } => format!("moved from block {from} to {to}"),
            Self::Gone { from } => format!("gone from block {from}"),
            Self::Unlocatable => "not locatable in this record before the edit".to_owned(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Describe_Should_Name_What_Became_Of_The_Statement()
    {
        assert_eq!(NormativeOutcome::Held { block: 2 }.Describe(), "held in block 2");
        assert_eq!(NormativeOutcome::Moved { from: 1, to: 3 }.Describe(), "moved from block 1 to 3");
        assert_eq!(NormativeOutcome::Gone { from: 4 }.Describe(), "gone from block 4");
        assert_eq!(
            NormativeOutcome::Unlocatable.Describe(),
            "not locatable in this record before the edit"
        );
    }
}
