//! Every way a fact store on disk refuses to be written, or to be read back and believed.
//!
//! Declared at the crate root beside [`crate::FactError`], for the same reason `lib.rs`
//! gives there: a file under `fact/` named for this type would repeat its parent, and one
//! named for the folder would declare a type it is not named for.
//!
//! These are refusals, not failures of a lookup. [`crate::FactError`] answers "this fact is
//! absent, superseded or backdated" — questions about one fact inside a store this process
//! built. This answers "the bytes on disk are not a store this build can serve", and every
//! variant names the file, because a refusal a reader cannot locate is a rumour.

use std::path::Path;
use std::path::PathBuf;
use nomos_contracts::Digest128;
use nomos_contracts::SchemaId;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistenceError
{
    Unwritable
    {
        file: PathBuf,
        detail: String,
    },
    Unreadable
    {
        file: PathBuf,
        detail: String,
    },
    Corrupt
    {
        file: PathBuf,
        detail: String,
    },
    ForeignFormat
    {
        file: PathBuf,
        written: u32,
        understood: u32,
    },
    ForeignKeyShape
    {
        file: PathBuf,
        written: Vec<String>,
        understood: Vec<String>,
    },
    ForeignKeyDigest
    {
        file: PathBuf,
        written: Digest128,
        recomputed: Digest128,
    },
    UnknownPayloadSchema
    {
        file: PathBuf,
        schema: SchemaId,
    },
}

impl PersistenceError
{
    /// The file the refusal is about.
    ///
    /// Every variant carries one and this returns it without a fallback, so a caller
    /// reporting a refusal cannot report one that names nothing.
    #[must_use]
    pub fn File(&self) -> &Path
    {
        return match self
        {
            Self::Unwritable { file, .. }
            | Self::Unreadable { file, .. }
            | Self::Corrupt { file, .. }
            | Self::ForeignFormat { file, .. }
            | Self::ForeignKeyShape { file, .. }
            | Self::ForeignKeyDigest { file, .. }
            | Self::UnknownPayloadSchema { file, .. } => file.as_path(),
        };
    }
}

impl core::fmt::Display for PersistenceError
{
    // Every arm is spelled out and there is no wildcard, so a variant added above fails to
    // compile here rather than reporting as nothing.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unwritable { file, detail } => write!(
                formatter,
                "{} could not be written: {detail}. Nothing was left behind for a later process to read as current",
                file.display()
            ),
            Self::Unreadable { file, detail } => write!(
                formatter,
                "{} could not be read: {detail}. A store that cannot be opened is not an empty one, and an empty one is what recomputing from nothing looks like",
                file.display()
            ),
            Self::Corrupt { file, detail } => write!(
                formatter,
                "{} is corrupt or truncated: {detail}. Nothing in it is served: a fact rescued out of damaged bytes is evidence about nothing",
                file.display()
            ),
            Self::ForeignFormat { file, written, understood } => write!(
                formatter,
                "{} was written in fact store format {written} and this build understands {understood}. The whole file is refused rather than partly read",
                file.display()
            ),
            Self::ForeignKeyShape { file, written, understood } => write!(
                formatter,
                "{} was written under fact key components [{}] and this build's are [{}]. A key whose components moved does not mean what its bytes say, so the whole file is refused rather than rescued entry by entry",
                file.display(),
                written.join(", "),
                understood.join(", ")
            ),
            Self::ForeignKeyDigest { file, written, recomputed } => write!(
                formatter,
                "{} carries a key written under digest {written} that this build addresses as {recomputed}. What the file says a fact is about and what this build would ask for are two different questions, so the whole file is refused",
                file.display()
            ),
            Self::UnknownPayloadSchema { file, schema } => write!(
                formatter,
                "{} carries a fact under payload schema {schema}, which this build does not understand. It is refused rather than handed to something that would read the bytes as a schema they were not written in",
                file.display()
            ),
        };
    }
}

impl std::error::Error for PersistenceError
{}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The file every sample refusal names.
    const SAMPLE_FILE: &str = "fact-store.json";

    /// The format version a sample file claims to have been written in, and the one this
    /// build claims to understand. They differ, because a refusal is what the pair is for.
    const WRITTEN_FORMAT: u32 = 1;
    const UNDERSTOOD_FORMAT: u32 = 2;

    /// The byte fill of the two digests a `ForeignKeyDigest` sample disagrees over.
    const WRITTEN_DIGEST_SEED: u8 = 1;
    const RECOMPUTED_DIGEST_SEED: u8 = 2;

    fn Every_Refusal() -> Vec<PersistenceError>
    {
        let file = PathBuf::from(SAMPLE_FILE);
        let detail = "no such file".to_owned();

        return vec![
            PersistenceError::Unwritable { file: file.clone(), detail: detail.clone() },
            PersistenceError::Unreadable { file: file.clone(), detail: detail.clone() },
            PersistenceError::Corrupt { file: file.clone(), detail },
            PersistenceError::ForeignFormat {
                file: file.clone(),
                written: WRITTEN_FORMAT,
                understood: UNDERSTOOD_FORMAT,
            },
            PersistenceError::ForeignKeyShape {
                file: file.clone(),
                written: vec!["subject".to_owned()],
                understood: vec!["contract".to_owned()],
            },
            PersistenceError::ForeignKeyDigest {
                file: file.clone(),
                written: Digest128::From_Bytes([WRITTEN_DIGEST_SEED; Digest128::BYTE_LENGTH]),
                recomputed: Digest128::From_Bytes([RECOMPUTED_DIGEST_SEED; Digest128::BYTE_LENGTH]),
            },
            PersistenceError::UnknownPayloadSchema {
                file,
                schema: SchemaId::New("nomos.test.unknown.v1"),
            },
        ];
    }

    #[test]
    fn Test_File_Should_Name_The_File_For_Every_Refusal()
    {
        for refusal in Every_Refusal()
        {
            assert_eq!(refusal.File(), Path::new(SAMPLE_FILE), "{refusal:?} named no file");
        }
    }

    #[test]
    fn Test_Display_Should_Name_The_File_For_Every_Refusal()
    {
        for refusal in Every_Refusal()
        {
            let said = refusal.to_string();

            assert!(said.contains(SAMPLE_FILE), "{refusal:?} reported as `{said}`, naming no file");
        }
    }
}
