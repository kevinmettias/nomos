//! Reading the domain volumes off disk and putting them in.

use super::{Assembly, Path, BTreeMap, Absence, Expected, Ingest_Source_Document, Refused_Absence, Subject};

pub(super) fn Ingest_Volumes(assembly: &mut Assembly, root: &Path, revision: &str)
{
    use super::roots::DOMAIN_VOLUMES;

    let directory = root.join(DOMAIN_VOLUMES);
    let Some(documents) = Volumes_Under(assembly, &directory)
    else
    {
        return;
    };

    let blocks = Ingest_Each(assembly, &directory, &documents, revision);
    assembly.read.push(format!(
        "{} domain volume(s) at {revision} from {}, {blocks} block(s)",
        documents.len(),
        directory.display()
    ));
}

/// Every markdown document under the volumes directory, or nothing when there are none.
///
/// Sorted, because a store assembled in directory order is a store whose block uids depend
/// on the filesystem — and two runs on two machines would then disagree about what a
/// document is without either of them being wrong about the bytes.
pub(super) fn Volumes_Under(assembly: &mut Assembly, directory: &Path) -> Option<BTreeMap<String, String>>
{
    let Ok(entries) = std::fs::read_dir(directory)
    else
    {
        let unopened = Unreadable_Volumes(directory);
        assembly.absent.push(unopened);

        return None;
    };
    let Volumes {
        documents,
        unreadable,
    } = Read_Volumes(entries);
    Note_Unreadable(assembly, directory, &unreadable);
    if documents.is_empty()
    {
        let empty = Empty_Volumes(directory);
        assembly.absent.push(empty);

        return None;
    }

    return Some(documents);
}

/// What one directory yielded: the documents read, and the paths that would not open.
///
/// Named rather than a pair. Both members are collections of text keyed to a path, so a
/// caller that swapped them would be handed the wrong one by a signature that still
/// compiles.
pub(super) struct Volumes
{
    pub(super) documents: BTreeMap<String, String>,
    pub(super) unreadable: Vec<String>,
}

/// Every markdown document the directory yielded, and the paths that would not open.
pub(super) fn Read_Volumes(entries: std::fs::ReadDir) -> Volumes
{
    let mut documents: BTreeMap<String, String> = BTreeMap::new();
    let mut unreadable: Vec<String> = Vec::new();

    for entry in entries.flatten()
    {
        let path = entry.path();
        Read_Volume(&path, &mut documents, &mut unreadable);
    }

    return Volumes {
        documents,
        unreadable,
    };
}

/// One directory entry: a markdown document read, an unreadable one named, anything else
/// passed over.
pub(super) fn Read_Volume(
    path: &Path,
    documents: &mut BTreeMap<String, String>,
    unreadable: &mut Vec<String>,
)
{
    if path.extension().is_none_or(|extension| return extension != "md")
    {
        return;
    }

    match std::fs::read_to_string(path)
    {
        Ok(text) =>
        {
            let name = path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default()
                .to_owned();

            documents.insert(name, text);
        }
        Err(error) => unreadable.push(format!("{}: {error}", path.display())),
    }
}

/// A volumes directory that could not be opened at all.
pub(super) fn Unreadable_Volumes(directory: &Path) -> Absence
{
    return Absence {
        subject: "the domain volumes".to_owned(),
        expected: format!("{}/*.md", directory.display()),
        cause: "the directory could not be read".to_owned(),
        cost: "no corpus document is in this store, so no corpus table has rows to read"
            .to_owned(),
    };
}

/// A volumes directory holding nothing this build recognises.
pub(super) fn Empty_Volumes(directory: &Path) -> Absence
{
    return Absence {
        subject: "the domain volumes".to_owned(),
        expected: format!("at least one .md file under {}", directory.display()),
        cause: "the directory holds no markdown document".to_owned(),
        cost: "no corpus document is in this store, so no corpus table has rows to read"
            .to_owned(),
    };
}

/// The volumes that were there and would not open, named individually.
pub(super) fn Note_Unreadable(assembly: &mut Assembly, directory: &Path, unreadable: &[String])
{
    if unreadable.is_empty()
    {
        return;
    }

    assembly.absent.push(Absence {
        subject: format!("{} domain volume(s)", unreadable.len()),
        expected: format!("{}/*.md to be readable text", directory.display()),
        cause: unreadable.join("; "),
        cost: "those documents and their table rows are not in this store, so a count taken \
               over it is a count over what happened to open"
            .to_owned(),
    });
}

/// Puts every document into the store, and says how many blocks went in.
///
/// A document the store refuses becomes an absence and the walk continues, because one
/// unparseable volume is not a reason to report the other eight as missing too.
pub(super) fn Ingest_Each(
    assembly: &mut Assembly,
    directory: &Path,
    documents: &BTreeMap<String, String>,
    revision: &str,
) -> u32
{
    let mut blocks = 0_u32;

    for (name, markdown) in documents
    {
        match Ingest_Source_Document(&mut assembly.store, name, revision, markdown)
        {
            Ok(written) => blocks = blocks.saturating_add(written),
            Err(error) =>
            {
                let refusal = Refused_Absence(
                    Subject(&format!("the domain volume {name}")),
                    Expected(&directory.join(name).display().to_string()),
                    &error,
                    "that document and its rows are not in this store",
                );
                assembly.absent.push(refusal);
            }
        }
    }

    return blocks;
}

#[cfg(test)]
mod tests
{
    use super::{
        Assembly, BTreeMap, Empty_Volumes, Ingest_Each, Ingest_Volumes, Note_Unreadable,
        Read_Volume, Read_Volumes, Unreadable_Volumes, Volumes_Under,
    };
    use nomos_spec_store::SpecificationStore;

    fn Empty_Assembly() -> Assembly
    {
        return Assembly {
            store: SpecificationStore::In_Memory().expect("an in-memory store always opens"),
            read: Vec::new(),
            absent: Vec::new(),
        };
    }

    #[test]
    fn Test_Ingest_Volumes_Should_Record_An_Absence_When_The_Directory_Is_Missing()
    {
        let mut assembly = Empty_Assembly();
        let root = std::env::temp_dir().join("nomos-spec-orchestration-volumes-missing-root");
        let _ignored = std::fs::remove_dir_all(&root);

        Ingest_Volumes(&mut assembly, &root, "v14.36");

        assert_eq!(assembly.absent.len(), 1);
        assert!(assembly.read.is_empty());
    }

    #[test]
    fn Test_Ingest_Volumes_Should_Note_What_It_Read_When_A_Volume_Is_There()
    {
        let mut assembly = Empty_Assembly();
        let root = std::env::temp_dir().join("nomos-spec-orchestration-volumes-real-root");
        let directory = root.join(crate::corpus::roots::DOMAIN_VOLUMES);
        std::fs::create_dir_all(&directory).expect("creates");
        std::fs::write(directory.join("a.md"), "# A\n\n| C | M |\n| --- | --- |\n| x | y |\n").expect("writes");

        Ingest_Volumes(&mut assembly, &root, "v14.36");

        assert_eq!(assembly.absent.len(), 0, "{:?}", assembly.absent);
        assert_eq!(assembly.read.len(), 1);
        assert!(
            assembly.read.first().expect("the assertion above proves one line was read").contains("1 domain volume"),
            "{}",
            assembly.read.first().expect("the assertion above proves one line was read")
        );
    }

    #[test]
    fn Test_Volumes_Under_Should_Report_An_Absence_When_The_Directory_Cannot_Be_Opened()
    {
        let mut assembly = Empty_Assembly();
        let directory = std::env::temp_dir().join("nomos-spec-orchestration-volumes-under-missing");
        let _ignored = std::fs::remove_dir_all(&directory);

        let documents = Volumes_Under(&mut assembly, &directory);

        assert!(documents.is_none());
        assert_eq!(assembly.absent.len(), 1);
    }

    #[test]
    fn Test_Volumes_Under_Should_Report_An_Absence_When_The_Directory_Holds_No_Markdown()
    {
        let mut assembly = Empty_Assembly();
        let directory = std::env::temp_dir().join("nomos-spec-orchestration-volumes-under-empty");
        std::fs::create_dir_all(&directory).expect("creates");

        let documents = Volumes_Under(&mut assembly, &directory);

        assert!(documents.is_none());
        assert_eq!(assembly.absent.len(), 1);
    }

    #[test]
    fn Test_Volumes_Under_Should_Return_Every_Markdown_Document_When_The_Directory_Holds_One()
    {
        let mut assembly = Empty_Assembly();
        let directory = std::env::temp_dir().join("nomos-spec-orchestration-volumes-under-one");
        std::fs::create_dir_all(&directory).expect("creates");
        std::fs::write(directory.join("a.md"), "text").expect("writes");

        let documents = Volumes_Under(&mut assembly, &directory).expect("one markdown document");

        assert_eq!(documents.len(), 1);
        assert!(assembly.absent.is_empty());
    }

    #[test]
    fn Test_Read_Volumes_Should_Read_Every_Markdown_Entry_And_Name_The_Rest_Unreadable()
    {
        let directory = std::env::temp_dir().join("nomos-spec-orchestration-read-volumes");
        std::fs::create_dir_all(&directory).expect("creates");
        std::fs::write(directory.join("a.md"), "text a").expect("writes");
        std::fs::write(directory.join("notes.txt"), "ignored").expect("writes");
        let entries = std::fs::read_dir(&directory).expect("opens");

        let read = Read_Volumes(entries);

        assert_eq!(read.documents.len(), 1);
        assert_eq!(read.documents.get("a.md"), Some(&"text a".to_owned()));
        assert!(read.unreadable.is_empty());
    }

    #[test]
    fn Test_Read_Volume_Should_Insert_A_Markdown_File_Into_Documents()
    {
        let directory = std::env::temp_dir().join("nomos-spec-orchestration-read-volume-md");
        std::fs::create_dir_all(&directory).expect("creates");
        let path = directory.join("a.md");
        std::fs::write(&path, "text").expect("writes");
        let mut documents: BTreeMap<String, String> = BTreeMap::new();
        let mut unreadable: Vec<String> = Vec::new();

        Read_Volume(&path, &mut documents, &mut unreadable);

        assert_eq!(documents.get("a.md"), Some(&"text".to_owned()));
        assert!(unreadable.is_empty());
    }

    #[test]
    fn Test_Read_Volume_Should_Ignore_A_Non_Markdown_File()
    {
        let directory = std::env::temp_dir().join("nomos-spec-orchestration-read-volume-txt");
        std::fs::create_dir_all(&directory).expect("creates");
        let path = directory.join("notes.txt");
        std::fs::write(&path, "text").expect("writes");
        let mut documents: BTreeMap<String, String> = BTreeMap::new();
        let mut unreadable: Vec<String> = Vec::new();

        Read_Volume(&path, &mut documents, &mut unreadable);

        assert!(documents.is_empty());
        assert!(unreadable.is_empty());
    }

    #[test]
    fn Test_Unreadable_Volumes_Should_Name_The_Directory_That_Could_Not_Open()
    {
        let directory = std::path::PathBuf::from("no/such/volumes/directory");

        let absence = Unreadable_Volumes(&directory);

        assert_eq!(absence.subject, "the domain volumes");
        assert!(absence.cause.contains("could not be read"), "{}", absence.cause);
    }

    #[test]
    fn Test_Empty_Volumes_Should_Name_The_Directory_That_Held_No_Markdown()
    {
        let directory = std::path::PathBuf::from("some/volumes/directory");

        let absence = Empty_Volumes(&directory);

        assert_eq!(absence.subject, "the domain volumes");
        assert!(absence.cause.contains("no markdown document"), "{}", absence.cause);
    }

    #[test]
    fn Test_Note_Unreadable_Should_Do_Nothing_When_Nothing_Was_Unreadable()
    {
        let mut assembly = Empty_Assembly();
        let directory = std::path::PathBuf::from("some/volumes/directory");

        Note_Unreadable(&mut assembly, &directory, &[]);

        assert!(assembly.absent.is_empty());
    }

    #[test]
    fn Test_Note_Unreadable_Should_Record_One_Absence_Naming_Every_Unreadable_Path()
    {
        let mut assembly = Empty_Assembly();
        let directory = std::path::PathBuf::from("some/volumes/directory");
        let unreadable = vec!["a.md: permission denied".to_owned(), "b.md: not found".to_owned()];

        Note_Unreadable(&mut assembly, &directory, &unreadable);

        assert_eq!(assembly.absent.len(), 1);
        assert!(
            assembly.absent.first().expect("the assertion above proves one absence was recorded").cause.contains("a.md: permission denied"),
            "{}",
            assembly.absent.first().expect("the assertion above proves one absence was recorded").cause
        );
        assert!(
            assembly.absent.first().expect("the assertion above proves one absence was recorded").cause.contains("b.md: not found"),
            "{}",
            assembly.absent.first().expect("the assertion above proves one absence was recorded").cause
        );
    }

    #[test]
    fn Test_Ingest_Each_Should_Ingest_Every_Document_And_Report_Its_Block_Count()
    {
        let mut assembly = Empty_Assembly();
        let directory = std::path::PathBuf::from("some/volumes/directory");
        let mut documents: BTreeMap<String, String> = BTreeMap::new();
        documents.insert(
            "a.md".to_owned(),
            "# A\n\n| Concept | Meaning |\n| --- | --- |\n| Ledger | a claim |\n".to_owned(),
        );

        let blocks = Ingest_Each(&mut assembly, &directory, &documents, "v14.36");

        assert!(blocks > 0, "a document with a table should ingest at least one block");
        assert!(assembly.absent.is_empty(), "{:?}", assembly.absent);
    }
}
