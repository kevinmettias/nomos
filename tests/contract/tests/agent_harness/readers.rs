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

/// A committed file read as the source of truth it is, rather than as a harness file.
///
/// Used to derive the gate's own command list and the README's own ledger verb reference so
/// neither has to be retyped as a constant here — the same reason `Board` reads
/// `work/ledger.json` through the ledger's own type instead of a second parse.
pub(crate) fn Read_Repo_File(relative: &str) -> String
{
    let path = Workspace::Workspace_Root().join(relative);

    return std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "cannot read {}: {error}. This check derives what it looks for from this file \
             rather than retyping it, so an unreadable source leaves nothing to compare \
             against.",
            path.display()
        )
    });
}

/// Every command the gate's own `run:` lines carry, derived from the workflow rather than
/// retyped beside it.
///
/// Deliberately not `Derive_Step`: that function wants a step name, and naming every step
/// here would be its own restatement of the workflow. This reads every `run:` line the
/// workflow has, whichever step it belongs to.
pub(crate) fn Gate_Run_Commands(workflow: &str) -> Vec<String>
{
    let mut commands: Vec<String> = workflow
        .lines()
        .map(str::trim)
        .filter_map(|line| return line.strip_prefix("run:"))
        .map(|command| return command.trim().to_owned())
        .filter(|command| return !command.is_empty())
        .collect();

    commands.sort();
    commands.dedup();
    return commands;
}

/// The gate commands a text carries verbatim, labelled so a failure can show which ones.
pub(crate) fn Restated_Gate_Commands(text: &str, commands: &[String]) -> Vec<String>
{
    return commands
        .iter()
        .filter(|command| return text.contains(command.as_str()))
        .cloned()
        .collect();
}

/// Every `nomos work <verb>` usage line the README's own ledger verb reference carries.
///
/// The reference is the fenced block under "Coordinating concurrent work"; every line in it
/// that opens a verb starts with `nomos work `, which is a shape nothing else in the README
/// happens to share.
pub(crate) fn Ledger_Verb_Lines(readme: &str) -> Vec<String>
{
    let mut lines: Vec<String> = readme
        .lines()
        .map(str::trim)
        .filter(|line| return line.starts_with("nomos work "))
        .map(str::to_owned)
        .collect();

    lines.sort();
    lines.dedup();
    return lines;
}

/// The verb-reference lines a text carries verbatim, matched whole line to whole line so a
/// sentence that merely mentions a verb (`` `nomos work validate` `` in prose) is not
/// mistaken for a copy of the block those lines come from.
pub(crate) fn Restated_Ledger_Verb_Lines(text: &str, verb_lines: &[String]) -> Vec<String>
{
    return text
        .lines()
        .map(str::trim)
        .filter(|line| return verb_lines.iter().any(|verb| return verb == line))
        .map(str::to_owned)
        .collect();
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

/// Every path a fresh checkout of this repository has: each tracked file, and every
/// directory that holds one.
///
/// Derived from git rather than from the disk, because the disk is one developer's machine.
/// `.gitignore` puts whole directories outside the tree -- `/.cargo/` among them -- so a file
/// left in one satisfies an existence check here and in no checkout anywhere. A check that
/// cannot tell those apart reports the state of a workstation and calls it the state of the
/// repository, which is `OD-GATE-001`'s defect arriving from the far side: not a test that
/// skipped, but one that passed on evidence the repository does not carry.
///
/// The index rather than `HEAD`. An item that adds a file the harness routes to can stage it
/// and be judged honestly in the same run, where judging the last commit would refuse the
/// work until after the commit that does it.
///
/// # Panics
///
/// Panics if git cannot produce the list. A guard that cannot see its subject fails loudly:
/// falling back to the disk would restore the exact defect this exists to remove, and the
/// fallback would be invisible, because it agrees with the old answer everywhere except the
/// one case worth catching.
pub(crate) fn Checked_Out_Paths() -> BTreeSet<String>
{
    let root = Workspace::Workspace_Root();
    let output = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(&root)
        .output()
        .unwrap_or_else(|error| panic!("cannot run git to list the checked-out files: {error}"));

    assert!(
        output.status.success(),
        "git could not list the tracked files in {} ({}).\n\
         This reader judges what a fresh checkout has rather than what this disk has, so \
         without git there is nothing honest left to check.",
        root.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    );

    return Checkout_Entries(&String::from_utf8_lossy(&output.stdout));
}

/// Every tracked path in a NUL-separated `git ls-files -z` listing, with every directory
/// along the way.
///
/// The directories are entries in their own right because the harness routes to directories
/// as well as to files -- `tests/contract` and `docs/records` are both rows in the contract's
/// own table -- and once they are here the lookup is set membership and nothing else.
///
/// Separated from the command that produces the listing so the controls can exercise it over
/// a checkout they write themselves. A control that had to reach for the real tree would be
/// asserting against the same machine-dependent state this reader exists to stop trusting.
pub(crate) fn Checkout_Entries(listing: &str) -> BTreeSet<String>
{
    let mut entries = BTreeSet::new();

    for file in listing.split('\0').filter(|entry| return !entry.is_empty())
    {
        entries.insert(file.to_owned());
        let mut directory = file;

        while let Some((parent, _)) = directory.rsplit_once('/')
        {
            entries.insert(parent.to_owned());
            directory = parent;
        }
    }

    return entries;
}

/// The paths a text names that a fresh checkout does not have.
pub(crate) fn Missing_Paths(text: &str, checkout: &BTreeSet<String>) -> Vec<String>
{
    return Named_Paths(text)
        .into_iter()
        .filter(|relative| return !checkout.contains(relative))
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

/// Every crate count a text states about the XVPE crossing.
///
/// `tests/contract/tests/boundaries/lock_pinning.rs` measures the crossing in both
/// directions — which packages cross, and that every one is pinned to the revision the
/// manifests declare. A number restated elsewhere is a second encoding of that same fact
/// with nothing re-deriving it, which is `OD-GATE-011`'s defect, and it is exactly how
/// `AGENTS.md` came to promise six crates over a crossing that carries sixteen declarations
/// across eight manifests.
///
/// A count is reported when a paragraph about the crossing carries a number immediately
/// before the noun it counts. Wording is otherwise left alone: this reads for the shape of a
/// restated measurement, not for every sentence a paragraph mentions XVPE in.
pub(crate) fn Crossing_Counts(text: &str) -> Vec<String>
{
    const COUNTED: [&str; 4] = ["crate", "manifest", "package", "dependency"];
    const SPELLED: [&str; 12] = [
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        "eleven", "twelve",
    ];

    // The paragraph is the unit, not the line and not the sentence. Line breaks in the
    // contract are wrapping rather than meaning, and the sentence is too narrow in the other
    // direction: a count fairly often sits one sentence away from the crossing it counts,
    // which is exactly where the retired sentence would have drifted to escape a check
    // written against the sentence.
    let lowered = text.to_lowercase();
    let mut counts = Vec::new();

    for paragraph in lowered.split("\n\n")
    {
        if !paragraph.contains("xvpe")
        {
            continue;
        }
        let words: Vec<&str> = paragraph.split_whitespace().collect();

        for pair in words.windows(2)
        {
            // Destructured rather than indexed. `windows(2)` cannot yield a short slice, but
            // the slice pattern says so in the type of the binding, where an index and a
            // comment claiming it is in range would leave a panic path a reader has to trust.
            let [left, right] = pair
            else
            {
                continue;
            };

            let number = left.chars().all(|character| return character.is_ascii_digit())
                || SPELLED.contains(left);
            let counted = COUNTED.iter().any(|noun| return right.starts_with(noun));

            if number && counted
            {
                counts.push(format!("{left} {right}"));
            }
        }
    }

    counts.sort();
    counts.dedup();
    return counts;
}
