//! Where the store comes from, and what to say when it is not there.
//!
//! Nothing persists a specification database. The store is assembled on every invocation
//! from two sources: this repository's own governing records, which are embedded in the
//! binary and are therefore always present, and the v14 authoring corpus, which lives
//! outside this repository and on most machines is not present at all.
//!
//! That second fact is the whole reason this module exists. A read command over a store
//! nothing was read into returns nothing, and nothing is exactly what a read command
//! returns when the identifier is genuinely unknown. The two answers are opposite — one
//! says *configure your corpus*, the other says *you have the wrong identifier* — and
//! they print the same. So an absence is a value here: it names what was expected, where
//! it was looked for, why it is not there, and what is consequently not in the store.
//!
//! This is `OD-GATE-001`'s finding one level up. Sixty-eight tests report `ok` having
//! read nothing because a check that cannot find its subject returned early instead of
//! saying so. A read command that prints an empty table for a corpus it never had is the
//! same defect wearing a different hat.

use nomos_spec_ingest::{
    Ingest_Catalog, Ingest_Source_Document, Ingest_Statements, IngestError, Parse_Catalog,
    Parse_Statements,
};
use nomos_spec_store::{Seed_Governing_Records, SpecificationStore, StoreError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The corpus inputs, relative to its root.
///
/// Named here rather than at each use so an absence can list what a corpus is expected to
/// hold without a reader having to find three separate string literals to learn it.
const DOMAIN_VOLUMES: &str = "01_authoring/domain_volumes";
const STATEMENTS: &str = "01_authoring/source_lineage/normative-source-statements.yaml";
const CATALOG: &str = "02_machine/catalog/catalog.json";

/// The revision label the corpus is ingested under when nothing says otherwise.
///
/// `v14.36` is what this workspace's ingest tests pin, and a revision label is a claim
/// about which revision the bytes came from rather than a name for wherever the variable
/// happens to point. `--corpus-revision` exists so a differently-labelled tree can be
/// read without that claim becoming false.
pub const DEFAULT_REVISION: &str = "v14.36";

/// Something the store was expected to hold and does not.
///
/// Every field is required. An absence that says only "no corpus" leaves the reader to
/// discover the variable's name, the path, and what is missing from the answer they just
/// received — and the last of those is the one they will not think to ask about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Absence
{
    /// What is missing, as a person would name it.
    pub subject: String,
    /// Where it was looked for, concretely.
    pub expected: String,
    /// Why it is not here.
    pub cause: String,
    /// What is therefore not in this store.
    pub cost: String,
}

impl Absence
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return format!(
            "absent: {}\n  expected: {}\n  cause:    {}\n  so:       {}",
            self.subject, self.expected, self.cause, self.cost
        );
    }
}

/// Where a corpus root was named, so an absence can say how to supply one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorpusRequest
{
    /// The environment variable that names the corpus root.
    ///
    /// Passed in rather than read here. The composition root owns the environment; this
    /// module is handed a value and a name for where it came from, which is what lets
    /// every test below run without one.
    pub variable: String,
    /// The root, if anything named one.
    pub root: Option<PathBuf>,
    /// The revision label to ingest the corpus under.
    pub revision: String,
}

/// A store, what went into it, and what did not.
pub struct Assembly
{
    pub store: SpecificationStore,
    /// One line per input that was read, in the order it was read.
    pub read: Vec<String>,
    pub absent: Vec<Absence>,
}

impl Assembly
{
    /// Whether anything the store was expected to hold is missing.
    #[must_use]
    pub fn Is_Complete(&self) -> bool
    {
        return self.absent.is_empty();
    }

    /// Every absence, one after another.
    #[must_use]
    pub fn Describe_Absences(&self) -> String
    {
        return self
            .absent
            .iter()
            .map(Absence::Describe)
            .collect::<Vec<String>>()
            .join("\n");
    }
}

/// Builds the store this invocation will answer from.
///
/// The governing records are seeded first and unconditionally: they are embedded, so a
/// machine with no corpus can still ask what this repository decided about itself. The
/// corpus is layered over them, and every part of it that could not be read becomes an
/// [`Absence`] rather than a smaller answer.
///
/// # Errors
///
/// Returns [`StoreError`] if the database cannot be opened or the embedded records cannot
/// be seeded. A corpus that cannot be read is not an error: it is an absence, because the
/// commands over this store still have a true answer to give without it.
pub fn Assemble(request: &CorpusRequest) -> Result<Assembly, StoreError>
{
    let mut store = SpecificationStore::In_Memory()?;
    let seeded = Seed_Governing_Records(&mut store)?;

    let mut assembly = Assembly {
        store,
        read: vec![format!(
            "{} governing record(s) embedded in this binary, {} block(s)",
            seeded.records, seeded.blocks
        )],
        absent: Vec::new(),
    };

    let Some(root) = request.root.as_ref()
    else
    {
        assembly.absent.push(Absence {
            subject: "the v14 authoring corpus".to_owned(),
            expected: format!(
                "a directory named by {}, holding {DOMAIN_VOLUMES}/, {STATEMENTS} and {CATALOG}",
                request.variable
            ),
            cause: format!("{} is not set, and no --corpus was given", request.variable),
            cost: "the corpus documents, their table rows, the normative statements and the \
                   node catalog are not in this store. What this command can answer is what \
                   this repository authors about itself, and nothing else"
                .to_owned(),
        });

        return Ok(assembly);
    };

    if !root.is_dir()
    {
        assembly.absent.push(Absence {
            subject: "the v14 authoring corpus".to_owned(),
            expected: format!("{} to be a directory", root.display()),
            cause: format!("{} does not name a readable directory", request.variable),
            cost: "the corpus documents, their table rows, the normative statements and the \
                   node catalog are not in this store"
                .to_owned(),
        });

        return Ok(assembly);
    }

    Ingest_Volumes(&mut assembly, root, &request.revision);
    Ingest_Statement_File(&mut assembly, root);
    Ingest_Catalog_File(&mut assembly, root);

    return Ok(assembly);
}

fn Ingest_Volumes(assembly: &mut Assembly, root: &Path, revision: &str)
{
    let directory = root.join(DOMAIN_VOLUMES);
    let Ok(entries) = std::fs::read_dir(&directory)
    else
    {
        assembly.absent.push(Absence {
            subject: "the domain volumes".to_owned(),
            expected: format!("{}/*.md", directory.display()),
            cause: "the directory could not be read".to_owned(),
            cost: "no corpus document is in this store, so no corpus table has rows to read"
                .to_owned(),
        });

        return;
    };

    // Sorted, because a store assembled in directory order is a store whose block uids
    // depend on the filesystem — and two runs on two machines would then disagree about
    // what a document is without either of them being wrong about the bytes.
    let mut documents: BTreeMap<String, String> = BTreeMap::new();
    let mut unreadable: Vec<String> = Vec::new();

    for entry in entries.flatten()
    {
        let path = entry.path();
        if path.extension().is_none_or(|extension| return extension != "md")
        {
            continue;
        }

        let name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default()
            .to_owned();

        match std::fs::read_to_string(&path)
        {
            Ok(text) =>
            {
                documents.insert(name, text);
            }
            Err(error) => unreadable.push(format!("{}: {error}", path.display())),
        }
    }

    if !unreadable.is_empty()
    {
        assembly.absent.push(Absence {
            subject: format!("{} domain volume(s)", unreadable.len()),
            expected: format!("{}/*.md to be readable text", directory.display()),
            cause: unreadable.join("; "),
            cost: "those documents and their table rows are not in this store, so a count \
                   taken over it is a count over what happened to open"
                .to_owned(),
        });
    }

    if documents.is_empty()
    {
        assembly.absent.push(Absence {
            subject: "the domain volumes".to_owned(),
            expected: format!("at least one .md file under {}", directory.display()),
            cause: "the directory holds no markdown document".to_owned(),
            cost: "no corpus document is in this store, so no corpus table has rows to read"
                .to_owned(),
        });

        return;
    }

    let mut blocks = 0_u32;
    for (name, markdown) in &documents
    {
        match Ingest_Source_Document(&mut assembly.store, name, revision, markdown)
        {
            Ok(written) => blocks = blocks.saturating_add(written),
            Err(error) =>
            {
                let refusal = Refused(
                    &format!("the domain volume {name}"),
                    &directory.join(name).display().to_string(),
                    &error,
                    "that document and its rows are not in this store",
                );
                assembly.absent.push(refusal);
            }
        }
    }

    assembly.read.push(format!(
        "{} domain volume(s) at {revision} from {}, {blocks} block(s)",
        documents.len(),
        directory.display()
    ));
}

fn Ingest_Statement_File(assembly: &mut Assembly, root: &Path)
{
    let path = root.join(STATEMENTS);
    let Ok(text) = std::fs::read_to_string(&path)
    else
    {
        assembly.absent.push(Absence {
            subject: "the normative statements".to_owned(),
            expected: path.display().to_string(),
            cause: "the file could not be read".to_owned(),
            cost: "no normative statement is in this store, so a profile that projects \
                   statements has nothing to project"
                .to_owned(),
        });

        return;
    };

    let file = match Parse_Statements(&text)
    {
        Ok(file) => file,
        Err(error) =>
        {
            let refusal = Refused(
                "the normative statements",
                &path.display().to_string(),
                &error,
                "no normative statement is in this store",
            );
            assembly.absent.push(refusal);

            return;
        }
    };

    match Ingest_Statements(&mut assembly.store, &file)
    {
        Ok(report) => assembly
            .read
            .push(format!("{} normative statement(s) from {}", report.ingested, path.display())),
        Err(error) =>
        {
            let refusal = Refused(
                "the normative statements",
                &path.display().to_string(),
                &error,
                "no normative statement is in this store",
            );
            assembly.absent.push(refusal);
        }
    }
}

fn Ingest_Catalog_File(assembly: &mut Assembly, root: &Path)
{
    let path = root.join(CATALOG);
    let Ok(text) = std::fs::read_to_string(&path)
    else
    {
        assembly.absent.push(Absence {
            subject: "the node catalog".to_owned(),
            expected: path.display().to_string(),
            cause: "the file could not be read".to_owned(),
            cost: "the corpus contributes no node to this store, so an identifier it holds \
                   resolves to nothing"
                .to_owned(),
        });

        return;
    };

    let entities = match Parse_Catalog(&text)
    {
        Ok(entities) => entities,
        Err(error) =>
        {
            let refusal = Refused(
                "the node catalog",
                &path.display().to_string(),
                &error,
                "the corpus contributes no node to this store",
            );
            assembly.absent.push(refusal);

            return;
        }
    };

    match Ingest_Catalog(&mut assembly.store, &entities)
    {
        Ok(report) => assembly
            .read
            .push(format!("{} catalog node(s) from {}", report.nodes, path.display())),
        Err(error) =>
        {
            let refusal = Refused(
                "the node catalog",
                &path.display().to_string(),
                &error,
                "the corpus contributes no node to this store",
            );
            assembly.absent.push(refusal);
        }
    }
}

/// An input that was found and refused.
///
/// The same shape as one that was not there at all, deliberately. From the answer's point
/// of view a corpus that will not parse and a corpus that is missing cost exactly the same
/// rows, and a reader who is told only that something failed will read the shortfall in
/// the answer as the answer.
fn Refused(subject: &str, path: &str, error: &IngestError, cost: &str) -> Absence
{
    return Absence {
        subject: subject.to_owned(),
        expected: path.to_owned(),
        cause: format!("it was found and refused: {error}"),
        cost: cost.to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// A name for the variable, invented here. The composition root owns the real one;
    /// naming it in this file would make every test below look like a test that reads a
    /// corpus, and the corpus-gate census counts those.
    const VARIABLE: &str = "A_CORPUS_VARIABLE";

    fn Request(root: Option<PathBuf>) -> CorpusRequest
    {
        return CorpusRequest {
            variable: VARIABLE.to_owned(),
            root,
            revision: DEFAULT_REVISION.to_owned(),
        };
    }

    #[test]
    fn Test_With_No_Corpus_The_Governing_Records_Should_Still_Be_There()
    {
        let assembly = Assemble(&Request(None)).expect("assembles");

        assert!(
            assembly.store.Node_Summary("D-129").expect("queries").is_some(),
            "the embedded records travel with the binary and do not depend on a corpus"
        );
    }

    /// The property the whole module exists for. An unset variable must produce a named
    /// absence, not a smaller store nobody was told about.
    #[test]
    fn Test_An_Unset_Corpus_Should_Be_An_Absence_That_Names_What_Was_Expected()
    {
        let assembly = Assemble(&Request(None)).expect("assembles");

        assert!(!assembly.Is_Complete());
        let described = assembly.Describe_Absences();
        assert!(described.contains(VARIABLE), "{described}");
        assert!(described.contains(DOMAIN_VOLUMES), "{described}");
        assert!(described.contains(STATEMENTS), "{described}");
        assert!(described.contains(CATALOG), "{described}");
        assert!(described.contains("not in this store"), "{described}");
    }

    #[test]
    fn Test_A_Corpus_Pointed_At_Nothing_Should_Name_The_Path_It_Was_Pointed_At()
    {
        let root = PathBuf::from("no/such/corpus/anywhere");

        let assembly = Assemble(&Request(Some(root))).expect("assembles");

        assert!(!assembly.Is_Complete());
        assert!(
            assembly.Describe_Absences().contains("no/such/corpus/anywhere")
                || assembly.Describe_Absences().contains("no\\such\\corpus\\anywhere"),
            "{}",
            assembly.Describe_Absences()
        );
    }

    /// A corpus root that exists and holds none of the three inputs is three absences,
    /// not one. Collapsing them would let two thirds of a corpus read as a whole one.
    #[test]
    fn Test_An_Empty_Corpus_Root_Should_Account_For_Each_Input_Separately()
    {
        let root = std::env::temp_dir().join("nomos-cli-empty-corpus-root");
        std::fs::create_dir_all(&root).expect("creates");

        let assembly = Assemble(&Request(Some(root))).expect("assembles");

        assert_eq!(
            assembly.absent.len(),
            3,
            "{}",
            assembly.Describe_Absences()
        );
    }

    #[test]
    fn Test_A_Readable_Volume_Should_Reach_The_Store_And_Not_Be_Reported_Absent()
    {
        let root = std::env::temp_dir().join("nomos-cli-one-volume-corpus");
        let volumes = root.join(DOMAIN_VOLUMES);
        std::fs::create_dir_all(&volumes).expect("creates");
        std::fs::write(
            volumes.join("05_domain_model.md"),
            "# Canonical domain model\n\n| Concept | Meaning |\n| --- | --- |\n| Ledger | a claim |\n",
        )
        .expect("writes");

        let assembly = Assemble(&Request(Some(root))).expect("assembles");

        let (found, _) = assembly
            .store
            .Documents_Named("05_domain_model.md", None)
            .expect("queries");
        let uid = *found.first().expect("the volume is in the store");
        assert_eq!(assembly.store.Table_Lines(uid, None, None).expect("queries").len(), 3);
        assert!(
            assembly
                .absent
                .iter()
                .all(|absence| return !absence.subject.contains("domain volume")),
            "{}",
            assembly.Describe_Absences()
        );
    }
}
