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
        return Self::Of(&Normalize_Whitespace(text));
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
    pub fn As_String_Slice(&self) -> &str
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
    // `fmt` is the fixed method name `std::fmt::Display` mandates; it is not a free choice
    // of abbreviation and cannot be spelled out without ceasing to implement the trait.
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
pub fn Normalize_Whitespace(text: &str) -> String
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
    return Normalize_Whitespace(text) == text;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Normalize_Whitespace_Should_Collapse_Runs_And_Trim_Ends()
    {
        assert_eq!(Normalize_Whitespace("a  b\t\tc"), "a b c");
        assert_eq!(Normalize_Whitespace("a\nb"), "a b");
        assert_eq!(Normalize_Whitespace("  a  "), "a");
        assert_eq!(Normalize_Whitespace("a\r\n\r\nb"), "a b");
    }

    #[test]
    fn Test_Is_Normalized_Should_Recognize_Every_Normalized_Fixed_Point()
    {
        for text in Texts_Whose_Normalization_Should_Be_Idempotent()
        {
            let once = Normalize_Whitespace(text);
            assert_eq!(Normalize_Whitespace(&once), once);
            assert!(Is_Normalized(&once));
        }
    }

    /// Texts whose normalization should be a fixed point: normalizing the already-normalized
    /// result must change nothing, and [`Is_Normalized`] must agree.
    fn Texts_Whose_Normalization_Should_Be_Idempotent() -> [&'static str; 5]
    {
        return ["a  b", "\n\na\tb\n", "", "   ", "one"];
    }

    #[test]
    fn Test_Non_Ascii_Should_Survive()
    {
        assert_eq!(Normalize_Whitespace("a — b"), "a — b");
        assert_eq!(Normalize_Whitespace("Nomos’s  rule"), "Nomos’s rule");
    }

    #[test]
    fn Test_Of_Should_Produce_A_Pinned_Sha256_Digest()
    {
        assert_eq!(
            ContentHash::Of("# Nomos Domain-Owned Specification Suite").As_String_Slice(),
            "sha256:11fab7648eb13872bb5c78990b206a7bc3e25465a54d277edb730ca9eb72bd5d"
        );
    }

    #[test]
    fn Test_Of_Bytes_Should_Match_Of_For_The_Same_Content()
    {
        assert_eq!(ContentHash::Of_Bytes("same".as_bytes()), ContentHash::Of("same"));
    }

    #[test]
    fn Test_Of_Normalized_Should_Hash_The_Normalized_Text()
    {
        assert_eq!(ContentHash::Of_Normalized("a   b"), ContentHash::Of("a b"));
    }

    #[test]
    fn Test_As_String_Slice_Should_Return_The_Digest_Text_Verbatim()
    {
        let text = "sha256:11fab7648eb13872bb5c78990b206a7bc3e25465a54d277edb730ca9eb72bd5d";
        let hash = ContentHash::Parse(text).expect("valid");

        assert_eq!(hash.As_String_Slice(), text);
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
