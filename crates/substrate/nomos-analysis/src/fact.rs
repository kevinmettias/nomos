use crate::identity::{FactIdentity, FactKey};
use nomos_contracts::{Digest128, EvidenceClass, GenerationId, Guarantee, SchemaId};
use nomos_model::Content_Digest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactPayload
{
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

impl FactPayload
{
    #[must_use]
    pub fn New(schema: SchemaId, bytes: Vec<u8>) -> Self
    {
        return Self { schema, bytes };
    }

    #[must_use]
    pub fn Digest(&self) -> Digest128
    {
        return Content_Digest(&self.bytes);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedFact
{
    pub identity: FactIdentity,
    pub evidence: EvidenceClass,
    pub guarantee: Guarantee,
    pub payload: FactPayload,
}

impl MaterializedFact
{
    #[must_use]
    pub const fn Key(&self) -> &FactKey
    {
        return self.identity.Key();
    }

    #[must_use]
    pub const fn Generation(&self) -> GenerationId
    {
        return self.identity.generation;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Supersession
{
    pub invalidated_at: GenerationId,
    pub cause: String,
}

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

impl std::error::Error for FactError {}
