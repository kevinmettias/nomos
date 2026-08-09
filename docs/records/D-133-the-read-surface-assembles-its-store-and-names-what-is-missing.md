---
id: D-133
type: decision
title: The read surface assembles its store on every invocation, and an absence is not an empty answer
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - command-line
  - corpus
relations:
  - target: ARC-SPECDB-001
    type: affects
  - target: OD-GATE-001
    type: relates-to
  - target: OD-SPEC-001
    type: relates-to
---

# The read surface assembles its store on every invocation, and an absence is not an empty answer

## Context

Four phases built a content-addressed store, a preservation ledger, a bundle round trip and
fourteen projection profiles. The `nomos` binary had one group, `work`, and did not depend
on any specification crate. Phase 2's stated payoff — ask what the canonical domain model
said, verbatim, and get the real rows — was not answerable from a command line, and the
fourteen renderers had never been run by anything but their own tests.

Building the group forced a question the tree had no answer to: *which store does a read
command open?* Nothing persists one. Every `SpecificationStore` in this workspace is
`In_Memory`, built inside a test and dropped at the end of it. There is no database file, no
command that produces one, and `OD-SPEC-001` deliberately leaves the backend question open
while naming the JSONL bundle rather than the database as the portable authority.

The second question is worse, and it is `OD-GATE-001`'s finding one level up. Most of what
this system is about lives in the v14 authoring corpus, which is not in this repository and
is not on most machines. A read command over a store the corpus never reached returns
nothing — and nothing is exactly what it returns when the identifier is genuinely unknown.
The two answers are opposite. One means *configure a corpus*; the other means *you have the
wrong identifier*. They printed the same.

## Decision

**The store is assembled per invocation, from two layers.** This repository's governing
records are embedded in the binary and seeded unconditionally, so a machine with no corpus
can still ask what Nomos decided about itself. The v14 corpus is layered over them when
`--corpus` or `NOMOS_V14_CORPUS` names one. Nothing is written to disk; there is no cache,
no build step and no store file that can be stale.

Measured rather than assumed: assembling the whole corpus — 10 domain volumes, 2,533 blocks,
493 normative statements, 2,619 catalog nodes — takes about 0.2 s, and the embedded records
alone about 0.02 s. At that cost a store nobody has to remember to rebuild is worth more than
one that answers faster and can disagree with the corpus it came from.

**An absence is a value, not a smaller answer.** Every source that was expected and not read
produces a record naming four things: what is missing, where it was looked for, why it is not
there, and *what is consequently not in this store*. The fourth is the one a reader would
never think to ask for, and it is the one that stops a short answer from reading as a
complete one. A command whose answer is empty because something was missing says so and exits
`6`; `1` means the store is whole and does not hold it. Those are two different next actions
and they get two different codes.

This applies to rendering as well as to reading. `nomos-spec-project` already refuses to
render a section that selected nothing, and its message — declare `may_be_empty` if nothing is
the honest answer — is right over a whole store and wrong over one the corpus never reached,
where it sends a reader to edit a profile because of a variable that is not set. So an empty
section over an incomplete store is reported as the absence instead.

**Content goes to standard output and everything about it goes to standard error.** So
`nomos spec record --id D-129 > D-129.md` writes the record's bytes and nothing else, and the
file it produces hashes to what the store holds. A header line mixed into the content would
make "verbatim" a claim about the interesting part of the output rather than about the output.

**The commands are exercised by running the built binary.** The libraries under them have
been covered since Phase 1; the part that had never run once was argument parsing — the
dispatch from `argv`, the flags, the exit codes and which stream each answer goes to. A test
that calls `Run` directly proves none of that, because it is the caller that assembled the
command.

## Consequences

`nomos-cli` now names three specification crates. It is a composition root at band 90 and the
only place permitted to: the spec system sits beside the kernel rather than above it, and
nothing in the product may reach it directly.

The corpus variable is read in exactly one place — `main.rs` — and handed to the assembler as
data, together with the name it came from so an absence can say how to supply one. That is
what lets the whole read surface, absences included, be exercised on a machine that has no
corpus at all.

`crates/host/nomos-cli/tests/read_surface.rs` runs the binary sixteen times and reads no
corpus: it removes the variable from the child's environment so a machine that has one
behaves like a machine that does not, and builds a two-row corpus of its own where one is
wanted. It is therefore not one of the sixty-eight gates `OD-GATE-001` counts, and it
assembles the variable's name with `concat!` for the reason that file does — a test that names
a corpus variable without reading a corpus would inflate the measured hole rather than
measure it.

## Alternatives Considered

**Persist a database and have `spec` open it.** Rejected. Nothing builds one, so the first
command anybody ran would fail with instructions to run a command that does not exist; and a
store file is a fourth copy of the corpus that can silently disagree with the three that
already exist. The bundle, not the database, is this system's portable authority.

**Report a missing corpus as an empty result.** Rejected, and it is the defect this record
exists to prevent. Sixty-eight tests already report `ok` having read nothing; a read surface
that prints an empty table for a corpus it never had is the same defect wearing a different
hat.

**Fail when no corpus is configured.** Rejected. The governing records are embedded and there
are real questions answerable without a corpus — every one of this repository's own decisions
is one. A command that can never succeed on most machines is a command everybody learns to
work around, which is the mutually-contradictory-gate defect `rustfmt.toml` already records.

## What This Is Not

Not the authoring round trip. `D-129` decides that a record can be read out as markdown *and
the edit written back* as a transaction with a mandatory preview. This builds the first half
only, over a store that is still seeded from files. `P9-AUTHORING` carries the rest, and
nothing here should be read as narrowing it.
