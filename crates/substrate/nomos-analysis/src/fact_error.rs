//! Every way a fact refuses to be stored or read.
//!
//! Declared at the crate root rather than in `fact/`, and `lib.rs` says why where it
//! declares it: the name it is published under already carries the fact, so a file named
//! for it cannot also sit inside the folder that name would otherwise group it with.

use nomos_contracts::GenerationId;
use crate::FactIdentity;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactError
{
    Absent
    {
        identity: Box<FactIdentity>,
    },
    Superseded
    {
        identity: Box<FactIdentity>,
        invalidated_at: GenerationId,
    },
    Backdated
    {
        identity: Box<FactIdentity>,
        current: GenerationId,
    },
}

impl core::fmt::Display for FactError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Absent { identity } => write!(
                formatter,
                "no fact for {identity}. Absent is not empty and not false: nothing has \
                 computed this"
            ),
            Self::Superseded {
                identity,
                invalidated_at,
            } => write!(
                formatter,
                "{identity} was invalidated at {invalidated_at}. It is evidence about the \
                 generation it was computed for and is not readable as current"
            ),
            Self::Backdated { identity, current } => write!(
                formatter,
                "{identity} would be written behind {current}. A fact may not be materialized \
                 into a generation the store has already left"
            ),
        };
    }
}

impl std::error::Error for FactError
{}
