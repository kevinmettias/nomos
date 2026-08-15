//! What the gate runs is pinned, and so is what it runs as.

use crate::workflow::{Executable_Part, Unpinned_Actions_In, With_A_Tagged_Action, Workflow};

/// The step that puts the supply-chain tool on the runner.
///
/// Named as a constant for the same reason `RULES_STEP` is: the assertions below derive it
/// rather than grepping for the command, so a step renamed out from under them fails loudly
/// instead of matching nothing.
const SUPPLY_CHAIN_TOOL_STEP: &str = "Install cargo-deny";

/// Nothing executes here by a name its owner can move.
///
/// This gate pins the code it checks — `Cargo.lock` for the dependency graph,
/// `rust-toolchain.toml` for the compiler — and for a while pinned nothing that does the
/// checking. A tag is a reference its owner may repoint at any commit at any time, so `@v4`
/// fetches whatever it names on the morning the job runs. The failure that makes this worth a
/// test rather than a review habit is that a moved tag changes what executes and changes
/// nothing in this repository: there is no diff to notice, so the only thing standing between
/// a repointed tag and a run of it is an assertion that the reference is not a tag at all.
#[test]
fn Test_Every_Action_Should_Be_Pinned_To_A_Commit()
{
    let unpinned = Unpinned_Actions_In(&Workflow());

    assert!(
        unpinned.is_empty(),
        "the gate reaches for {unpinned:?} by something other than a full commit identifier. \
         A tag is a name its owner can move, so this workflow would run different code with \
         no change in this repository to show it"
    );
}

/// The control that shows the assertion above can fire.
///
/// Without it, `Test_Every_Action_Should_Be_Pinned_To_A_Commit` is satisfied by a workflow
/// with no actions in it at all, and a string-absence assertion that has never been shown to
/// fail is indistinguishable from a comment — the reasoning
/// `Test_The_Excuse_Check_Should_Reject_An_Excused_Step` already makes for its own subject.
#[test]
fn Test_The_Pin_Check_Should_Reject_An_Action_On_A_Tag()
{
    let tagged = With_A_Tagged_Action(&Workflow());

    assert!(
        tagged.contains("@v4"),
        "the fixture rewrote nothing, so this control is about a workflow that does not exist"
    );
    assert_eq!(
        Unpinned_Actions_In(&tagged),
        vec!["actions/checkout@v4".to_owned()],
        "the check did not report an action this fixture put back on a moving tag"
    );
}

/// The supply-chain tool arrives at a version this file chose.
///
/// `cargo install cargo-deny --locked` builds whatever `crates.io` serves that day.
/// `--locked` pins the dependencies of the version it selected and does not select a version,
/// so it reads like a pin while leaving the binary free to change — which is the whole of why
/// this is asserted rather than left to the flag that looks like it already covers it.
///
/// Derived by step name rather than grepped, so rewriting the install into a script fails
/// here instead of satisfying this by containing the right words.
#[test]
fn Test_The_Supply_Chain_Tool_Should_Be_Installed_At_A_Chosen_Version()
{
    let argv = nomos_ledger::Derive_Step(&Workflow(), SUPPLY_CHAIN_TOOL_STEP)
        // Deriving by step name is what makes this assertion survive the install being
        // rewritten into a shell script — but only if the refusal that rewrite produces stops
        // the run. Swallowed, it would leave `Version_Pinned_In` an empty argv and the pin
        // unexamined, which is the same unversioned install this test was written against.
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

    assert!(
        argv.iter().any(|argument| return argument == "--locked"),
        "the install must still lock the tool's own dependency graph, got {argv:?}"
    );

    let pinned = Version_Pinned_In(&argv);

    assert!(
        pinned
            .chars()
            .next()
            .is_some_and(|character| return character.is_ascii_digit()),
        "`--version {pinned}` is not a version"
    );
}

/// The value `--version` names, which must be there and must be a version.
fn Version_Pinned_In(argv: &[String]) -> &str
{
    let flag = argv
        .iter()
        .position(|argument| return argument == "--version")
        .unwrap_or_else(|| {
            // The absent flag is precisely the condition this file exists to fail on, so it
            // cannot be reported by returning something the caller then has to interpret —
            // the caller's own check is "does it start with a digit", and any placeholder
            // either passes that check or fails it for the wrong reason.
            panic!(
                "the supply-chain tool is installed at whatever version crates.io serves \
                 today: {argv:?}"
            )
        });

    return argv
        .get(flag.saturating_add(1))
        // A `--version` sitting last in the argv is a different defect from a `--version`
        // that is absent, and both must be told apart from a version that is merely odd. The
        // signature returns a borrowed `&str` with no vocabulary for "there was no next
        // argument", so the distinction is drawn here or nowhere.
        .unwrap_or_else(|| panic!("`--version` names no version: {argv:?}"));
}

/// The job declares the token it runs with, rather than inheriting it.
///
/// Every step in this gate reads. Without a `permissions:` block the workflow runs with
/// whatever the repository-wide default happens to be, which is a permission grant decided in
/// a settings page this file cannot show and no test in this repository can read — the same
/// shape as a declaration nothing executes, which is `OD-GATE-006`'s subject.
#[test]
fn Test_The_Gate_Should_Declare_The_Token_It_Runs_With()
{
    let executable = Executable_Part(&Workflow());

    assert!(
        executable.contains("\npermissions:"),
        "the gate declares no permissions, so its token carries the repository default"
    );
    assert!(
        executable.contains("contents: read"),
        "the gate's permissions do not grant the read this job needs to check out"
    );
    assert!(
        !executable.contains("contents: write"),
        "no step in this gate writes to the repository, so nothing here needs write"
    );
}
