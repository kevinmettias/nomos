use crate::archive_error::ArchiveError;
use crate::listing::Listing;
use std::io::Read as _;
use std::path::{Path, PathBuf};

/// A zip file's index, read without unpacking anything.
///
/// Both failures answer with the archive path rather than the underlying message alone,
/// because a caller walking twenty archives cannot tell from "invalid zip" which one it was.
fn Opened(path: &Path) -> Result<zip::ZipArchive<std::io::BufReader<std::fs::File>>, ArchiveError>
{
    let file = std::fs::File::open(path).map_err(|error| ArchiveError::Unreadable {
        archive: path.to_path_buf(),
        cause: error.to_string(),
    })?;

    return zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|error| {
        return ArchiveError::Unreadable {
            archive: path.to_path_buf(),
            cause: error.to_string(),
        };
    });
}

/// The files an archive holds, sorted so an iteration order never depends on how the
/// archive was written.
fn Entries(inner: &zip::ZipArchive<std::io::BufReader<std::fs::File>>) -> Vec<String>
{
    let mut paths: Vec<String> = inner
        .file_names()
        .filter(|name| !name.ends_with('/'))
        .map(str::to_owned)
        .collect();
    paths.sort();

    return paths;
}

/// One versioned archive, read in place.
///
/// Nothing is unpacked. Keeping the revisions inside their archives is the same barrier
/// that stops generated output becoming source: an extracted tree sitting in the working
/// directory is a thing a later ingest can mistake for authored input.
pub struct Archive
{
    path: PathBuf,
    inner: zip::ZipArchive<std::io::BufReader<std::fs::File>>,
    listing: Listing,
}

impl Archive
{
    /// # Errors
    ///
    /// Returns [`ArchiveError::Unreadable`] if the file is absent or is not a zip, and
    /// [`ArchiveError::Empty`] if it holds no files.
    pub fn Open(path: &Path) -> Result<Self, ArchiveError>
    {
        let inner = Opened(path)?;
        let paths = Entries(&inner);

        if paths.is_empty()
        {
            return Err(ArchiveError::Empty {
                archive: path.to_path_buf(),
            });
        }

        return Ok(Self {
            path: path.to_path_buf(),
            inner,
            listing: Listing::Of(paths),
        });
    }

    #[must_use]
    pub fn Path(&self) -> &Path
    {
        return &self.path;
    }

    /// What the archive holds, without opening any of it.
    #[must_use]
    pub const fn Listing(&self) -> &Listing
    {
        return &self.listing;
    }

    /// # Errors
    ///
    /// Returns [`ArchiveError::NoSuchEntry`] if the archive has no such file.
    pub fn Read(&mut self, entry: &str) -> Result<Vec<u8>, ArchiveError>
    {
        let mut file = self
            .inner
            .by_name(entry)
            .map_err(|_| return ArchiveError::NoSuchEntry {
                archive: self.path.clone(),
                entry: entry.to_owned(),
            })?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| ArchiveError::Unreadable {
                archive: self.path.clone(),
                cause: format!("{entry}: {error}"),
            })?;

        return Ok(bytes);
    }

    /// The bytes as UTF-8, left exactly as authored.
    ///
    /// A byte order mark is not stripped here. `Segment` consumes one as part of the
    /// front matter fence (D-131), and stripping it twice in two places is how the two
    /// come to disagree about what a document's first bytes were.
    ///
    /// # Errors
    ///
    /// Returns [`ArchiveError::NoSuchEntry`] or [`ArchiveError::NotText`].
    pub fn Read_Text(&mut self, entry: &str) -> Result<String, ArchiveError>
    {
        let bytes = self.Read(entry)?;

        return String::from_utf8(bytes).map_err(|error| ArchiveError::NotText {
            archive: self.path.clone(),
            entry: entry.to_owned(),
            cause: error.to_string(),
        });
    }
}

/// Every archive in a directory, sorted by path.
///
/// # Errors
///
/// Returns [`ArchiveError::Unreadable`] if the directory cannot be listed. A directory
/// holding no archives is an error for the same reason an empty archive is.
pub fn Archives_In(directory: &Path) -> Result<Vec<PathBuf>, ArchiveError>
{
    let entries = std::fs::read_dir(directory).map_err(|error| ArchiveError::Unreadable {
        archive: directory.to_path_buf(),
        cause: error.to_string(),
    })?;

    let mut archives: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("zip"))
        .collect();
    archives.sort();

    if archives.is_empty()
    {
        return Err(ArchiveError::Empty {
            archive: directory.to_path_buf(),
        });
    }

    return Ok(archives);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Discards the archive so a refusal can be asserted on. `Archive` is not `Debug`,
    /// and deriving it purely for `expect_err` would put a zip reader's internals into a
    /// public trait impl.
    fn Refusal(path: &str) -> ArchiveError
    {
        return match Archive::Open(Path::new(path))
        {
            Ok(_) => panic!("{path} should have been refused"),
            Err(error) => error,
        };
    }

    #[test]
    fn Test_A_Missing_Archive_Should_Name_Itself()
    {
        let refusal = Refusal("no-such-file.zip");

        assert!(matches!(refusal, ArchiveError::Unreadable { .. }), "{refusal}");
        assert!(
            refusal.to_string().contains("no-such-file.zip"),
            "the error does not say which archive: {refusal}"
        );
    }

    #[test]
    fn Test_A_File_That_Is_Not_An_Archive_Should_Be_Refused()
    {
        let refusal = Refusal("Cargo.toml");

        assert!(matches!(refusal, ArchiveError::Unreadable { .. }), "{refusal}");
    }

    #[test]
    fn Test_A_Directory_With_No_Archives_Should_Be_Refused()
    {
        let refusal = Archives_In(Path::new("src")).expect_err("must refuse");

        assert!(matches!(refusal, ArchiveError::Empty { .. }), "{refusal}");
    }

    #[test]
    fn Test_A_Missing_Directory_Should_Be_Refused()
    {
        let refusal = Archives_In(Path::new("no-such-directory")).expect_err("must refuse");

        assert!(matches!(refusal, ArchiveError::Unreadable { .. }), "{refusal}");
    }
}
