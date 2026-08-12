//! The identifier a normative statement is cited by.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementId(String);

impl StatementId
{
    /// Accepts `PREFIX-NNN`, where the prefix is upper-case ASCII with optional inner
    /// hyphens and the suffix is at least three digits.
    #[must_use]
    pub fn Parse(text: &str) -> Option<Self>
    {
        let (prefix, number) = text.rsplit_once('-')?;

        if prefix.is_empty() || !Is_A_Suffix(number)
        {
            return None;
        }

        let prefix_ok = prefix
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()));

        return prefix_ok.then(|| Self(text.to_owned()));
    }

    #[must_use]
    pub fn Prefix(&self) -> &str
    {
        return self.0.rsplit_once('-').map_or("", |(prefix, _)| prefix);
    }

    #[must_use]
    pub fn As_Str(&self) -> &str
    {
        return &self.0;
    }
}

/// Three digits at least, because two would let a section number read as a statement.
const SHORTEST_SUFFIX: usize = 3;

/// Whether a trailing segment is the number a statement identifier ends in.
fn Is_A_Suffix(number: &str) -> bool
{
    return number.len() >= SHORTEST_SUFFIX && All_Digits(number);
}

/// A non-empty run of ASCII digits and nothing else.
fn All_Digits(text: &str) -> bool
{
    return !text.is_empty() && text.bytes().all(|byte| return byte.is_ascii_digit());
}

impl core::fmt::Display for StatementId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.pad(&self.0);
    }
}
