use nomos_contracts::{Digest128, SchemaId};
use nomos_model::Digest_Of_Parts;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Authority
{
    Observed,
    Authored,
}

impl Authority
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Observed => "observed",
            Self::Authored => "authored",
        };
    }

    #[must_use]
    pub const fn Admits(self, kind: DocumentKind) -> bool
    {
        return kind.Authority() as u8 == self as u8;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocumentKind
{
    Snapshot,
    Fact,
    Finding,
    Run,
    Specification,
    Record,
    Projection,
}

impl DocumentKind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Snapshot => "snapshot",
            Self::Fact => "fact",
            Self::Finding => "finding",
            Self::Run => "run",
            Self::Specification => "specification",
            Self::Record => "record",
            Self::Projection => "projection",
        };
    }

    #[must_use]
    pub const fn Authority(self) -> Authority
    {
        return match self
        {
            Self::Snapshot | Self::Fact | Self::Finding | Self::Run => Authority::Observed,
            Self::Specification | Self::Record | Self::Projection => Authority::Authored,
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Snapshot,
            Self::Fact,
            Self::Finding,
            Self::Run,
            Self::Specification,
            Self::Record,
            Self::Projection,
        ];
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentId(Digest128);

impl DocumentId
{
    #[must_use]
    pub const fn From_Digest(digest: Digest128) -> Self
    {
        return Self(digest);
    }

    #[must_use]
    pub const fn Digest(self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for DocumentId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.0.fmt(formatter);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document
{
    pub kind: DocumentKind,
    pub schema: SchemaId,
    pub bytes: Vec<u8>,
}

impl Document
{
    #[must_use]
    pub fn New(kind: DocumentKind, schema: SchemaId, bytes: Vec<u8>) -> Self
    {
        return Self {
            kind,
            schema,
            bytes,
        };
    }

    #[must_use]
    pub fn Id(&self) -> DocumentId
    {
        return DocumentId(Digest_Of_Parts(&[
            self.kind.Label().as_bytes(),
            self.schema.As_Str().as_bytes(),
            &self.bytes,
        ]));
    }

    #[must_use]
    pub fn Authority(&self) -> Authority
    {
        return self.kind.Authority();
    }
}
