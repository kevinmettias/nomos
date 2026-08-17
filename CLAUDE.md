# Claude Code in this repository

@AGENTS.md

`AGENTS.md` is the operating contract and it is not repeated here. This file holds only
what is specific to Claude Code.

## Skills

Procedural skills are committed under .claude/skills and are discovered automatically —
they are not listed here, because a list of them would be a second copy of what the
directory already says. A skill earns its place only when it encodes a reusable multi-step
procedure that cannot be reduced to one command plus a pointer to an existing authority.
Anything short of that is a routing line in `AGENTS.md`.

The file .claude/settings.local.json is personal and stays out of git. Project settings, if they
ever arrive, are for mechanical tool policy only — never for architectural knowledge, which
belongs to the records and the contract tests.

## Subagents

Work is decomposed by territory, not by topic, because territory is what the ledger can
prove disjoint. Give each worker exactly one claimed item and nothing else: its id, its
territory, the authorities its task actually reaches, and its verification predicate. A
worker that receives the whole architecture will spend its context rediscovering what its
item did not need.

Workers return a summary — what changed, what was run, what refused — not a transcript.

Two workers may run at once only when the ledger grants both claims. It refuses an
unanswerable overlap rather than guessing, so a refusal is an answer and not an obstacle to
route around.

## The obvious mistakes

Do not create a task list under .claude or anywhere else. `work/ledger.json` is the
board, and a second one is a second authority.

Do not `Write` a file you have not read in this session. Another session may have changed
it since your task was described to you.

Scope every commit to explicit paths. `git add -A` in this tree sweeps up work that belongs
to somebody else.

Do not write a handoff, at compaction or at session end, that summarizes the repository.
`docs/records/OD-AGENT-002-a-handoff-carries-session-local-state-and-routes-to-authority-for-everything-else.md`
says what it carries instead.
