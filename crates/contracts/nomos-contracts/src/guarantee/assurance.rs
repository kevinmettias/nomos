//! Whether a formal property of a provider's output is claimed, denied, or unestablished.

use serde::{Deserialize, Serialize};

const SOUND_LABEL: &str = "Sound";
const UNSOUND_LABEL: &str = "Unsound";
const UNKNOWN_LABEL: &str = "Unknown";

/// Whether a provider claims a formal property of its output.
///
/// Three states, not two. [`Assurance::Unknown`] is the honest answer for most real
/// analyzers and it must not be spelled the same as [`Assurance::Unsound`] — one says
/// "this reports things that are not there", the other says "nobody has established
/// either way", and a consumer's response to them differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Assurance
{
    /// The property is claimed and the claim is backed.
    Sound,
    /// The property is known not to hold.
    Unsound,
    /// Nobody has established this either way.
    Unknown,
}

impl Assurance
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Sound => SOUND_LABEL,
            Self::Unsound => UNSOUND_LABEL,
            Self::Unknown => UNKNOWN_LABEL,
        };
    }

    /// Whether this assurance satisfies a requirement for the property.
    ///
    /// Only [`Assurance::Sound`] does. `Unknown` deliberately does not, because a
    /// requirement met by an absence of information is not a requirement.
    #[must_use]
    pub const fn Satisfies_Requirement(self) -> bool
    {
        return matches!(self, Self::Sound);
    }
}

impl core::fmt::Display for Assurance
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use alloc::vec::Vec;
    use super::*;

    /// `Label` is the `Display` form every variant renders through.
    #[test]
    fn Test_Label_Should_Spell_Every_Variant_Distinctly()
    {
        let all = [Assurance::Sound, Assurance::Unsound, Assurance::Unknown];

        let mut labels: Vec<&str> = all.iter().map(|assurance| return assurance.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two assurances share a wire spelling");
    }

    /// Only `Sound` satisfies a requirement; `Unknown` deliberately does not, because a
    /// requirement met by an absence of information is not a requirement.
    #[test]
    fn Test_Satisfies_Requirement_Should_Be_True_For_Sound_Only()
    {
        assert!(Assurance::Sound.Satisfies_Requirement());
        assert!(!Assurance::Unsound.Satisfies_Requirement());
        assert!(!Assurance::Unknown.Satisfies_Requirement());
    }
}
