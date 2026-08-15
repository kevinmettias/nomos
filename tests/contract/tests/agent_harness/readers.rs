use crate::{ADAPTER, BOARD, CONTRACT, SKILL_ROOT};
use nomos_contract_tests::Workspace;
use nomos_ledger::LedgerDocument;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn Read_Harness_File(relative: &str) -> String
{
    let path = Workspace::Workspace_Root().join(relative);

    return std::fs::read_to_string(&path).unwrap_or_else(|error| {
        let location = path.display();
        // Every check in this suite is a statement about this text, so a file returned as an
        // empty string would satisfy all of them at once: no route to contradict, no item
        // named, no hazard left to expire. The suite would go green for the one tree where
        // the front door had gone missing.
        panic!(
            "cannot read {location}: {error}.\n\
             OD-AGENT-001 makes this file the repository's agent front door, and an agent \
             that does not find it rediscovers the architecture from source every session."
        )
    });
}

/// Every committed harness file, as a path and its text.
///
/// The skills are included so that a route added inside one is checked the same way as a
/// route in the contract. They are discovered rather than declared: a list of skills here
/// would be a second copy of what the directory says, which is the defect this whole file
/// is about.
pub(crate) fn Harness_Files() -> Vec<(String, String)>
{
    let mut files = vec![
        (CONTRACT.to_owned(), Read_Harness_File(CONTRACT)),
        (ADAPTER.to_owned(), Read_Harness_File(ADAPTER)),
    ];
    for directory in Skill_Directories()
    {
        let skill = Skill_File(&directory);

        files.extend(skill);
    }

    return files;
}

/// One skill's manifest, or nothing where the directory holds none.
///
/// Named relative to the root rather than by its absolute location, so that a declaration
/// below and a failure message above spell one file one way.
pub(crate) fn Skill_File(directory: &Path) -> Option<(String, String)>
{
    let text = std::fs::read_to_string(directory.join("SKILL.md")).ok()?;
    let name = directory
        .file_name()
        .map(|name| return name.to_string_lossy().into_owned())
        .unwrap_or_default();

    return Some((format!("{SKILL_ROOT}/{name}/SKILL.md"), text));
}

/// The board as the ledger's own type reads it.
///
/// Through `LedgerDocument` rather than a second walk over the same JSON. A private reader
/// here would be a second opinion about what a finished item looks like, and this
/// workspace has already recorded twice what two guards for one question cost.
pub(crate) fn Board() -> LedgerDocument
{
    let path = Workspace::Workspace_Root().join(BOARD);
    let text = std::fs::read_to_string(&path)
        // `BOARD` is a committed path. A board that will not open means the layout this suite
        // is written against has moved, and every caller below would otherwise report that as
        // "the item is not on the board" — the right failure attributed to the wrong file.
        .unwrap_or_else(|error| panic!("cannot read the board at {}: {error}", path.display()));

    return serde_json::from_str(&text).unwrap_or_else(|error| {
        // The ledger's own type refusing the file that is meant to be its serialization is a
        // defect in the board itself, not an answer to the caller's question. Substituting an
        // empty document would answer "no such item" for every hazard the harness declares.
        panic!("the board at {} did not parse: {error}", path.display())
    });
}

/// Whether a token is authored in the shape of a ledger item identifier.
///
/// `P<phase>-<WORD>[-<WORD>…]`. Deliberately a shape rather than a lookup: an identifier
/// that matches nothing on the board is the interesting case, not one to be filtered out
/// before anybody notices it.
pub(crate) fn Is_Item_Id(token: &str) -> bool
{
    let mut segments = token.split('-');
    let Some(phase) = segments.next().and_then(|first| return first.strip_prefix('P'))
    else
    {
        return false;
    };
    if phase.is_empty() || !phase.chars().all(|character| return character.is_ascii_digit())
    {
        return false;
    }

    let words: Vec<&str> = segments.collect();

    return !words.is_empty() && words.iter().all(|word| return Is_Word(word));
}

/// A word in an identifier: not empty, and uppercase throughout.
pub(crate) fn Is_Word(word: &str) -> bool
{
    return !word.is_empty() && word.chars().all(|character| return character.is_ascii_uppercase());
}

/// Every ledger item a text names.
pub(crate) fn Named_Items(text: &str) -> Vec<String>
{
    let mut found: Vec<String> = text
        .split(|character: char| {
            return !(character.is_ascii_alphanumeric() || character == '-');
        })
        .filter(|token| return Is_Item_Id(token))
        .map(str::to_owned)
        .collect();

    found.sort();
    found.dedup();
    return found;
}

/// Every directory under the skill root, whether or not it holds a manifest.
///
/// Absence is not an error. Skills arrive on their own ledger items, and a repository with
/// none is a repository that has not needed one yet.
pub(crate) fn Skill_Directories() -> Vec<PathBuf>
{
    let root = Workspace::Workspace_Root().join(SKILL_ROOT);
    let Ok(entries) = std::fs::read_dir(&root)
    else
    {
        return Vec::new();
    };

    let mut directories: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| return path.is_dir())
        .collect();

    directories.sort();
    return directories;
}

/// The repository paths a harness file names, taken from its inline code spans.
///
/// A code span is a repository path when it carries no whitespace and either contains a
/// separator or ends in an extension this workspace uses. That excludes the commands —
/// `cargo fmt`, `git add -A` — which are code spans naming no file, and the identifiers
/// like `done_when`, which name a field rather than a path.
pub(crate) fn Named_Paths(text: &str) -> Vec<String>
{
    let mut paths = Vec::new();
    for (index, span) in text.split('`').enumerate()
    {
        // Splitting on the delimiter puts the spans at the odd positions: text, span,
        // text, span. An unbalanced backtick therefore reads the prose as a span, which
        // is loud rather than silent, and is what should happen.
        if index % 2 == 1
        {
            let named = Named_Path(span);

            paths.extend(named);
        }
    }

    paths.sort();
    paths.dedup();
    return paths;
}

/// One code span, if it names a repository path.
///
/// It does when it carries no whitespace and either contains a separator or ends in an
/// extension this workspace uses. That excludes the commands and the field names.
pub(crate) fn Named_Path(span: &str) -> Option<String>
{
    if span.split_whitespace().count() != 1
    {
        return None;
    }

    let candidate = span.trim_end_matches('/');
    let named = candidate.contains('/')
        || [".md", ".rs", ".toml", ".json", ".yml"]
            .iter()
            .any(|extension| return candidate.ends_with(extension));

    return named.then(|| return candidate.to_owned());
}

/// The paths a text names that are not in the tree rooted at `root`.
pub(crate) fn Missing_Paths(text: &str, root: &Path) -> Vec<String>
{
    return Named_Paths(text)
        .into_iter()
        .filter(|relative| return !root.join(relative).exists())
        .collect();
}

/// The table rows in a text that name a workspace crate.
///
/// The band table is `| <band> | `<crate>` | … |`, so a row mentioning a member is the
/// signature of a restatement whether or not the number came with it. Prose naming a crate
/// is left alone: a sentence cannot drift into a table.
pub(crate) fn Restated_Rows(text: &str, members: &BTreeSet<String>) -> Vec<String>
{
    return text
        .lines()
        .filter(|line| return line.trim_start().starts_with('|'))
        .filter(|line| return members.iter().any(|member| return line.contains(member.as_str())))
        .map(|line| return line.trim().to_owned())
        .collect();
}

/// Whether a text imports another file rather than carrying its own copy.
pub(crate) fn Imports(text: &str, target: &str) -> bool
{
    return text.contains(&format!("@{target}"));
}

/// The `name` a skill manifest declares in its front matter.
///
/// Front matter or nothing. A `name:` line further down the body is prose about a name
/// rather than a declaration of one, and reading it would let a manifest satisfy the
/// directory check by accident.
pub(crate) fn Declared_Skill_Name(text: &str) -> Option<String>
{
    let body = text.strip_prefix("---")?;
    let end = body.find("\n---")?;

    return body.get(..end)?.lines().find_map(|line| {
        let value = line.strip_prefix("name:")?;
        return Some(value.trim().to_owned());
    });
}
