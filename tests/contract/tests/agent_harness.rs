//! The agent harness routes, and a routing document that restates is an unchecked copy.
//!
//! `OD-AGENT-001` decides that the committed agent surface — `AGENTS.md`, a `CLAUDE.md`
//! that imports it, and skills under the Claude directory — answers where authority lives
//! and how to act on it safely, and never what the authority says. The pressure against
//! that is specific: an instruction file is consumed by a machine at the start of every
//! session and reviewed by a person approximately never, so a table pasted into it goes
//! stale more quietly than the same table anywhere else.
//!
//! # What is asserted
//!
//! The structural promises, not the prose. That the adapter imports the contract rather
//! than growing its own copy; that every repository path the harness names is a path that
//! exists; that the contract names the work and rationale authorities at all; that no row
//! naming a workspace crate has been restated here, which is the signature of the band
//! table `tests/contract/tests/boundaries.rs` already checks against reality in both
//! directions; and that a committed skill declares a name matching its own directory.
//!
//! Each has a negative control over a fixture string in
//! [`Test_Every_Check_Here_Should_Fail_On_A_Fixture_That_Breaks_It`]. A check whose failing
//! case has never been observed is `OD-GATE-001`'s defect wearing a new file name, and four
//! of the five checks below pass trivially over a file that says nothing.
//!
//! # What is not asserted
//!
//! Whether the routing is *correct* — whether the authority a line names is really the one
//! that answers that question. That is meaning, and this crate deliberately has no type
//! that decides it. What is caught is the cheap half: a route to a file somebody moved.
//!
//! Path recognition is limited to inline code spans, so dropping the backticks around a
//! path evades it. That is accepted rather than closed. Routing lines are written in code
//! spans by convention, and a recogniser that guessed at bare prose would report every
//! sentence containing a full stop.

use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The most lines `AGENTS.md` may carry.
///
/// The limit is the mechanism rather than a style preference. Routing does not need the
/// room, so a contract that grows past this has started restating something — and the
/// restatement arrives one reasonable paragraph at a time, which is exactly the growth no
/// reviewer refuses. A figure somebody chose, in the shape `OD-GATE-001` uses, rather than
/// a silence nobody measured: the contract was 93 lines when this was written, so the
/// headroom is about a quarter — enough for a routing line to a record that does not exist
/// yet, and not enough for a section.
const CONTRACT_LINE_BUDGET: usize = 120;

/// The most lines a vendor adapter may carry.
///
/// Smaller for the same reason and more sharply, because the adapter is the file with a
/// second copy already available to it: everything it might restate is one import away.
const ADAPTER_LINE_BUDGET: usize = 60;

/// The canonical contract, and the adapter that must not become a second one.
const CONTRACT: &str = "AGENTS.md";
const ADAPTER: &str = "CLAUDE.md";

/// Where committed skills live, relative to the workspace root.
const SKILL_ROOT: &str = ".claude/skills";

/// Reads a file under the workspace root, failing with the path when it is not there.
fn Read_Harness_File(relative: &str) -> String
{
    let path = Workspace::Workspace_Root().join(relative);

    return std::fs::read_to_string(&path).unwrap_or_else(|error| {
        let location = path.display();
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
fn Harness_Files() -> Vec<(String, String)>
{
    let mut files = vec![
        (CONTRACT.to_owned(), Read_Harness_File(CONTRACT)),
        (ADAPTER.to_owned(), Read_Harness_File(ADAPTER)),
    ];

    for directory in Skill_Directories()
    {
        let manifest = directory.join("SKILL.md");
        let Ok(text) = std::fs::read_to_string(&manifest)
        else
        {
            continue;
        };

        files.push((manifest.display().to_string().replace('\\', "/"), text));
    }

    return files;
}

/// Every directory under the skill root, whether or not it holds a manifest.
///
/// Absence is not an error. Skills arrive on their own ledger items, and a repository with
/// none is a repository that has not needed one yet.
fn Skill_Directories() -> Vec<PathBuf>
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
fn Named_Paths(text: &str) -> Vec<String>
{
    let mut paths = Vec::new();

    for (index, span) in text.split('`').enumerate()
    {
        // Splitting on the delimiter puts the spans at the odd positions: text, span,
        // text, span. An unbalanced backtick therefore reads the prose as a span, which
        // is loud rather than silent, and is what should happen.
        if index % 2 == 0 || span.is_empty()
        {
            continue;
        }

        if span.split_whitespace().count() != 1
        {
            continue;
        }

        let candidate = span.trim_end_matches('/');
        let named = candidate.contains('/')
            || [".md", ".rs", ".toml", ".json", ".yml"]
                .iter()
                .any(|extension| return candidate.ends_with(extension));

        if named
        {
            paths.push(candidate.to_owned());
        }
    }

    paths.sort();
    paths.dedup();
    return paths;
}

/// The paths a text names that are not in the tree rooted at `root`.
fn Missing_Paths(text: &str, root: &Path) -> Vec<String>
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
fn Restated_Rows(text: &str, members: &BTreeSet<String>) -> Vec<String>
{
    return text
        .lines()
        .filter(|line| return line.trim_start().starts_with('|'))
        .filter(|line| return members.iter().any(|member| return line.contains(member.as_str())))
        .map(|line| return line.trim().to_owned())
        .collect();
}

/// Whether a text imports another file rather than carrying its own copy.
fn Imports(text: &str, target: &str) -> bool
{
    return text.contains(&format!("@{target}"));
}

/// The `name` a skill manifest declares in its front matter.
///
/// Front matter or nothing. A `name:` line further down the body is prose about a name
/// rather than a declaration of one, and reading it would let a manifest satisfy the
/// directory check by accident.
fn Declared_Skill_Name(text: &str) -> Option<String>
{
    let body = text.strip_prefix("---")?;
    let end = body.find("\n---")?;

    return body.get(..end)?.lines().find_map(|line| {
        let value = line.strip_prefix("name:")?;
        return Some(value.trim().to_owned());
    });
}

/// The front door has to be at the door.
///
/// Claude Code and Codex both look for a file by name before they look at anything else,
/// and neither existed here. Empty counts as absent: a file present and saying nothing
/// routes an agent nowhere while making the absence undetectable.
#[test]
fn Test_The_Repository_Should_Have_An_Agent_Front_Door()
{
    for name in [CONTRACT, ADAPTER]
    {
        let text = Read_Harness_File(name);

        assert!(
            text.lines().filter(|line| return !line.trim().is_empty()).count() > 5,
            "{name} exists and says almost nothing, which is indistinguishable from its \
             absence to the agent reading it"
        );
    }
}

/// The adapter imports the contract; it does not grow a second one.
///
/// Two independently maintained instruction files is the drift `OD-AGENT-001` refuses,
/// and it is worse than one stale file because the two disagree without either being
/// wrong on its face.
#[test]
fn Test_The_Adapter_Should_Import_The_Contract_Rather_Than_Repeat_It()
{
    let adapter = Read_Harness_File(ADAPTER);

    assert!(
        Imports(&adapter, CONTRACT),
        "{ADAPTER} does not import {CONTRACT}. An adapter that restates the contract is a \
         second authority, and OD-AGENT-001 admits only vendor mechanics here."
    );

    let lines = adapter.lines().count();
    assert!(
        lines <= ADAPTER_LINE_BUDGET,
        "{ADAPTER} is {lines} lines against a budget of {ADAPTER_LINE_BUDGET}. Everything \
         it might restate is one import away, so length here is the tell."
    );
}

/// Routing does not need the room, so the budget is what keeps it routing.
#[test]
fn Test_The_Contract_Should_Stay_Inside_Its_Line_Budget()
{
    let lines = Read_Harness_File(CONTRACT).lines().count();

    assert!(
        lines <= CONTRACT_LINE_BUDGET,
        "{CONTRACT} is {lines} lines against a budget of {CONTRACT_LINE_BUDGET}. The limit \
         is the mechanism OD-AGENT-001 relies on: a contract that grows past it has started \
         restating something, one reasonable paragraph at a time."
    );
}

/// A route to a file somebody moved is a route to nothing.
///
/// This is the half of correctness a machine can check. It runs over the skills too, so a
/// path named inside a procedure is held to the same standard as one in the contract.
#[test]
fn Test_Every_Path_The_Harness_Names_Should_Exist()
{
    let root = Workspace::Workspace_Root();
    let mut broken = Vec::new();

    for (name, text) in Harness_Files()
    {
        for missing in Missing_Paths(&text, &root)
        {
            broken.push(format!("{name} names {missing}, which is not in the tree"));
        }
    }

    assert!(
        broken.is_empty(),
        "the harness routes to paths that do not exist: {broken:#?}.\n\
         A path that has moved is the failure this check exists for; a path that is real \
         but uncommitted is the other one, and CI is where it shows."
    );
}

/// The band table is checked where it lives, and nowhere else may hold a copy.
///
/// `tests/contract/tests/boundaries.rs` compares the README's rows against the workspace
/// in both directions. A second copy in a file nobody reviews would be unchecked, and
/// would go stale exactly the way the README already had when `OD-PROJECT-001` found it:
/// twenty-two members described by eleven rows.
#[test]
fn Test_The_Harness_Should_Not_Restate_The_Band_Table()
{
    let workspace = Workspace::Load();
    let members: BTreeSet<String> = workspace
        .Members()
        .into_iter()
        .map(|member| return member.name.clone())
        .collect();

    assert!(
        !members.is_empty(),
        "no workspace members were resolved, so this check would pass over any file at all"
    );

    let mut restated = Vec::new();
    for (name, text) in Harness_Files()
    {
        for row in Restated_Rows(&text, &members)
        {
            restated.push(format!("{name}: {row}"));
        }
    }

    assert!(
        restated.is_empty(),
        "the harness carries table rows naming workspace crates: {restated:#?}.\n\
         OD-AGENT-001 admits a link to README.md and refuses a copy of it."
    );
}

/// Routing is the contract's whole job, so the two authorities it exists for are required.
///
/// Named as paths rather than as prose, because a path is what an agent can open. The
/// board and the rationale are the two an arriving session cannot infer: the ledger is not
/// discoverable from the source, and a record explains a decision the code only shows the
/// result of.
#[test]
fn Test_The_Contract_Should_Name_The_Work_And_Rationale_Authorities()
{
    let named: BTreeSet<String> = Named_Paths(&Read_Harness_File(CONTRACT)).into_iter().collect();

    for authority in ["work/ledger.json", "docs/records"]
    {
        assert!(
            named.iter().any(|path| return path.starts_with(authority)),
            "{CONTRACT} never names {authority}. An agent that does not find the board \
             invents one, and an agent that does not find the records repeats a decision \
             somebody already made."
        );
    }
}

/// A skill is addressed by name, and the name it declares must be the one it is found at.
///
/// A mismatch is not cosmetic: the directory is how the tool finds it and the front matter
/// is how the tool describes it, so the two disagreeing produces a skill that is invoked
/// under one name and reports itself under another.
#[test]
fn Test_Every_Committed_Skill_Should_Declare_A_Name_Matching_Its_Directory()
{
    for directory in Skill_Directories()
    {
        let label = directory.display().to_string().replace('\\', "/");
        let manifest = directory.join("SKILL.md");

        assert!(
            manifest.is_file(),
            "{label} is a skill directory with no SKILL.md, so it is a directory the tool \
             will not load and a reader will assume works"
        );

        let text = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|error| panic!("cannot read {label}/SKILL.md: {error}"));

        let declared = Declared_Skill_Name(&text)
            .unwrap_or_else(|| panic!("{label}/SKILL.md declares no name in its front matter"));

        let expected = directory
            .file_name()
            .map(|name| return name.to_string_lossy().into_owned())
            .unwrap_or_default();

        assert_eq!(
            declared, expected,
            "{label}/SKILL.md calls itself {declared} while living at {expected}"
        );
    }
}

/// Every check above passes over a file that says nothing, so each is shown failing.
///
/// Over fixtures rather than over the tree, because the alternative is a test that edits
/// the repository to prove a point and leaves it edited when it fails partway.
#[test]
fn Test_Every_Check_Here_Should_Fail_On_A_Fixture_That_Breaks_It()
{
    let root = Workspace::Workspace_Root();

    assert!(
        !Missing_Paths("routes to `docs/records/there-is-no-such-record.md`", &root).is_empty(),
        "a named path that is not in the tree was not reported, so the routing check would \
         accept a link to anywhere"
    );
    assert!(
        Missing_Paths("run `cargo fmt` and read `README.md`", &root).is_empty(),
        "a command span or a real path was mistaken for a broken route"
    );

    let members: BTreeSet<String> = ["nomos-rules".to_owned()].into_iter().collect();
    assert!(
        !Restated_Rows("| 30 | `nomos-rules` | A rule is a pure function. |", &members).is_empty(),
        "a pasted band row was not reported, so the band table could be copied here"
    );
    assert!(
        Restated_Rows("The rule crate is `nomos-rules`, in README.md.", &members).is_empty(),
        "prose naming a crate was reported as a restated row"
    );

    assert!(
        !Imports("# Claude Code\n\nThe bands are as follows.", CONTRACT),
        "an adapter with no import was accepted, so it could carry its own contract"
    );

    assert_eq!(
        Declared_Skill_Name("---\nname: nomos-task\ndescription: x\n---\n").as_deref(),
        Some("nomos-task"),
        "a declared skill name was not read back out of its own front matter"
    );
    assert_eq!(
        Declared_Skill_Name("# nomos-task\n\nNo front matter here.\n"),
        None,
        "a manifest with no front matter reported a name, so the check would compare \
         nothing against the directory"
    );
}
