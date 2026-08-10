//! Reading the domain volumes off disk and putting them in.

use super::{Assembly, Path, BTreeMap, Absence, Ingest_Source_Document, Refused};
use super::roots::DOMAIN_VOLUMES;

pub(super) fn Ingest_Volumes(assembly: &mut Assembly, root: &Path, revision: &str)
{
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

    let (documents, unreadable) = Read_Volumes(entries);
    Note_Unreadable(assembly, directory, &unreadable);
    if documents.is_empty()
    {
        let empty = Empty_Volumes(directory);
        assembly.absent.push(empty);

        return None;
    }

    return Some(documents);
}

/// Every markdown document the directory yielded, and the paths that would not open.
pub(super) fn Read_Volumes(entries: std::fs::ReadDir) -> (BTreeMap<String, String>, Vec<String>)
{
    let mut documents: BTreeMap<String, String> = BTreeMap::new();
    let mut unreadable: Vec<String> = Vec::new();

    for entry in entries.flatten()
    {
        let path = entry.path();
        Read_Volume(&path, &mut documents, &mut unreadable);
    }

    return (documents, unreadable);
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

    let name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_owned();

    match std::fs::read_to_string(path)
    {
        Ok(text) =>
        {
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

    return blocks;
}
