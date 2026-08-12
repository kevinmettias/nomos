use sha2::{Digest, Sha256};

pub const HASH_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentHash(String);

impl ContentHash
{
    #[must_use]
    pub fn Of(text: &str) -> Self
    {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        return Self(format!("{HASH_PREFIX}{:x}", hasher.finalize()));
    }

    #[must_use]
    pub fn Of_Bytes(content: &[u8]) -> Self
    {
        let mut hasher = Sha256::new();
        hasher.update(content);
        return Self(format!("{HASH_PREFIX}{:x}", hasher.finalize()));
    }

    #[must_use]
    pub fn Of_Normalized(text: &str) -> Self
    {
        return Self::Of(&Normalize(text));
    }

    /// A SHA-256 digest spelled in lowercase hexadecimal.
    const DIGEST_HEX_LENGTH: usize = 64;

    #[must_use]
    pub fn Parse(text: &str) -> Option<Self>
    {
        let digest = text.strip_prefix(HASH_PREFIX)?;
        if digest.len() == Self::DIGEST_HEX_LENGTH && Is_Lowercase_Hex(digest)
        {
            return Some(Self(text.to_owned()));
        }
        return None;
    }

    #[must_use]
    pub fn As_Str(&self) -> &str
    {
        return &self.0;
    }
}

/// A run of ASCII hex digits with no upper case in it.
///
/// Case is part of the address rather than presentation: two spellings of one digest would
/// be two content addresses for one piece of content.
fn Is_Lowercase_Hex(text: &str) -> bool
{
    return text.bytes().all(|byte| return byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
}

impl core::fmt::Display for ContentHash
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.pad(&self.0);
    }
}

/// Collapses every run of whitespace to one space and trims the ends.
///
/// Recovered from the v14 corpus rather than chosen: it is the only function that
/// reproduces all 2533 recorded `normalized_hash` values, and it is also the function
/// every stored `canonical_text` is already a fixed point of.
#[must_use]
pub fn Normalize(text: &str) -> String
{
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;

    for character in text.chars()
    {
        if character.is_whitespace()
        {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space
        {
            out.push(' ');
            pending_space = false;
        }
        out.push(character);
    }

    return out;
}

#[must_use]
pub fn Is_Normalized(text: &str) -> bool
{
    return Normalize(text) == text;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Whitespace_Runs_Should_Collapse()
    {
        assert_eq!(Normalize("a  b\t\tc"), "a b c");
        assert_eq!(Normalize("a\nb"), "a b");
        assert_eq!(Normalize("  a  "), "a");
        assert_eq!(Normalize("a\r\n\r\nb"), "a b");
    }

    #[test]
    fn Test_Normalization_Should_Be_Idempotent()
    {
        for text in ["a  b", "\n\na\tb\n", "", "   ", "one"]
        {
            let once = Normalize(text);
            assert_eq!(Normalize(&once), once);
            assert!(Is_Normalized(&once));
        }
    }

    #[test]
    fn Test_Non_Ascii_Should_Survive()
    {
        assert_eq!(Normalize("a — b"), "a — b");
        assert_eq!(Normalize("Nomos’s  rule"), "Nomos’s rule");
    }

    #[test]
    fn Test_A_Known_Digest_Should_Be_Pinned()
    {
        assert_eq!(
            ContentHash::Of("# Nomos Domain-Owned Specification Suite").As_Str(),
            "sha256:11fab7648eb13872bb5c78990b206a7bc3e25465a54d277edb730ca9eb72bd5d"
        );
    }

    #[test]
    fn Test_Parse_Should_Refuse_Malformed_Hashes()
    {
        assert!(ContentHash::Parse("sha256:11fab7648eb13872bb5c78990b206a7bc3e25465a54d277edb730ca9eb72bd5d").is_some());
        assert!(ContentHash::Parse("11fab7648eb13872bb5c78990b206a7bc3e25465a54d277edb730ca9eb72bd5d").is_none());
        assert!(ContentHash::Parse("sha256:abc").is_none());
        assert!(ContentHash::Parse("md5:11fab7648eb13872bb5c78990b206a7bc3e25465a54d277edb730ca9eb72bd5d").is_none());
        assert!(ContentHash::Parse("sha256:11FAB7648EB13872BB5C78990B206A7BC3E25465A54D277EDB730CA9EB72BD5D").is_none());
    }
}
