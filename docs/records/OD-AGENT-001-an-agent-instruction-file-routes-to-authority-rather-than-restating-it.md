---
id: OD-AGENT-001
type: decision
title: An agent instruction file routes to authority rather than restating it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - documentation
  - agents
  - ledger
relations:
  - target: OD-PROJECT-001
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
---

# An agent instruction file routes to authority rather than restating it

## Question

Several coding agents work this repository, and each of them looks for a file by name
before it looks at anything else. None of those files existed. `CLAUDE.md`, `AGENTS.md`,
`.codex/` and `.github/copilot-instructions.md` were all absent, and the only thing under
`.claude/` was a `settings.local.json` that a global ignore keeps out of git — so the
committed agent-facing surface of this repository was empty.

The consequence is not that agents could not work here. They did. It is that each session
rediscovered the same four things from source: which crate sits in which band, what the
ledger verbs are, what the gate runs, and which operations damage the tree. The first
three are written down. The fourth is not written down anywhere, and it is the one that
costs a session an afternoon.

The obvious remedy is a documentation tree for agents — an architecture map, a workflow
page, a testing page, a conventions page. That remedy was proposed, and this record is why
it was not built.

## The Restatement Is The Defect

`tests/contract/tests/boundaries.rs` reads the band tables out of `README.md` and compares
them against the real workspace **in both directions**: a member missing from the table
fails the gate, and a table row naming a crate that is not a member fails the gate. That
is `OD-PROJECT-001`'s remedy for a hand-authored file that had already drifted — twenty-two
members described by eleven rows, and nothing to notice.

A `docs/agent/architecture-map.md` carrying those same bands would be a copy of a checked
file that is itself unchecked. The gate would stay green while it went stale, which is
precisely the state `OD-PROJECT-001` found the README in, reintroduced one directory over
and one year younger. The same holds for a workflow page beside the README's ledger
section, and for a testing page beside `.github/workflows/gate.yml`, which `work finish`
already derives its lint step from rather than copying — `P9-PREDICATE` made that choice
for this reason and `OD-LEDGER-003` records it.

An agent instruction file is subject to a stronger version of the same pressure than an
ordinary document, because nobody reads it. It is consumed by a machine at the start of
every session and reviewed by a person approximately never, so drift in it is both more
likely and less visible than drift in a page a human opens.

## The Decision

**The committed agent surface is `AGENTS.md`, a `CLAUDE.md` that imports it, and skills
under `.claude/skills/`. `AGENTS.md` answers where the truth is and how to act on it
safely. It does not answer what the truth is.**

The criterion, stated so the next paragraph somebody wants to add can be judged by it
rather than by taste:

> A sentence belongs in `AGENTS.md` exactly when no existing authority in this repository
> would be the better place to read it, and no mechanical check already asserts it.

Applied, that admits two kinds of content and rejects a third.

**Routing** is admitted. Which authority answers a question is a fact about this
repository that no single authority can state about itself — `README.md` cannot tell you
to prefer `docs/records` for rationale, and `docs/records` cannot tell you the board is in
`work/ledger.json`. Nothing else holds it.

**Operating hazards** are admitted. That `cargo fmt` damages the tree, that a stale copy
of the `nomos` binary rewrites `work/ledger.json` without the fields it does not know and
exits zero, that `work finish` runs the gate lint step before the item's predicate, that
the ledger is shared with sessions that are running right now — these are facts about
working here with an autonomous agent. Some have a *why* in a record or a source comment;
none has a *where* that an arriving agent would find first. They are stated as stable
operating rules, and a hazard tied to an open defect names the item that will close it, so
it can be removed rather than accumulating.

**Restatement** is rejected. No band table, no gate command list, no second copy of the
ledger verb reference, no conventions section. `AGENTS.md` names the file that holds each
and stops.

## The Boundary This Does Not Draw

The proposal that prompted this record also carried an ecosystem split — which product
owns knowledge, which owns runtime infrastructure, which owns engineering reality. It is
absent here deliberately. That boundary has not been decided by this repository: `KWB`
appears in this workspace five times, all incidental, and `xvpe` appears as a style
sibling and in `D-130`. Writing it into an agent instruction file would promote a design
conversation into normative architecture by way of a file nobody reviews, which is the
exact mechanism this record exists to prevent. It needs its own record first. When it has
one, `AGENTS.md` gains a routing line to it and no conclusions from it.

## What Is Guarded, And What Is Not

`tests/contract/tests/agent_harness.rs` asserts the structural promises rather than the
prose: that `CLAUDE.md` imports `AGENTS.md` instead of duplicating it, that every
repository path the harness names exists on disk, that `AGENTS.md` names the work and
rationale authorities, that no crate row derived from the README's bands is restated in
it, and that each committed skill carries frontmatter whose name matches its directory.
Each has a negative control over a fixture string, because a check whose failing case has
never been observed is `OD-GATE-001`'s defect in a new file.

What is not guarded is whether the routing is *correct* — whether the authority a line
names is really the one that answers the question. That is meaning, and this workspace
deliberately has no type that decides it. The check that a named path exists catches the
cheap half: a route to a file somebody moved.

The size limit is guarded too, and it is the mechanism rather than a style preference. An
`AGENTS.md` that grows past its declared line budget is one that has started restating
something, because routing does not need the room. The number is declared in the test with
its reason, in the shape `OD-GATE-001` uses: a figure somebody chose rather than a silence
nobody measured.

## Status

Closed by `P10-AGENT-HARNESS`.
