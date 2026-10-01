//! [`BuildEvaluation`], what `MSBuild` answers about one build: its symbols and the files it
//! compiles.

use crate::DefinitionSet;
use std::collections::BTreeSet;

/// One build as `MSBuild` evaluated it: the definition set its compiler is handed, and every
/// file under the repository root it hands that compiler.
///
/// The second half is here because which build compiles a file is not a question a reading of
/// the file, or of the directory it sits in, can answer. An SDK-style project compiles every
/// `.cs` file beneath it by default -- including those beneath a nested project, which is why a
/// nearest-project guess is wrong -- and a project may turn that off, add files from outside its
/// directory, or remove some. `MSBuild`'s `Compile` items are the list the compiler is given, and
/// the same evaluation that answers the symbols answers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildEvaluation
{
    /// The symbols the build defines, and the build they are for.
    pub definitions: DefinitionSet,
    /// Every file the build compiles that sits under the repository root, as a path relative to
    /// it with forward slashes, spelled as `MSBuild` spelled it. A file the build compiles from
    /// outside the root is not listed: nothing under the root names it.
    pub compiled: BTreeSet<String>,
}

impl BuildEvaluation
{
    /// Whether the build compiles `path`, a path relative to the repository root with forward
    /// slashes.
    ///
    /// Compared without regard to ASCII case on Windows, where the file system does not
    /// distinguish it and a project may spell an explicit `Compile` item differently from the
    /// file on disk; exactly elsewhere.
    #[must_use]
    pub fn Compiles(&self, path: &str) -> bool
    {
        if cfg!(windows)
        {
            return self.compiled.iter().any(|compiled| return compiled.eq_ignore_ascii_case(path));
        }

        return self.compiled.contains(path);
    }
}
