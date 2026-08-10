---
id: OD-SPEC-006
type: decision
title: The round trip is built, and docs/records remains the authoring substrate on stated grounds
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - specification-system
  - documentation-architecture
  - authoring
relations:
  - target: D-129
    type: affects
  - target: OD-SPEC-005
    type: relates-to
  - target: D-133
    type: relates-to
---

# The round trip is built, and docs/records remains the authoring substrate on stated grounds

## Decision

The markdown round trip `D-129` decided is built. A record is read out of the store as
markdown rendered from the store's own rows, an edit is written back as a transaction, and
the transaction's preview is mandatory because committing takes a preview value that nothing
outside the module can construct.

Markdown files under `docs/records` remain the substrate this repository authors in. That is
recorded here rather than fixed, on the grounds below, and `D-129` is amended at version 3 to
say so rather than to keep describing an arrangement the tree does not have.

## What Changed

Three things exist that did not.

`nomos spec markdown --id <node-id>` renders a record from the declared front matter and the
block rows. It never reads the ingested blob — a projection that read the blob back would be
an echo, and would prove nothing about whether the store holds enough to be a substrate.
`Test_Every_Governing_Record_Should_Project_To_Its_Own_Bytes` asserts byte-identity for every
governing record, and `Test_A_Projection_Should_Come_From_The_Rows_And_Not_The_Blob` points a
document at different bytes and shows the markdown unchanged.

`nomos spec preview` and `nomos spec commit` are the write half. An edit is staged, previewed
and committed in one transaction: block surrogates survive, a rename updates the document row
rather than minting a new one, the declared relations move the graph in both directions, and a
refusal writes nothing.

The preview names whether normative wording moved, which `D-129` calls mandatory. It answers
from the blocks — the normalizer decides whether two spellings are one wording, so a reflowed
paragraph reports as reflowed rather than as reworded — and from the `normative_statements`
rows where a corpus has provided them. It says which of the two it used, because a preview
that reported only the statement-level answer would print nothing at all for this
repository's own records, and nothing reads as *no*.

## Why The Files Remain The Substrate

Nothing persists a specification database. `nomos-cli`'s `corpus.rs` opens with that sentence
and it is still true: the store is assembled on every invocation from records compiled into
the binary by `include_str!`, plus a corpus that lives outside this repository. So a commit
against the store is checked by a real transaction and then thrown away with the store, and
what survives an invocation is the file.

Two consequences worth stating plainly, because a reader who assumes otherwise will be
wrong in a way nothing warns them about. A record edited through this surface is re-seeded
from the copy compiled into the binary until `nomos-spec-store` is rebuilt. And the commands
say so on every run rather than leaving it to this record.

What is different from the arrangement `D-129` version 2 described is the *direction of the
evidence*, not the direction of the dependency. Before, the store held a copy of unknown
fidelity: the seed said identity was stable and nothing checked that the store could produce
the document again. Now the store is a projection whose fidelity is asserted over every
record in it, both ways — the file renders from the rows, and an edit that the rows could not
reproduce is refused rather than rewritten.

## What Would Have To Change

For the store to become the durable substrate, the seed would have to stop reading
`include_str!` and read something committed to the repository that is not the record files —
the bundle `nomos-spec-bundle` already produces is the candidate — and the files under
`docs/records` would become rendered outputs with a freshness check, the way `D-128` governs
every other maintained document.

That is not done here, and one specific obstacle is already recorded. `OD-SPEC-005` refuses
deriving the seeded record list from the directory, because
`Test_Every_Canonical_Record_On_Disk_Should_Be_Governing` would then compare a directory
against itself; the same shape applies to seeding from the files it is meant to govern. A
substrate move therefore has to arrive with its own check, and inventing one here would put
an unexamined guard under the only assertion that catches a governing record living outside
the store.

## Consequences

An author edits a file, and now has a command that says what the edit changes before it is
made, in a vocabulary the store can defend: which blocks moved, which wording changed, which
relations the graph gained and lost.

`D-129`'s first paragraph is unaffected and was never in doubt: identity lives in the store, a
`node_id` travels in a bundle, and a path is navigation. Retyping `id:` in a staged file is
refused for exactly that reason.

The surface refuses documents it cannot reproduce rather than normalising them. A v14 record
carrying a byte order mark (`D-131`) or spelling a relation key `relation` is readable,
hashable and preservable, and is not editable here until that is settled. Refusing is the
conservative half of the round trip: every refusal is a document this surface declines to
author, and every case wrongly allowed is a record whose bytes changed for a reason nobody
recorded.

## Alternatives Considered

Leaving `D-129` version 2 in place was rejected. It says the round trip does not exist, which
is now false, and a governing record that describes an absent capability is the failure mode
this build is organised against — in the other direction this time, which is not better.

Declaring the store the substrate and treating `docs/records` as generated was rejected for
now on the evidence above rather than on preference. It is a phase of work with a guard of its
own to design, and asserting it while the seed still compiles the files into the binary would
make this record as inaccurate as the one it amends.
