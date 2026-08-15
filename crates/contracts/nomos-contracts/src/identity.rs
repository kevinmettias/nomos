//! Stable identifiers, and the digest they are all built from.
//!
//! Every identity in Nomos is one of two shapes: a 128-bit content digest, or a
//! human-authored string with a stable spelling. Nothing is identified by its path and
//! line, ever — that is the single most consequential lesson carried over from the
//! prototype, where every comparative feature (diffing, history, trajectories,
//! co-change, transformation tracking) turned out to be a join with no key.

// The generation an identity was minted in.
mod generation_id;

pub use generation_id::GenerationId;

use serde::{Deserialize, Serialize};

/// A 128-bit content digest, rendered as 32 lowercase hex characters.
///
/// One shape, many uses. Truncating a 256-bit hash to 128 bits is deliberate: these
/// identifiers appear in file names, URLs, log lines and terminal output, and 32
/// characters is the point where a human can still compare two of them by eye. The
/// collision margin at 128 bits is not the binding constraint for a workspace-scale
/// corpus; legibility is.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Digest128([u8; 16]);

impl Digest128
{
    /// The number of bytes in the digest.
    pub const BYTE_LENGTH: usize = 16;

    /// The number of hex characters in the rendered form.
    pub const HEX_LENGTH: usize = 32;

    /// Wraps raw digest bytes.
    #[must_use]
    pub const fn From_Bytes(bytes: [u8; Self::BYTE_LENGTH]) -> Self
    {
        return Self(bytes);
    }

    /// The raw digest bytes.
    #[must_use]
    pub const fn Bytes(&self) -> &[u8; Self::BYTE_LENGTH]
    {
        return &self.0;
    }
}

impl core::fmt::Display for Digest128
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        for byte in &self.0
        {
            write!(formatter, "{byte:02x}")?;
        }
        return Ok(());
    }
}

/// Rendered as its hex form rather than as a byte array, because a digest in a debug
/// dump that reads `[187, 12, 244, …]` cannot be matched against the same digest in a
/// log line, and matching them is the only reason to print one.
impl core::fmt::Debug for Digest128
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "Digest128({self})");
    }
}

/// Declares a newtype over [`Digest128`] with the standard derives and accessors.
///
/// These identifiers are distinct types rather than aliases so that handing a snapshot
/// identity to something expecting a subject identity is a compile error. The prototype
/// carried a `Waiver { Check: path, Path: check }` that compiled, ran, and suppressed
/// nothing for as long as it existed — the fields were both `string`, so nothing could
/// have caught it.
macro_rules! Digest_Identity
{
    ($(#[$attribute:meta])* $name:ident) =>
    {
        $(#[$attribute])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(Digest128);

        impl $name
        {
            /// Wraps a digest as this identity.
            #[must_use]
            pub const fn From_Digest(digest: Digest128) -> Self
            {
                return Self(digest);
            }

            /// The underlying digest.
            #[must_use]
            pub const fn Digest(&self) -> Digest128
            {
                return self.0;
            }
        }

        impl core::fmt::Display for $name
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
            {
                return self.0.fmt(formatter);
            }
        }
    };
}

Digest_Identity!
{
    /// Identity of an addressable analysis subject.
    SubjectId
}

Digest_Identity!
{
    /// Identity of a pinned workspace state.
    SnapshotId
}

Digest_Identity!
{
    /// Identity of one canonical entity as it appears under one snapshot, build variant
    /// and configuration.
    ///
    /// Findings, architecture nodes, feature members and test paths all reference this
    /// rather than a bare entity identity, because "the same function" in two build
    /// variants is two different things to measure and one thing to talk about.
    SnapshotEntityId
}

Digest_Identity!
{
    /// Identity of one configured program variant — target, features, profile, toolchain.
    BuildVariantId
}

Digest_Identity!
{
    /// Digest of a fully resolved effective policy.
    ///
    /// Being a digest rather than a name is what makes configuration change invalidate
    /// exactly the right cached facts for free: it is a component of a fact's identity,
    /// so a changed policy simply fails to match rather than needing a rule that
    /// remembers to invalidate.
    ConfigurationId
}

Digest_Identity!
{
    /// Identity of one execution of a workflow or gate.
    RunId
}

/// Declares a newtype over an authored, human-readable identifier.
macro_rules! Named_Identity
{
    ($(#[$attribute:meta])* $name:ident) =>
    {
        $(#[$attribute])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name
        {
            /// Wraps an authored identifier.
            #[must_use]
            pub fn New(value: impl Into<String>) -> Self
            {
                return Self(value.into());
            }

            /// The identifier as authored.
            #[must_use]
            pub fn As_Str(&self) -> &str
            {
                return &self.0;
            }
        }

        impl core::fmt::Display for $name
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
            {
                return formatter.write_str(&self.0);
            }
        }
    };
}

Named_Identity!
{
    /// Identity of a capability contract, such as `nomos.cap.syntax.tree`.
    CapabilityId
}

Named_Identity!
{
    /// Identity of a normative rule.
    RuleId
}

Named_Identity!
{
    /// Identity of a provider implementation.
    ProviderId
}

Named_Identity!
{
    /// Identity of an installable package.
    PackageId
}

Named_Identity!
{
    /// Identity of a schema, such as `nomos.finding.v1`.
    SchemaId
}

Named_Identity!
{
    /// The canonical name of a service operation, such as `nomos.findings.query`.
    ///
    /// This is the **only** authoritative name for an operation. A CLI spelling, an MCP
    /// tool name, an HTTP route and a generated SDK method are all projections of this
    /// and none of them may disagree with it or with each other.
    OperationName
}

Named_Identity!
{
    /// Identity of one claim, rationale, or decision minted by an external knowledge
    /// system, carried by Nomos as a citation.
    ///
    /// Deliberately a [`Named_Identity`] and not a [`Digest_Identity`]: Nomos does not
    /// compute this value from bytes it holds, so wrapping it as one of Nomos's own
    /// content digests would claim a verification Nomos cannot perform. The identity is
    /// opaque here on purpose — this crate names the shape a citation takes, not what
    /// the referenced thing means, which stays the citing system's business.
    KnowledgeReferenceId
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Digest_Should_Render_As_Thirty_Two_Hex_Characters()
    {
        let digest = Digest128::From_Bytes([0x0a; Digest128::BYTE_LENGTH]);
        let rendered = digest.to_string();

        assert_eq!(rendered.len(), Digest128::HEX_LENGTH);
        assert_eq!(rendered, "0a".repeat(Digest128::BYTE_LENGTH));
    }

    /// A leading zero byte must survive rendering. Formatting a digest with `{:x}` per
    /// byte and no width would drop it, and two distinct digests would print the same.
    #[test]
    fn Test_Digest_Should_Zero_Pad_Every_Byte()
    {
        let mut bytes = [0x00_u8; Digest128::BYTE_LENGTH];
        bytes[Digest128::BYTE_LENGTH - 1] = 0x0f;
        let low = Digest128::From_Bytes(bytes);

        bytes[Digest128::BYTE_LENGTH - 1] = 0xf0;
        let high = Digest128::From_Bytes(bytes);

        assert_eq!(low.to_string().len(), Digest128::HEX_LENGTH);
        assert_eq!(high.to_string().len(), Digest128::HEX_LENGTH);
        assert_ne!(low.to_string(), high.to_string());
    }

    /// Debug must be matchable against Display, or a digest in a panic message cannot
    /// be found in a log.
    #[test]
    fn Test_Digest_Debug_Should_Contain_The_Hex_Form()
    {
        let digest = Digest128::From_Bytes([0xab; Digest128::BYTE_LENGTH]);

        assert!(format!("{digest:?}").contains(&digest.to_string()));
    }
}
