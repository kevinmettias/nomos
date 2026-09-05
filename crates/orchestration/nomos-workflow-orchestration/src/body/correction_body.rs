//! What a correction-body step names, before `nomos-correction-orchestration::Run_Correction`
//! ever sees it.

use nomos_rules::SourceFile;
use std::path::PathBuf;

/// The tree a correction body judges and, if a blocking claim from either correction
/// family matches, stages, validates and optionally commits a fix for -- already walked,
/// the identical reason [`crate::CheckBody`] carries `sources` rather than a root to walk
/// itself: this crate has no `nomos_platform::FileSystem` port to walk a directory
/// through, so a workflow step's own author assembles `sources` before building this
/// body.
///
/// `root` is carried alongside `sources` for the same reason [`crate::CheckBody::root`]
/// is: `Run_Correction` files the corrected content back through
/// `nomos_correction_orchestration::CorrectionCommand::root`, and a workspace state needs
/// a real path to be seeded and diffed against even though `sources` itself is already in
/// memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionBody
{
    /// The tree this correction runs against, and where a committed fix is written.
    pub root: PathBuf,
    /// The already-walked source this correction judges.
    pub sources: Vec<SourceFile>,
    /// Whether to actually commit and write the corrected file, or stop after staging and
    /// validating it -- `nomos_correction_orchestration::CorrectionCommand::commit`'s own
    /// field, carried here under the identical name.
    pub commit: bool,
}

impl CorrectionBody
{
    /// Constructs a correction body.
    #[must_use]
    pub fn New(root: PathBuf, sources: Vec<SourceFile>, commit: bool) -> Self
    {
        return Self { root, sources, commit };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Bodies_With_Equal_Content_Should_Be_Equal()
    {
        let one = CorrectionBody::New(PathBuf::from("."), Vec::new(), false);
        let other = CorrectionBody::New(PathBuf::from("."), Vec::new(), false);

        assert_eq!(one, other);
    }

    #[test]
    fn Test_A_Different_Commit_Flag_Should_Change_Equality()
    {
        let staged_only = CorrectionBody::New(PathBuf::from("."), Vec::new(), false);
        let committing = CorrectionBody::New(PathBuf::from("."), Vec::new(), true);

        assert_ne!(staged_only, committing);
    }
}
