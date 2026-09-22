# Working this repository as an agent

This file answers **where the truth is and how to act on it safely**. It does not answer
what the truth is. Nothing here is a summary of another file, because a summary of a
checked file is an unchecked copy of it, and this repository has already paid for one —
`docs/records/OD-AGENT-001-an-agent-instruction-file-routes-to-authority-rather-than-restating-it.md`
records why, and `tests/contract/tests/agent_harness.rs` keeps this file honest about it.

Read the authority. Do not infer the architecture from the code nearest to your task.

## Which authority answers which question

| Question | Read |
|---|---|
| What exists, what owns what, which band may depend on which | `README.md` |
| Is that still true? | `tests/contract/` — it asserts the README's tables against the real workspace, both directions |
| Why was it decided that way? | `docs/records/` — one record per decision, and they are canonical |
| Which of the four products owns this responsibility? | `docs/records/ARC-ECOSYSTEM-001-four-products-share-one-seam-and-ownership-is-decided-by-semantics.md` |
| What work is available, claimed, blocked, or already refused? | `work/ledger.json`, through the `nomos work` verbs the README documents |
| What must pass before I finish? | the claimed item's own verification predicate, then `.github/workflows/gate.yml` |
| What is canonical specification content? | the specification store, not any file rendered out of it |
| What formatting and lint rules apply? | `README.md`, `rustfmt.toml`, `clippy.toml` |
| Which files am I allowed to change? | the territory of the item you hold, and nothing else |
| What does a handoff carry when my context runs out before an item is done? | `docs/records/OD-AGENT-002-a-handoff-carries-session-local-state-and-routes-to-authority-for-everything-else.md` |

When two of those disagree, the mechanical one wins and the disagreement is a defect worth
an item.

## The loop

1. `git log --oneline -5` and `git status`. Other sessions are working this tree right now,
   so any state described to you may already be false.
2. Read the board. `nomos work list` prints the live board and names the item to claim next
   on its `next:` line, computed from every item rather than picked by eye — `--all` adds the
   rows that have ended. Add one if it names none; an item that reserves nothing is refused,
   because it would exclude nobody while looking like work.
3. Claim it. **Exit 0 is the only thing that means you have it.** The listing column is a
   snapshot and means nothing seconds later.
4. Read only the authorities your item needs.
5. Implement inside your territory. Make the smallest change that satisfies the item's
   `done_when`, and do not introduce a second authority for something already governed.
6. Run the item's predicate yourself, plus the tests your change actually reaches.
7. Finish through the ledger. Do not hand-edit an item to Done.
8. Commit the paths you touched — they go on `git add`, never `git add -A`. Re-read the board.

A decision that outlives your item belongs in a record, not in a comment and not in this
file. A record is registered by adding a file under
`crates/spec/nomos-spec-store/records/`; writing the document alone does not make it
governing.

## Operating hazards

These are the facts that have no other home. Each is a rule for working here, not an
architectural claim; where a *why* exists, it is named.

**The XVPE crossing is a pinned revision, not a checkout beside this repository.** The root
manifest declares every `xvpe-*` crate at one revision and each member inherits it, so the
workspace resolves XVPE from that revision and needs no sibling directory to build. A local override does
exist and is opt-in: `.cargo/xvpe-local.toml` substitutes a sibling checkout, is deliberately
not named so cargo cannot discover it, and is never the governing form (`OD-PLATFORM-004`).
Requesting it rewrites `Cargo.lock`, so the build then answers to whatever that checkout sits
at, and no number taken under it is evidence about what this repository publishes. How the
crossing is pinned is measured by `tests/contract/tests/boundaries/lock_pinning.rs`.

**Never run `cargo fmt`.** It cannot produce this workspace's style and rewrites the tree
every time. `README.md` and `rustfmt.toml` carry the reason. `cargo fmt --check` is
deliberately not a gate step.

**Build the binary fresh, and re-copy it after every pull.** A `nomos` built before a ledger
field existed cannot write that field back. A build carrying the guard refuses the whole
verb and says so; one copied before it carries no guard and drops the field silently at
exit 0. `nomos work validate` prints the file's schema beside the build's, which is how you
tell which copy you are holding.

**Do not run a ledger verb through `cargo run` when its predicate is `cargo test`.** The
rebuild deletes the binary that is the running process; the predicate then fails for a
reason that looks like your tests failing. Run a copy of the built binary instead.

**The ledger is global coordination state, shared with live sessions.** Territory is
declared, not enforced — nothing stops a write into a claimed path, so the claim only
works if everyone respects it. Never widen or rewrite another holder's territory. If a
shared check goes red for a reason that is not yours, it is probably somebody else's
in-flight work: wait and retry rather than editing their file to unblock yourself.

**Branch on exit codes, not on output.** The refusal and success lines share vocabulary, so
a text match reads a refusal as a success. Retryable and fatal are different codes for a
reason; `README.md` has the table.

**Write commit messages to a file and use a bare `git commit -F`.** The shell splits long prose
into arguments and the commit fails after `git add`; a pathspec commits the tree, not the index.

**A test that cannot find its corpus passes.** Three corpora live outside this repository
and CI has none of them, so a green run is not evidence a corpus-backed claim was checked.
`tests/contract/` declares the size of that hole; read it before believing a number.

**Changing a governing record leaves every committed projection stale, and the gate fails on
them.** `diagrams/relations.mmd` and `spec/domain-specification.md` are both derived from the
record set, so a commit touching `docs/records/` changes both whether or not it meant to.
Re-render **both** before committing — the gate requires each one, so rendering only the
diagram leaves the other stale:
`nomos spec render --profile diagram-set --into .` and
`nomos spec render --profile domain-specification --into .`. Render last, and only from a tree
whose registered records are exactly the ones your commit publishes — not the tree you are
sitting in. `.claude/skills/nomos-spec-change/SKILL.md` has the four commands that construct
it; `docs/records/OD-GATE-005-a-derived-projection-is-owned-by-nobody-and-is-rendered-from-the-record-set-its-commit-publishes.md`
is why the shared tree is never that tree.

## What this file is not

It is not a place to record architecture. If you find yourself about to paste a table of
crates, a list of gate commands, or the ledger verb reference into this file, the thing you
actually want is a link to the file that already holds it — and if no file holds it, the
change belongs there rather than here.

It is not a task list either. `work/ledger.json` is the only board. Do not create a
parallel one under a tool's own directory.

Agent-specific mechanics live in the adapter for that agent: `CLAUDE.md` for Claude Code.
The contract in this file is the same for all of them.
