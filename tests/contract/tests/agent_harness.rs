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
//! exists; that the contract names each authority it claims to route to; that no row
//! naming a workspace crate has been restated here, which is the signature of the band
//! table `tests/contract/tests/boundaries.rs` already checks against reality in both
//! directions; and that a committed skill declares a name matching its own directory.
//!
//! And that a hazard written down as temporary is still temporary. `OD-AGENT-001` admits
//! an operating hazard tied to an open defect *on the condition* that it names the item
//! which will close it, so that it can be removed rather than accumulate. That condition
//! is only a promise until something reads the board: a warning deleted early disappears
//! while the defect is still live, and one left behind costs every session a step spent
//! defending against nothing.
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
//!
//! # Why this file declares modules through `#[path]`
//!
//! `AGENTS.md` names `tests/contract/tests/agent_harness.rs` in a code span, and
//! [`Test_Every_Path_The_Harness_Names_Should_Exist`] is the check that the path is real. So
//! this file cannot become `agent_harness/main.rs`: the split would turn its own check red
//! against the document it is about, and `AGENTS.md` is not in this item's territory.
//!
//! A test binary root resolves a bare `mod x;` against its own directory — `tests/x.rs` —
//! which cargo would then build as a second test target. `#[path]` is what lets the root keep
//! the name the contract routes to while its readers and its checks live beside it.

#[path = "agent_harness/checks.rs"]
mod checks;
#[path = "agent_harness/controls.rs"]
mod controls;
#[path = "agent_harness/readers.rs"]
mod readers;


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

/// The board, which is the authority on whether a temporary hazard is still temporary.
const BOARD: &str = "work/ledger.json";

/// The authorities the contract claims to route to, and must therefore name.
///
/// Four rather than two. The architecture pair was left unasserted when this file was
/// written, which meant an edit could drop the two rows that send an agent to the
/// description of the workspace and to the tests that check it, and the gate would stay
/// green over a routing table that had stopped routing.
///
/// The fifth and sixth are whole records rather than directories, and are the entries here
/// that are not satisfied by their neighbours. `docs/records` already matches any record
/// path, so either row could be deleted without failing anything while the generic
/// rationale row survived — and each answers a question the generic row does not. *Why was
/// it decided that way* is answered by whichever record argued the case; *which product owns
/// this* is answered by one record, and getting it wrong is how the scope drift
/// `ARC-ECOSYSTEM-001` exists to stop happens again; *what does a handoff carry* is answered
/// by `OD-AGENT-002`, and a routing table that quietly dropped it would leave a session that
/// exhausts its context with nowhere to be told not to paste the repository into its own
/// successor. Naming the path also puts it under the existence check, so renaming the record
/// without fixing the route fails here rather than in a reader's hands.
const ROUTED_AUTHORITIES: &[&str] = &[
    "README.md",
    "tests/contract",
    "docs/records",
    "docs/records/ARC-ECOSYSTEM-001-four-products-share-one-seam-and-ownership-is-decided-by-semantics.md",
    "docs/records/OD-AGENT-002-a-handoff-carries-session-local-state-and-routes-to-authority-for-everything-else.md",
    BOARD,
];

/// Where the gate command list lives, so a restatement of it can be derived rather than
/// retyped beside this test.
const GATE_WORKFLOW: &str = nomos_ledger::GATE_WORKFLOW;

/// Where the ledger verb reference lives, so a restatement of it can be derived the same way.
const LEDGER_VERB_REFERENCE: &str = "README.md";

/// Every hazard the harness states because a defect is currently open, and the item that
/// will close it.
///
/// Declared rather than derived, because whether a sentence is *about* an open defect is a
/// question about meaning. What is derived is the other direction — every item id the
/// harness mentions — so the declaration cannot go stale in the way that flatters: a
/// hazard added without an entry here fails, and an entry whose sentence has gone fails
/// too.
///
/// The same defect is named in two places on purpose. A hazard belongs at the step where
/// it bites, and the step is in the procedure while the rule is in the contract.
/// Empty is a real state and not a disabled check.
///
/// `P10-STALE-WRITER` closed, and both files stopped naming it in the same commit that
/// emptied this list — which is the sequence these two tests exist to force. The direction
/// that still has teeth while this is empty is the other one: naming any item without an
/// entry here fails, so the list cannot be emptied to silence a warning that is still true.
const TEMPORARY_HAZARDS: &[(&str, &str)] = &[];

/// Every path the harness names that a fresh checkout deliberately does not have, and the
/// record that decided the absence.
///
/// Declared rather than derived, for the reason `TEMPORARY_HAZARDS` above is: whether a file
/// is absent *by design* is a question about meaning, and the absence cannot answer it. An
/// absence by design and an absence by accident are the same absence on disk, which is why
/// [`checks::Test_Every_Path_The_Harness_Names_Should_Exist`] needs to be told and cannot
/// work it out.
///
/// What is derived is the other direction, and it is what stops this from becoming the
/// blanket exemption that answers the new question by no longer asking the old one. An entry
/// whose file has since been committed fails, so the list cannot be used to cover a path that
/// is really there. An entry no harness file names any more fails, so an exemption cannot
/// outlive the sentence it was written for. An entry citing a record the tree does not carry
/// fails, so the authority has to be a real one.
///
/// `.cargo/xvpe-local.toml` is the first entry and the reason the list exists.
/// `OD-PLATFORM-004` decides that the local XVPE override is opt-in and never the governing
/// form, and `.gitignore` ignores `/.cargo/` whole so the name cannot be recreated. The
/// contract names the file in order to say it is not there, and that sentence is correct: it
/// was the check that could not tell a deliberate absence from a route to nothing.
const DELIBERATELY_ABSENT_PATHS: &[(&str, &str)] = &[(".cargo/xvpe-local.toml", "OD-PLATFORM-004")];

/// `OD-LEDGER-023`'s half of `P11-NEXT-WORK`'s `done_when`: once selection gains an
/// authority, the contract stops instructing the agent to pick and names the thing that
/// picks instead.
///
/// A standalone test rather than a case added to `checks.rs`'s registry — that file, like
/// `readers.rs` and `controls.rs`, sits under `agent_harness/` and outside this item's
/// territory, which reserves the single file `agent_harness.rs`. It calls the same
/// `readers::Read_Harness_File` every check in the registry uses, so it is one opinion about
/// the same text rather than a second reader of it.
#[test]
fn Test_The_Contract_Should_Name_The_Selection_Authority_Rather_Than_Instruct_Picking()
{
    let contract = readers::Read_Harness_File(CONTRACT);

    assert!(
        !contract.contains("Pick an item"),
        "AGENTS.md still tells the agent to pick by eye. `nomos_ledger::Eligible_Items` \
         computes the real answer now (`OD-LEDGER-023`), and the contract must route to it \
         rather than to a model's judgment"
    );
    assert!(
        contract.contains("next:"),
        "AGENTS.md's step 2 must name the mechanism that replaced picking -- `nomos work \
         list`'s `next:` line -- not merely stop saying \"pick\""
    );
}

/// A citation a harness file makes of a specific numbered step in `AGENTS.md`'s loop,
/// paired with the fragment of that step's own wording the citing prose depends on.
///
/// The shape is `AGENTS.md step N: "anchor"`, deliberately not delimited by backticks as a
/// whole span -- the existence check that walks every code span would otherwise try to
/// resolve the citation as a repository path the way it resolves every other one.
fn Step_Citations(text: &str) -> Vec<(u32, String)>
{
    let marker = "AGENTS.md step ";
    let mut citations = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find(marker)
    {
        let after_marker = &rest[start.saturating_add(marker.len())..];
        let (citation, remainder) = Parse_One_Citation(after_marker);
        rest = remainder;
        if let Some(citation) = citation
        {
            citations.push(citation);
        }
    }

    return citations;
}

/// One citation parsed from the text immediately after an `AGENTS.md step ` marker, paired
/// with the text remaining once the citation's digits (and, if well-formed, its whole quoted
/// anchor) are consumed.
///
/// [`None`] in the first slot when the digits do not resolve to a well-formed citation -- the
/// remainder still advances past the digits, so the caller's search for the next marker
/// cannot loop on the same text.
fn Parse_One_Citation(rest: &str) -> (Option<(u32, String)>, &str)
{
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let rest = &rest[digits.len()..];

    let Some(step) = digits.parse::<u32>().ok()
    else
    {
        return (None, rest);
    };
    let Some(after_colon) = rest.strip_prefix(": \"")
    else
    {
        return (None, rest);
    };
    let Some(end) = after_colon.find('"')
    else
    {
        return (None, rest);
    };

    return (Some((step, after_colon[..end].to_owned())), &after_colon[end.saturating_add(1)..]);
}

/// One numbered step of `AGENTS.md`'s loop, its continuation lines rejoined into one string.
///
/// The loop is the only place in the contract a bare `N. ` opens a line -- checked once,
/// above `Is_Loop_Step_Marker` -- so finding the marker anywhere in the text is sound rather
/// than a coincidence this function trusts blindly.
fn Loop_Step_Text(contract: &str, step: u32) -> Option<String>
{
    let marker = format!("{step}. ");
    let mut lines = contract.lines().skip_while(|line| return !line.starts_with(marker.as_str()));
    let first = lines.next()?;
    let mut collected = vec![first[marker.len()..].to_owned()];

    for line in lines
    {
        if line.trim().is_empty() || line.starts_with('#') || Is_Loop_Step_Marker(line)
        {
            break;
        }
        collected.push(line.trim().to_owned());
    }

    return Some(collected.join(" "));
}

/// Whether a line opens the next numbered step, which is where [`Loop_Step_Text`] stops
/// collecting continuation lines.
fn Is_Loop_Step_Marker(line: &str) -> bool
{
    let digits: String = line.chars().take_while(char::is_ascii_digit).collect();

    return !digits.is_empty() && line[digits.len()..].starts_with(". ");
}

/// `OD-AGENT-003`'s second path, for a procedure that must carry a contract step in its own
/// words to stay usable: the citation is the promise, and this is what holds it to the step
/// it names.
///
/// [`Test_The_Contract_Should_Name_The_Selection_Authority_Rather_Than_Instruct_Picking`]
/// above is this same defect's one already-fixed instance, asserted by hand because nothing
/// generic existed yet. This is the generic form -- it does not stop working once that
/// sentence is edited again, and it holds any citation any skill makes of any step, not
/// only step 2's.
///
/// Fixture-driven for the same reason every derived reader in this suite is: a check whose
/// failing case has never been observed is `OD-GATE-001`'s defect wearing a new file name.
#[test]
fn Test_A_Cited_Contract_Step_Should_Still_Say_What_Is_Quoted_From_It()
{
    Assert_Uncited_Prose_Reads_No_Citations();
    Assert_Well_Formed_Citation_Reads_Back();

    let fixture_contract = Fixture_Loop_Contract();
    Assert_Fixture_Step_Carries_Its_Own_Wording(fixture_contract);
    Assert_Editing_The_Fixture_Changes_What_Is_Read_Back(fixture_contract);

    let broken = Citations_Broken_Against_The_Live_Contract();

    assert!(
        broken.is_empty(),
        "a harness file quotes a contract step that has moved on without it: {broken:#?}.\n\
         OD-AGENT-003 admits a procedure carrying a step in its own words only paired with a \
         citation in exactly this shape, so a paraphrase and the step it paraphrases are held \
         together mechanically rather than by a reviewer noticing the drift."
    );
}

/// Prose with no `AGENTS.md step ` marker at all must read back as citing nothing.
fn Assert_Uncited_Prose_Reads_No_Citations()
{
    assert_eq!(
        Step_Citations("no citation appears in this prose at all"),
        Vec::<(u32, String)>::new(),
        "prose with no citation marker was read as citing a step"
    );
}

/// A well-formed citation must read back its step number and quoted anchor.
fn Assert_Well_Formed_Citation_Reads_Back()
{
    assert_eq!(
        Step_Citations("see AGENTS.md step 2: \"picked by eye\" for why"),
        vec![(2, "picked by eye".to_owned())],
        "a well-formed citation was not read back out of the prose that carries it"
    );
}

/// A small loop fixture carrying a step 2 whose wording the two assertions that follow it
/// depend on: that the reader finds the wording, and that it stops finding it once edited.
fn Fixture_Loop_Contract() -> &'static str
{
    return "## The loop\n\n\
        1. Read the board.\n\
        2. Read the board. `nomos work list` names the item to claim next, computed rather\n   \
           than picked by eye.\n\
        3. Claim it.\n";
}

/// The fixture's own step 2 must carry "picked by eye", or the reader below cannot be
/// trusted to have found it in the live contract either.
fn Assert_Fixture_Step_Carries_Its_Own_Wording(fixture_contract: &str)
{
    let live = Loop_Step_Text(fixture_contract, 2).expect("the fixture names a step 2");

    assert!(
        live.contains("picked by eye"),
        "the fixture's own step 2 carries \"picked by eye\" and the reader did not find it: \
         {live}"
    );
}

/// Editing the fixture's step 2 wording must be visible to the reader, or a real edit to
/// `AGENTS.md` would never be seen by this check either.
fn Assert_Editing_The_Fixture_Changes_What_Is_Read_Back(fixture_contract: &str)
{
    let edited_contract = fixture_contract.replace("picked by eye", "chosen by a session");
    let after_edit =
        Loop_Step_Text(&edited_contract, 2).expect("step 2 still exists after the edit");

    assert!(
        !after_edit.contains("picked by eye"),
        "the edited fixture still reads back the pre-edit wording, so a real edit to \
         AGENTS.md would never be seen by this check"
    );
}

/// Every citation any harness file makes of a live `AGENTS.md` loop step, checked against
/// what that step currently says -- one message per citation whose anchor the step no
/// longer carries, or that names a step the loop no longer has.
fn Citations_Broken_Against_The_Live_Contract() -> Vec<String>
{
    let contract = readers::Read_Harness_File(CONTRACT);
    let mut broken = Vec::new();
    for (file, text) in readers::Harness_Files()
    {
        for (step, anchor) in Step_Citations(&text)
        {
            match Loop_Step_Text(&contract, step)
            {
                Some(current) if current.contains(anchor.as_str()) =>
                {}
                Some(current) => broken.push(format!(
                    "{file} cites {CONTRACT} step {step} for \"{anchor}\", which step {step} \
                     no longer says -- it now reads: {current}"
                )),
                None => broken.push(format!(
                    "{file} cites {CONTRACT} step {step}, and {CONTRACT}'s loop has no such \
                     step any more"
                )),
            }
        }
    }

    return broken;
}

