//! One configured program: target, profile, toolchain and features.
//!
//! A build variant is part of a fact's identity because "the same function" compiled for
//! two targets is two things to measure and one thing to talk about. Getting the identity
//! wrong in either direction is expensive: too coarse and two variants share a cache entry
//! that describes one of them, too fine and every fact recomputes because a feature list
//! arrived in a different order.

use nomos_contracts::BuildVariantId;
use std::collections::BTreeSet;

/// A configured program variant.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuildVariant
{
    pub target: String,
    pub profile: String,
    pub toolchain: String,
    /// Enabled features.
    ///
    /// A set, and an ordered one. Cargo hands features over in whatever order it resolved
    /// them, and a `Vec` would make `["a", "b"]` and `["b", "a"]` two variants — so every
    /// fact in the store would be recomputed because a resolver's iteration order changed.
    /// The set also collapses a feature named twice, which is the same defect wearing a
    /// duplicate.
    pub features: BTreeSet<String>,
}

impl BuildVariant
{
    #[must_use]
    pub fn New(
        target: impl Into<String>,
        profile: impl Into<String>,
        toolchain: impl Into<String>,
        features: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self
    {
        return Self {
            target: target.into(),
            profile: profile.into(),
            toolchain: toolchain.into(),
            features: features.into_iter().map(Into::into).collect(),
        };
    }

    /// The variant's identity.
    ///
    /// Every component is length-delimited by [`Digest_Of_Parts`] rather than joined into
    /// one string. Joining with a separator means a target named `a` with feature `b::c`
    /// and a target named `a::b` with feature `c` can hash the same, and two variants
    /// sharing an identity is two programs sharing a cache.
    #[must_use]
    pub fn Id(&self) -> BuildVariantId
    {
        use nomos_model::Digest_Of_Parts;

        let mut parts: Vec<&[u8]> = vec![
            self.target.as_bytes(),
            self.profile.as_bytes(),
            self.toolchain.as_bytes(),
        ];
        parts.extend(self.features.iter().map(|feature| return feature.as_bytes()));

        return BuildVariantId::From_Digest(Digest_Of_Parts(&parts));
    }

    /// The variant as one line of a snapshot, features comma-separated in set order.
    #[must_use]
    pub fn Rendered(&self) -> String
    {
        return format!(
            "{}\t{}\t{}\t{}",
            self.target,
            self.profile,
            self.toolchain,
            self.features
                .iter()
                .cloned()
                .collect::<Vec<String>>()
                .join(",")
        );
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The property that stops a resolver's iteration order from invalidating a corpus.
    #[test]
    fn Test_Rendered_Should_Ignore_Feature_Order()
    {
        let one = BuildVariant::New("t", "dev", "1.85", ["alpha", "beta", "gamma"]);
        let other = BuildVariant::New("t", "dev", "1.85", ["gamma", "alpha", "beta"]);

        assert_eq!(one, other);
        assert_eq!(one.Id(), other.Id());
        assert_eq!(one.Rendered(), other.Rendered());
    }

    /// The negative control. If the identity ignored features, the test above would pass
    /// while every feature combination shared one variant — and one build's facts would
    /// answer for another's.
    #[test]
    fn Test_Different_Features_Should_Be_Different_Variants()
    {
        assert_ne!(
            Host().Id(),
            BuildVariant::New("x86_64-pc-windows-msvc", "dev", "1.85", ["telemetry"]).Id()
        );
        assert_ne!(
            Host().Id(),
            BuildVariant::New("x86_64-pc-windows-msvc", "dev", "1.85", Vec::<String>::new()).Id()
        );
    }

    #[test]
    fn Test_Every_Component_Should_Reach_The_Identity()
    {
        let base = Host();

        for altered in Altered_Variants()
        {
            assert_ne!(base.Id(), altered.Id(), "{altered:?} must not share an identity");
        }
    }

    fn Host() -> BuildVariant
    {
        return BuildVariant::New(
            "x86_64-pc-windows-msvc",
            "dev",
            "1.85",
            ["telemetry", "analysis"],
        );
    }

    /// One host variant altered in each component in turn: target, profile, and toolchain.
    fn Altered_Variants() -> Vec<BuildVariant>
    {
        return vec![
            BuildVariant::New("aarch64-apple-darwin", "dev", "1.85", ["telemetry", "analysis"]),
            BuildVariant::New("x86_64-pc-windows-msvc", "release", "1.85", ["telemetry", "analysis"]),
            BuildVariant::New("x86_64-pc-windows-msvc", "dev", "nightly", ["telemetry", "analysis"]),
        ];
    }

    /// Components are delimited, not concatenated. Two variants whose fields differ only
    /// in where one field ends and the next begins must not collide.
    #[test]
    fn Test_Id_Should_Not_Let_A_Field_Boundary_Be_Forged()
    {
        let left = BuildVariant::New("a", "b", "c", ["d"]);
        let right = BuildVariant::New("ab", "c", "d", Vec::<String>::new());

        assert_ne!(left.Id(), right.Id());
    }

    /// A feature named twice is one feature. Otherwise a caller that appended before
    /// checking would produce a variant nothing else in the system can reach.
    #[test]
    fn Test_New_Should_Deduplicate_A_Repeated_Feature()
    {
        assert_eq!(
            BuildVariant::New("t", "dev", "1.85", ["alpha", "alpha"]).Id(),
            BuildVariant::New("t", "dev", "1.85", ["alpha"]).Id()
        );
    }
}
