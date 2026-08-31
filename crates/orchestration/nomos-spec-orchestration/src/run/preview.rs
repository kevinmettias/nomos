//! Resolving `nomos spec preview`'s request against an assembled store: reading the staged
//! text and asking the store what committing it would change.
//!
//! # Why this is generic over [`FileSystem`]
//!
//! `--from` names a single file at a path an author chose, not one this crate already knows
//! the way [`crate::run::render`]'s output path is known from a profile's own declaration --
//! the "different shape" this crate's own top-level documentation flagged as increment 3's
//! open question. It is still exactly [`FileSystem::Read_To_String`]'s shape regardless of
//! who chose the path: one file, read whole, and the port does not care which caller named
//! it. Going through it keeps this crate's whole I/O boundary uniform -- a test can hand
//! `Preview` a fake `FileSystem` and exercise `--from` missing without touching a real disk,
//! the same benefit `Render`'s own tests already get for the write half.

use nomos_platform::FileSystem;
use nomos_spec_store::EditPreview;
use std::path::Path;

use crate::corpus::Assembly;
use crate::spec_outcome::PreviewRefusal;
use crate::request::EditRequest;

/// What committing `request` would change, without writing anything.
///
/// Moved from `nomos-cli::spec::verb::editing`'s own `Preview`, `Staged_Text` and
/// `Previewed`, minus the writing: only the preview itself is built here.
///
/// # Errors
///
/// Returns [`PreviewRefusal::Unreadable`] when `request.from` could not be read, and
/// [`PreviewRefusal::Edit`] when the store refuses the staged edit.
pub fn Preview_Staged_Edit<Filesystem: FileSystem>(
    assembly: &Assembly,
    request: &EditRequest,
    filesystem: &Filesystem,
) -> Result<EditPreview, PreviewRefusal>
{
    let staged = Staged_Text(&request.from, filesystem)?;

    return Preview_Of_Staged_Text(assembly, request, &staged);
}

/// The bytes an author staged at `from`, read through `filesystem`.
fn Staged_Text<Filesystem: FileSystem>(from: &Path, filesystem: &Filesystem) -> Result<String, PreviewRefusal>
{
    return filesystem
        .Read_To_String(from)
        .map_err(|error| return PreviewRefusal::Unreadable { path: from.to_owned(), error });
}

/// The staged text, checked against the store and turned into a preview.
fn Preview_Of_Staged_Text(assembly: &Assembly, request: &EditRequest, staged: &str) -> Result<EditPreview, PreviewRefusal>
{
    let rename = request.rename.as_deref();

    return assembly
        .store
        .Claim_For_Edit(&request.id, None)
        .and_then(|claimed| return claimed.Stage(staged, rename))
        .and_then(|edit| return edit.Preview(&assembly.store))
        .map_err(PreviewRefusal::Edit);
}

#[cfg(test)]
mod tests
{
    use super::{EditRequest, PreviewRefusal, Preview_Staged_Edit};
    use crate::corpus::{Assemble_Corpus, Assembly, CorpusRequest};
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    #[test]
    fn Test_Preview_Staged_Edit_Should_Refuse_A_Staged_File_That_Cannot_Be_Read()
    {
        let assembly = Assembled();
        let missing = PathBuf::from("no-such-staged-file-anywhere.md");

        let error = Preview_Staged_Edit(&assembly, &EditRequest { id: "D-132".to_owned(), from: missing.clone(), rename: None }, &StdFileSystem)
            .expect_err("a --from naming nothing must refuse");

        assert!(matches!(&error, PreviewRefusal::Unreadable { path, .. } if *path == missing), "{error:?}");
    }

    #[test]
    fn Test_Preview_Staged_Edit_Should_Describe_A_Canonical_Heading_Rename()
    {
        let assembly = Assembled();
        let markdown = crate::run::Rendered_Markdown(&assembly, &crate::request::RecordRequest { id: "D-132".to_owned(), revision: None })
            .expect("D-132 is embedded")
            .markdown;
        let edited = markdown.replace("## Decision", "## The decision");
        let staged = Scratch("colocated").join("staged.md");
        std::fs::write(&staged, &edited).expect("writes the staged edit");

        let preview = Preview_Staged_Edit(&assembly, &EditRequest { id: "D-132".to_owned(), from: staged, rename: None }, &StdFileSystem)
            .expect("a canonical heading rename previews cleanly");

        assert!(preview.Is_Wording_Moved(), "a heading rename must count as wording moved");
    }

    fn Scratch(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(format!("nomos-spec-orchestration-preview-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch build root");
        return root;
    }

    fn Assembled() -> Assembly
    {
        let request = CorpusRequest { variable: "A_PREVIEW_TEST_CORPUS_VARIABLE".to_owned(), root: None, revision: "v14.36".to_owned() };
        return Assemble_Corpus(&request).expect("assembles from the embedded records alone");
    }
}
