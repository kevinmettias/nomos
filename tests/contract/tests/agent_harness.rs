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
/// The fifth is a whole record rather than a directory, and is the one entry here that is
/// not satisfied by its neighbours. `docs/records` already matches any record path, so the
/// ownership row could be deleted without failing anything while the generic rationale row
/// survived — and the two questions are different. *Why was it decided that way* is answered
/// by whichever record argued the case; *which product owns this* is answered by one record
/// and getting it wrong is how the scope drift `ARC-ECOSYSTEM-001` exists to stop happens
/// again. Naming the path also puts it under the existence check, so renaming the record
/// without fixing the route fails here rather than in a reader's hands.
const ROUTED_AUTHORITIES: &[&str] = &[
    "README.md",
    "tests/contract",
    "docs/records",
    "docs/records/ARC-ECOSYSTEM-001-four-products-share-one-seam-and-ownership-is-decided-by-semantics.md",
    BOARD,
];

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

