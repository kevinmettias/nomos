//! What a wave-scheduled correction run was asked to do, and what it runs through.

use nomos_platform::FileSystem;
use nomos_workspace::BuildVariant;
use std::path::Path;

/// The root, build variant and filesystem [`crate::Run_Correction_Waves`] needs but does
/// not compute, and whether it may write.
///
/// The same grouping [`crate::CorrectionEnvironment`] is for [`crate::Run_Correction`],
/// minus the launcher and the environment: this entry point is handed findings somebody
/// else already judged, so it never runs a provider and has nothing to run one through.
/// `OD-HOST-001` still holds for the two it does keep -- the composition root chooses the
/// filesystem and the build variant, and this crate neither picks a platform nor decides
/// what is being built.
pub struct WaveScheduling<'a, Fs: FileSystem>
{
    /// The tree the findings were judged over, and the tree a committed correction is
    /// written into.
    pub root: &'a Path,
    /// The variant the run's own workspace is opened under. Nothing here computes one.
    pub variant: BuildVariant,
    /// The filesystem every candidate reads through and every commit writes through.
    pub filesystem: &'a Fs,
    /// Whether to commit each validated wave and write its corrected files, or stop after
    /// staging and validating every wave.
    pub commit: bool,
}
