//! Where a corpus is, and what its absence costs.

use super::{Assembly, CorpusRequest, Path, Absence};

/// The corpus inputs, relative to its root.
///
/// Named here rather than at each use so an absence can list what a corpus is expected to
/// hold without a reader having to find three separate string literals to learn it.
pub(super) const DOMAIN_VOLUMES: &str = "01_authoring/domain_volumes";

pub(super) const STATEMENTS: &str = "01_authoring/source_lineage/normative-source-statements.yaml";

pub(super) const CATALOG: &str = "02_machine/catalog/catalog.json";

/// The revision label the corpus is ingested under when nothing says otherwise.
///
/// `v14.36` is what this workspace's ingest tests pin, and a revision label is a claim
/// about which revision the bytes came from rather than a name for wherever the variable
/// happens to point. `--corpus-revision` exists so a differently-labelled tree can be
/// read without that claim becoming false.
pub const DEFAULT_REVISION: &str = "v14.36";

/// The corpus directory, or nothing and a recorded absence saying why there is none.
///
/// Unset and unreadable are two absences rather than one, because the reader's next move
/// differs: the first is a variable to set and the second is a path that is already wrong.
pub(super) fn Corpus_Root<'a>(assembly: &mut Assembly, request: &'a CorpusRequest) -> Option<&'a Path>
{
    let Some(root) = request.root.as_ref()
    else
    {
        let unnamed = Unnamed_Corpus(&request.variable);
        assembly.absent.push(unnamed);

        return None;
    };

    if !root.is_dir()
    {
        let unreadable = Unreadable_Corpus(root, &request.variable);
        assembly.absent.push(unreadable);

        return None;
    }

    return Some(root);
}

/// No corpus was named at all.
pub(super) fn Unnamed_Corpus(variable: &str) -> Absence
{
    return Absence {
        subject: "the v14 authoring corpus".to_owned(),
        expected: format!(
            "a directory named by {variable}, holding {DOMAIN_VOLUMES}/, {STATEMENTS} and \
             {CATALOG}"
        ),
        cause: format!("{variable} is not set, and no --corpus was given"),
        cost: "the corpus documents, their table rows, the normative statements and the node \
               catalog are not in this store. What this command can answer is what this \
               repository authors about itself, and nothing else"
            .to_owned(),
    };
}

/// A corpus was named and is not a directory this build can read.
pub(super) fn Unreadable_Corpus(root: &Path, variable: &str) -> Absence
{
    return Absence {
        subject: "the v14 authoring corpus".to_owned(),
        expected: format!("{} to be a directory", root.display()),
        cause: format!("{variable} does not name a readable directory"),
        cost: "the corpus documents, their table rows, the normative statements and the node \
               catalog are not in this store"
            .to_owned(),
    };
}
