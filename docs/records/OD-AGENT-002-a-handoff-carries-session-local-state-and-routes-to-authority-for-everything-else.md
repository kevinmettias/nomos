---
id: OD-AGENT-002
type: decision
title: A handoff carries session-local state and routes to authority for everything else
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - documentation
  - agents
  - ledger
relations:
  - target: OD-AGENT-001
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
---

# A handoff carries session-local state and routes to authority for everything else

## Question

A session that exhausts its context has two places to put what it knows, and neither takes
it. The ledger holds the claim, the territory and the predicate — what a successor needs in
order to start — but not what this session learned and has not yet earned a record for: the
hypothesis it ruled out, the command that hangs, the reading of a record that turned out to
be wrong. The records hold decisions, and none of that is a decision yet.

So the state goes into a prose handoff, and the shape a handoff takes by default is a
summary of the repository: the crate bands, the gate commands, the ledger verbs, the
hazards. `OD-AGENT-001` already refused that document once, for the reason that a summary
of a checked file is an unchecked copy of it, and `tests/contract/tests/agent_harness.rs`
keeps `AGENTS.md` honest about it. A handoff prompt is the same document at a different
address, written by a machine at the moment it is least able to check what it is copying —
mid context-exhaustion, past the point where it would re-read the file it is paraphrasing —
and read by a successor with no way to tell the copy from the authority it was copied from.
The successor cannot even fall back to distrust: an instruction file it did not expect
reads as legitimate.

## The Same Defect At A Different Address

`OD-AGENT-001`'s criterion was: a sentence belongs in an agent-facing file exactly when no
existing authority would be the better place to read it, and no mechanical check already
asserts it. A handoff that lists the crate bands, the gate steps or the ledger verbs fails
that criterion for the identical reason `AGENTS.md` would — `README.md`, `.github/workflows/
gate.yml` and the ledger verb reference in `README.md` already hold each, checked against
reality, and a copy inside a handoff is checked against nothing. The only thing different
about a handoff is that nobody wrote it down as a place restatement could happen, because a
handoff was never a committed file to begin with.

The compression direction cuts the other way from what a summary normally does. Agent-to-
agent prose is worth compressing — the connective English, the scene-setting, the parts a
human handoff would spend a paragraph on — but the qualifiers are the expensive part to
lose. *Unverified*, *failed*, *assumed*, *not run*, and the provenance of a claim (which
command produced it, against which file) are exactly what a summary drops first, because
they read as hedges rather than content. They are the content. A successor that receives
"the corpus test passes" where the session meant "the corpus test passes because no corpus
is present, per the note in `tests/contract/`" repeats the investigation that already
happened, or worse, trusts a conclusion nobody reached.

## The Decision

**A handoff carries session-local state, and nothing an authority already holds.**

Session-local state is what has no other home yet:

- What was tried and rejected — the hypothesis ruled out, the command that hangs, the
  reading of a record that turned out wrong — stated with its outcome, not just its topic.
- What was observed and not yet recorded — an error message, a fact about the running
  environment, a result nobody has filed as a record — carried with the qualifier that
  marks how sure it is: unverified, assumed, observed once, not reproduced.
- What the next action is — one concrete step, not a re-derivation of the plan that led to
  it.

Everything reconstructible from an authority is a reference to that authority, not a
paraphrase of it: an item id the ledger can look up, a record id the store can render, a
commit the log can show, a path a reader can open. Naming `P11-AGENT-CONTINUATION` costs
four tokens; restating what its territory and predicate say costs a paragraph that is
already stale the moment the item is edited.

Compression is directional and applies only to the English around these facts, never to the
facts themselves. A handoff may say "tried three approaches to the gate-command check before
landing on line-derived matching" in place of a paragraph narrating each attempt. It may not
turn "the corpus test passed because the corpus is absent, unverified against a populated
one" into "the corpus test passed" — the second sentence is shorter and false in exactly the
direction that costs a successor the most.

## Where This Lives

The constraint on content is agent-agnostic — any agent working this repository can exhaust
its context, not only this one — so it is a record, per `AGENTS.md`'s own routing table
rather than restated inside it: a policy about not duplicating authority cannot itself live
only in the instruction file it is about, which is the same objection `OD-AGENT-001` raises
against a documentation tree. `AGENTS.md` gains one routing row and no restatement of this
decision's reasoning.

`CLAUDE.md` gains one line under its list of obvious mistakes, because *how* a handoff gets
written — at compaction, at session end — is Claude Code's own mechanism, and mechanics are
what `AGENTS.md`'s closing line already assigns to the adapter file. The line names this
record and says a handoff must not restate the repository; it does not re-derive the content
policy CLAUDE.md would otherwise be duplicating this record to state.

## What Is Guarded, And What Is Not

`tests/contract/tests/agent_harness.rs` extends the same mechanism it already runs against
the crate-band table — a derived source of truth compared against every committed harness
file, in both directions the check can reach. Two more restatement shapes join the one
already caught: a handoff, or any harness file, that carries two or more of the gate's own
`run:` commands (derived from `.github/workflows/gate.yml`, not retyped into the test) is
carrying the gate command list; one that carries two or more of the `nomos work` verb lines
`README.md` documents is carrying the ledger verb reference. A single command or verb named
in passing — `AGENTS.md` already names `nomos work validate` once, as a routing example — is
not a restatement of either list, only a copy of both would be, and the threshold is set
where an example stops being an example.

What is not guarded is a handoff itself, because a handoff is not a committed file — it is
text a session writes into a prompt or a ledger note at the moment its context runs out, and
nothing mechanical reads that text before a successor does. What is guarded is the one
version of this content that is committed: `AGENTS.md` and `CLAUDE.md` naming this policy
without restating what it forbids, so the routing line a handoff would point a successor to
stays trustworthy.

## Status

Closed by `P11-AGENT-CONTINUATION`.
