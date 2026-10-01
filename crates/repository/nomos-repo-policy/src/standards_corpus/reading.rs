//! Reading `nomos-standards-corpus.json` and the documents it points at into a repository's
//! declared standards corpus.
//!
//! # Why its own file, and not `standards.json`
//!
//! The `OD-RULES-011` modules beside this one read `standards.json`, and this one cannot. That
//! file is shared: `code-standards` decodes the whole of it into one struct with unknown fields
//! disallowed, and a key nomos would own outright is not available in it. `OD-HOST-009` settled
//! where a repository-declared criterion without a `code-standards` key travels: a dedicated,
//! language-neutral file at the repository root, the convention `nomos-gate.json`,
//! `nomos-architecture.json` and `nomos-test-material.json` already set.
//!
//! # What this reads, and what reading means here
//!
//! A corpus is markdown, and the only machine-readable thing in one is the delimiter-fenced
//! block at the top of each document — the format `xvpe/docs/arch/standards/rule.schema.json`
//! declares. Everything below the fence is prose and is not read at all: a document's body is
//! not an input to this reader, which is why the guarantee this provider offers is
//! [`nomos_contracts::FactVariant::Syntactic`] rather than a resolved one.
//!
//! Three outcomes are carried for every document, and they are three rather than two on
//! purpose. A document that opens no fence declared nothing, and the schema makes no demand
//! of it. A document that opens a fence and never closes it, or that names a kind, a severity
//! or a gate outside the corpus's own closed vocabulary, wrote a declaration this workspace
//! cannot read — and that is recorded as an issue rather than as an absence, because an
//! absence and a defect must not arrive at the rule as the same fact. The third is a
//! declaration read whole, which is the only path by which a document becomes a rule.
//!
//! # The population is every document on disk
//!
//! Every markdown file under every declared root gets a [`DocumentDeclaration`] row, whatever
//! it declared. The subset declaring `kind: rule` additionally gets a [`DeclaredRule`] row or
//! an [`DeclarationIssue`] row, and a reader that dropped a document it could not read would
//! report a smaller corpus that looks clean — the failure this capability exists to prevent.

use nomos_cap_standards_corpus::{
    DeclarationIssue, DeclarationKind, Declared_Gate, DeclaredRule, DocumentDeclaration, RuleSeverity, StandardsCorpusPayload,
};
use nomos_platform::{FileSystem, FileSystemError};
use std::path::{Path, PathBuf};

/// The file a repository declares the standards corpora it owns in.
///
/// `nomos-<concern>.json` at the repository root, the convention `nomos-gate.json`,
/// `nomos-architecture.json` and `nomos-test-material.json` already set for a file this
/// workspace owns outright, as against `standards.json`, which it shares.
pub const STANDARDS_CORPUS_JSON: &str = "nomos-standards-corpus.json";

/// The key naming the repository-relative roots the population is read from.
const ROOTS_KEY: &str = "corpus_roots";

/// The line that opens and closes a document's declaration.
const FENCE: &str = "---";

/// The suffix a standards document carries.
const DOCUMENT_SUFFIX: &str = ".md";

/// The one key every document's declaration carries, and the one that decides whether the
/// document is a rule.
const KIND_KEY: &str = "kind";

/// The four keys a declaration must carry on top of `kind` to be a rule.
const ID_KEY: &str = "id";
const TITLE_KEY: &str = "title";
const SEVERITY_KEY: &str = "severity";
const GATE_KEY: &str = "gate";
const ENFORCED_BY_KEY: &str = "enforced_by";

/// `nomos-standards-corpus.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandardsCorpusError
{
    pub reason: String,
}

impl core::fmt::Display for StandardsCorpusError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Every markdown document under every root `root`'s own `nomos-standards-corpus.json`
/// declares, with whatever each one declares about itself — declaring nothing when the file is
/// absent or names no `corpus_roots` key, since an unconfigured repository is not a repository
/// this capability failed to read.
///
/// # Errors
///
/// [`StandardsCorpusError`] if `nomos-standards-corpus.json` exists but could not be read for a
/// reason other than absence, is not valid JSON, or declares `corpus_roots` as something other
/// than an array of non-empty strings.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<StandardsCorpusPayload, StandardsCorpusError>
{
    let path = root.join(STANDARDS_CORPUS_JSON);
    let text = match filesystem.Read_To_String(&path)
    {
        Ok(text) => text,
        // An absent file is a repository that has not declared a corpus, which is a real answer
        // rather than a read this provider failed at.
        Err(FileSystemError::NotFound { .. }) => return Ok(StandardsCorpusPayload::default()),
        Err(error) => return Err(StandardsCorpusError { reason: format!("{STANDARDS_CORPUS_JSON} could not be read: {error}") }),
    };

    let declared: serde_json::Value = serde_json::from_str(&text).map_err(|error| return StandardsCorpusError {
        reason: format!("{STANDARDS_CORPUS_JSON} is not valid JSON: {error}"),
    })?;

    return Ok(Read_Corpus(root, &Declared_Roots(&declared)?, filesystem));
}

/// The roots `declared` names, sorted and deduplicated.
fn Declared_Roots(declared: &serde_json::Value) -> Result<Vec<String>, StandardsCorpusError>
{
    let Some(listed) = declared.get(ROOTS_KEY)
    else
    {
        return Ok(Vec::new());
    };

    let Some(entries) = listed.as_array()
    else
    {
        return Err(StandardsCorpusError { reason: format!("{STANDARDS_CORPUS_JSON}'s {ROOTS_KEY} is not an array") });
    };

    let mut roots = Vec::new();
    for entry in entries
    {
        roots.push(Root_Of(entry)?);
    }

    // Sorted and deduplicated: two declarations naming one root are one corpus, and the
    // population must not depend on the order a JSON array happened to list them in.
    roots.sort();
    roots.dedup();

    return Ok(roots);
}

/// One declared root, normalized to a repository-relative path with forward slashes.
fn Root_Of(entry: &serde_json::Value) -> Result<String, StandardsCorpusError>
{
    let Some(declared) = entry.as_str()
    else
    {
        return Err(StandardsCorpusError { reason: format!("{STANDARDS_CORPUS_JSON}'s {ROOTS_KEY} has a non-string entry") });
    };

    let normalized = declared.trim().trim_matches('/').to_owned();
    if normalized.is_empty()
    {
        return Err(StandardsCorpusError { reason: format!("{STANDARDS_CORPUS_JSON}'s {ROOTS_KEY} names an empty root") });
    }

    return Ok(normalized);
}

/// Walks every declared root, filling a payload with the population it holds.
///
/// Infallible, and deliberately not wrapped in a `Result`: a root that could not be walked is a
/// row in `issues` rather than a failure of the read. The only question this provider can fail
/// is whether `nomos-standards-corpus.json` itself parses, which [`Discover_Workspace`] answers
/// before reaching here.
fn Read_Corpus<Fs: FileSystem>(root: &Path, roots: &[String], filesystem: &Fs) -> StandardsCorpusPayload
{
    let mut payload = StandardsCorpusPayload {
        roots: roots.to_vec(),
        ..StandardsCorpusPayload::default()
    };

    for declared in roots
    {
        Walk_Root(root, declared, filesystem, &mut payload);
    }

    return Canonical(payload);
}

/// `payload` in the order its own encoding writes.
///
/// [Encode](nomos_cap_standards_corpus::Encode_Payload) sorts every list so two readers that
/// reached the same documents produce identical bytes, and sorting here as well is what makes
/// a reader's own payload equal the payload [`nomos_cap_standards_corpus::Parse_Payload`]
/// recovers from its bytes — so a test can compare the two rather than only their encodings.
fn Canonical(mut payload: StandardsCorpusPayload) -> StandardsCorpusPayload
{
    payload.roots.sort();
    payload.roots.dedup();

    // Deduplicated as well as sorted, because the population is a *set* of documents and a
    // declaration may name overlapping roots -- `docs` and `docs/standards` together reach
    // `docs/standards/naming.md` twice. A duplicate would not merely read as a corpus with two
    // of one document: `Parse_Payload` refuses a path declared twice, so the payload this
    // reader produced could not survive its own encoding.
    payload.documents.sort();
    payload.documents.dedup();
    payload.rules.sort_by(|left, right| return left.path.cmp(&right.path));
    payload.rules.dedup_by(|left, right| return left.path == right.path);
    payload.issues.sort();
    payload.issues.dedup();

    return payload;
}

/// Walks one declared root, or records why it could not be walked.
fn Walk_Root<Fs: FileSystem>(root: &Path, declared: &str, filesystem: &Fs, payload: &mut StandardsCorpusPayload)
{
    let directory = root.join(declared);

    return match filesystem.Read_Directory(&directory)
    {
        Ok(entries) => Push_Entries(declared, entries, filesystem, payload),
        // A declared root that is not a directory is a defect in the declaration, not an empty
        // corpus: reporting it as a zero-document root is the silent omission this reader
        // exists to prevent, one level up from the documents themselves.
        Err(error) => Push_Issue(payload, declared, &format!("a declared corpus root could not be listed: {error}")),
    };
}

/// `entries` in a stable order, each one descended into.
fn Push_Entries<Fs: FileSystem>(prefix: &str, entries: Vec<PathBuf>, filesystem: &Fs, payload: &mut StandardsCorpusPayload)
{
    // Sorted so the population does not depend on the order a filesystem happens to report its
    // entries in, which is what makes two readers of one tree produce one payload.
    let mut entries = entries;
    entries.sort();

    for entry in entries
    {
        Descend(prefix, &entry, filesystem, payload);
    }
}

/// One directory entry: a nested directory is walked, a markdown document is read, and
/// anything else is not a standards document and is passed over.
fn Descend<Fs: FileSystem>(prefix: &str, entry: &Path, filesystem: &Fs, payload: &mut StandardsCorpusPayload)
{
    let Some(name) = entry.file_name().and_then(|name| return name.to_str())
    else
    {
        return;
    };
    let relative = format!("{prefix}/{name}");

    // A directory is whatever `Read_Directory` accepts, and the port has no question that
    // answers "is this a directory" more directly than performing the listing does. Asking
    // first is what keeps a *document* a thing this reader reads rather than a listing that
    // came back empty.
    if let Ok(entries) = filesystem.Read_Directory(entry)
    {
        return Push_Entries(&relative, entries, filesystem, payload);
    }

    if name.ends_with(DOCUMENT_SUFFIX)
    {
        Read_Document(entry, &relative, filesystem, payload);
    }
}

/// Reads one markdown document into the population.
fn Read_Document<Fs: FileSystem>(path: &Path, relative: &str, filesystem: &Fs, payload: &mut StandardsCorpusPayload)
{
    return match filesystem.Read_To_String(path)
    {
        Ok(text) => Record(&text, relative, payload),
        Err(error) => Push_Unreadable(payload, relative, &format!("the document could not be read: {error}")),
    };
}

/// Records one document: its population row, and — where it declares one — the declaration
/// itself.
fn Record(text: &str, relative: &str, payload: &mut StandardsCorpusPayload)
{
    return match Declared_Block(text)
    {
        // No fence at all. A document that declares nothing is not a rule and owes no further
        // statement; the schema makes no demand of it.
        Ok(None) => Push_Document(payload, relative, DeclarationKind::Undeclared),
        Err(reason) => Push_Unreadable(payload, relative, reason),
        Ok(Some(block)) => Record_Declaration(block, relative, payload),
    };
}

/// Records a document that wrote a declaration, along with the kind it names.
fn Record_Declaration(block: &str, relative: &str, payload: &mut StandardsCorpusPayload)
{
    let Some(declared) = Declared_Value(block, KIND_KEY)
    else
    {
        return Push_Unreadable(payload, relative, "the declaration names no kind");
    };
    let Some(kind) = DeclarationKind::Declared_By(declared)
    else
    {
        return Push_Unreadable(payload, relative, "the declaration's kind is not in the corpus's vocabulary");
    };

    Push_Document(payload, relative, kind);

    if kind == DeclarationKind::Rule
    {
        Record_Rule(block, relative, payload);
    }
}

/// Records what a document declaring `kind: rule` declares about itself, or why it could not
/// be read.
fn Record_Rule(block: &str, relative: &str, payload: &mut StandardsCorpusPayload)
{
    return match Declared_Rule(block, relative)
    {
        Ok(rule) => payload.rules.push(rule),
        Err(reason) => Push_Issue(payload, relative, &reason),
    };
}

/// The rule `block` declares, or why it is not a rule this workspace can read.
///
/// Every refusal here is a refusal rather than a default. A document that names no gate, or a
/// gate outside the corpus's vocabulary, has not declared that its enforcer can fail a build,
/// and a reader that supplied `Review` would be reporting a declaration nobody wrote — the
/// flattening the census this capability is measured against could not then detect.
fn Declared_Rule(block: &str, relative: &str) -> Result<DeclaredRule, String>
{
    let severity = RuleSeverity::Declared_By(Required(block, SEVERITY_KEY)?)
        .ok_or_else(|| return format!("{SEVERITY_KEY} is not in the corpus's vocabulary"))?;
    let gate = Declared_Gate(Required(block, GATE_KEY)?)
        .ok_or_else(|| return format!("{GATE_KEY} is not in the corpus's vocabulary"))?;

    return Ok(DeclaredRule {
        path: relative.to_owned(),
        id: Required(block, ID_KEY)?.to_owned(),
        title: Required(block, TITLE_KEY)?.to_owned(),
        severity,
        gate,
        enforced_by: Owners(Required(block, ENFORCED_BY_KEY)?)?,
    });
}

/// The non-empty value `block` declares for `key`.
fn Required<'a>(block: &'a str, key: &str) -> Result<&'a str, String>
{
    return match Declared_Value(block, key)
    {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(format!("the declaration names no {key}")),
    };
}

/// The enforcers `block` declares for [`ENFORCED_BY_KEY`], read from the flow sequence the
/// corpus's schema requires.
///
/// An empty sequence is refused rather than read as a rule nothing enforces: the schema
/// requires at least one entry, and this provider's payload carries no row for a rule whose
/// ownership the document left unwritten.
fn Owners(declared: &str) -> Result<Vec<String>, String>
{
    let inner = declared
        .strip_prefix('[')
        .and_then(|rest| return rest.strip_suffix(']'))
        .ok_or_else(|| return format!("{ENFORCED_BY_KEY} is not the [a, b] sequence the corpus's schema declares"))?;

    let owners: Vec<String> = inner.split(',').map(|owner| return owner.trim().to_owned()).filter(|owner| return !owner.is_empty()).collect();
    if owners.is_empty()
    {
        return Err(format!("{ENFORCED_BY_KEY} names no enforcer, and the corpus's schema requires at least one"));
    }

    return Ok(owners);
}

/// The value `block` declares for `key`, or `None` where it declares none.
///
/// Keys this workspace does not read are passed over rather than refused: the corpus's schema
/// declares more keys than a census needs (`applies_to`, `canonical`, `see_also`, and keys
/// the sibling `code-standards` corpus added of its own), and a reader that refused a document
/// for carrying one would report a defect where a document is simply richer than this reader.
fn Declared_Value<'a>(block: &'a str, key: &str) -> Option<&'a str>
{
    for line in block.lines()
    {
        let Some((name, value)) = line.split_once(':')
        else
        {
            continue;
        };
        if name.trim() == key
        {
            return Some(value.trim());
        }
    }

    return None;
}

/// The text between a document's declaration fences.
///
/// `Ok(None)` when the document opens no fence at all — a document that declared nothing, which
/// is not a defect. `Err` when it opens one and never closes it, which is: the document tried
/// to declare something and the declaration cannot be recovered, and that is a different fact
/// from an absence.
fn Declared_Block(text: &str) -> Result<Option<&str>, &'static str>
{
    let Some(after_open) = After_Opening_Fence(text)
    else
    {
        return Ok(None);
    };

    // The declaration is the lines *before* the closing fence, so the end is the byte offset at
    // which that fence line begins. Accumulated with `saturating_add` because this workspace
    // denies `arithmetic_side_effects` outright; a saturating length can only be wrong on a
    // document larger than the address space.
    let mut end = 0usize;
    for line in after_open.split_inclusive('\n')
    {
        if line.trim_end() == FENCE
        {
            return Ok(Some(&after_open[..end]));
        }
        end = end.saturating_add(line.len());
    }

    return Err("the declaration's fence never closes");
}

/// What follows a document's opening [`FENCE`] line, or `None` where its first line is not one.
fn After_Opening_Fence(text: &str) -> Option<&str>
{
    let mut lines = text.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end() != FENCE
    {
        return None;
    }

    return Some(&text[first.len()..]);
}

/// Records `path` in the population at `kind`.
fn Push_Document(payload: &mut StandardsCorpusPayload, path: &str, kind: DeclarationKind)
{
    payload.documents.push(DocumentDeclaration {
        path: path.to_owned(),
        kind,
    });
}

/// Records `path` as a document whose declaration could not be read.
///
/// Both a row and an issue, never one of the two: the row is what keeps the document in the
/// population a census counts, and the issue is what keeps the reason visible. The payload's
/// own [`nomos_cap_standards_corpus::Parse_Payload`] refuses a rule-kind document carrying
/// neither, so a reader that pushed only one of them could not produce an encodable fact.
fn Push_Unreadable(payload: &mut StandardsCorpusPayload, path: &str, reason: &str)
{
    Push_Document(payload, path, DeclarationKind::Undeclared);
    Push_Issue(payload, path, reason);
}

/// Records that `path`'s declaration could not be read, and why.
fn Push_Issue(payload: &mut StandardsCorpusPayload, path: &str, reason: &str)
{
    payload.issues.push(DeclarationIssue {
        path: path.to_owned(),
        reason: reason.to_owned(),
    });
}

#[cfg(test)]
#[path = "reading/tests.rs"]
mod tests;
