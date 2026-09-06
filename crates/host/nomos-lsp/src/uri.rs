//! Converting between a `file://` `lsp_types::Uri` and a local path.
//!
//! `lsp-types` 0.97 carries its own `Uri` (a thin wrapper over `fluent_uri::Uri<String>`,
//! replacing the earlier `url::Url`) with no `to_file_path`/`from_file_path` of its own.
//! This is a small, deliberately narrow conversion over that type's own `as_str`/
//! `FromStr` -- correct for the ASCII, forward-slash, repo-relative paths this workspace's
//! own convention already produces (`Finding::locations`' own doc: "Repo-relative, forward
//! slashes"), not a general RFC 3986 implementation. A client naming a workspace root
//! outside that convention is not a case this crate has seen.

use lsp_types::Uri;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// The local path a `file://` URI names, or `None` when `uri` is not a `file://` URI.
#[must_use]
pub(crate) fn Path_From_File_Uri(uri: &Uri) -> Option<PathBuf>
{
    let rest = uri.as_str().strip_prefix("file://")?;
    let decoded = Percent_Decoded(rest);
    let path = Strip_Windows_Drive_Slash(&decoded);

    if path.is_empty()
    {
        return None;
    }

    return Some(PathBuf::from(path));
}

/// The `file://` URI naming `path`, or `None` when it cannot be rendered as one -- `path`
/// containing bytes that are not valid UTF-8, which none of this workspace's own repo-
/// relative paths ever do.
#[must_use]
pub(crate) fn File_Uri_From_Path(path: &Path) -> Option<Uri>
{
    let display = path.to_str()?.replace('\\', "/");
    let absolute = if display.starts_with('/') { display } else { format!("/{display}") };

    return Uri::from_str(&format!("file://{}", Percent_Encoded(&absolute))).ok();
}

/// `/C:/...` (the URI form of a Windows absolute path) becomes `C:/...`; every other path
/// is returned unchanged.
///
/// Reads the first three bytes through an iterator rather than by index: this workspace's
/// own `indexing_slicing` lint policy (`Cargo.toml`'s own `[workspace.lints.clippy]`) denies
/// a direct `path[..]`, since a byte count this function does not control could panic one.
fn Strip_Windows_Drive_Slash(path: &str) -> String
{
    let mut bytes = path.bytes();
    let is_drive_path = matches!(
        (bytes.next(), bytes.next(), bytes.next()),
        (Some(b'/'), Some(letter), Some(b':')) if letter.is_ascii_alphabetic()
    );

    if is_drive_path
    {
        return path.strip_prefix('/').unwrap_or(path).to_owned();
    }

    return path.to_owned();
}

/// A minimal `%XX` decoder -- every byte this crate ever encodes with [`Percent_Encoded`]
/// round-trips through it, which is the only property this conversion needs.
///
/// Walks `raw` one byte at a time through an iterator, cloning it to look two bytes ahead
/// only when the current one might start an escape, rather than an index variable this
/// workspace's own `indexing_slicing`/`arithmetic_side_effects` lint policy denies -- both
/// are `deny`, not `warn`, in `Cargo.toml`'s own `[workspace.lints.clippy]`.
fn Percent_Decoded(raw: &str) -> String
{
    let mut decoded = Vec::with_capacity(raw.len());
    let mut remaining = raw.bytes();

    while let Some(byte) = remaining.next()
    {
        if byte != b'%'
        {
            decoded.push(byte);
            continue;
        }

        let mut lookahead = remaining.clone();
        let escaped = lookahead
            .next()
            .zip(lookahead.next())
            .and_then(|(high, low)| return std::str::from_utf8(&[high, low]).ok().and_then(|hex| return u8::from_str_radix(hex, 16).ok()));

        match escaped
        {
            Some(value) =>
            {
                decoded.push(value);
                remaining = lookahead;
            }
            None => decoded.push(byte),
        }
    }

    return String::from_utf8_lossy(&decoded).into_owned();
}

/// Percent-encodes every byte outside a small unreserved set safe for a repo-relative
/// path: letters, digits, and `-._~/:`. Anything else -- a space is the case this
/// workspace's own paths actually hit -- is escaped as `%XX`.
fn Percent_Encoded(raw: &str) -> String
{
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(raw.len());

    for byte in raw.bytes()
    {
        let is_unreserved = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/' | b':');

        if is_unreserved
        {
            encoded.push(byte as char);
        }
        else
        {
            write!(encoded, "%{byte:02X}").expect("writing a fixed-width hex escape into a String cannot fail");
        }
    }

    return encoded;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_File_Uri_From_Path_Should_Round_Trip_A_Forward_Slash_Path()
    {
        let uri = File_Uri_From_Path(Path::new("/repo/a.rs")).expect("a plain path renders as a uri");

        let recovered = Path_From_File_Uri(&uri).expect("the uri this crate builds is one this crate reads back");
        assert_eq!(recovered, PathBuf::from("/repo/a.rs"));
    }

    #[test]
    fn Test_File_Uri_From_Path_Should_Round_Trip_A_Windows_Drive_Path()
    {
        let uri = File_Uri_From_Path(Path::new(r"C:\repo\a.rs")).expect("a windows path renders as a uri");

        assert!(uri.as_str().starts_with("file:///C:/"), "{}", uri.as_str());
        let recovered = Path_From_File_Uri(&uri).expect("the uri this crate builds is one this crate reads back");
        assert_eq!(recovered, PathBuf::from("C:/repo/a.rs"));
    }

    #[test]
    fn Test_File_Uri_From_Path_Should_Percent_Encode_A_Space()
    {
        let uri = File_Uri_From_Path(Path::new("/repo/a b.rs")).expect("a space is encoded, not rejected");

        assert!(uri.as_str().contains("%20"), "{}", uri.as_str());
        let recovered = Path_From_File_Uri(&uri).expect("round trip");
        assert_eq!(recovered, PathBuf::from("/repo/a b.rs"));
    }

    #[test]
    fn Test_Path_From_File_Uri_Should_Be_None_For_A_Non_File_Scheme()
    {
        let uri = Uri::from_str("https://example.invalid/a.rs").expect("a valid, non-file uri");

        assert!(Path_From_File_Uri(&uri).is_none());
    }
}
