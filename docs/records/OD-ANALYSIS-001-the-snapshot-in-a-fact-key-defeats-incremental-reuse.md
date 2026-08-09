---
id: OD-ANALYSIS-001
type: decision
title: The snapshot in a fact key is a third answer, and it defeats the other two
status: closed
version: 2
authority: canonical-normative-record
tags:
  - analysis
  - fact-identity
  - incrementality
relations:
  - target: D-129
    type: affects
---

# The snapshot in a fact key is a third answer, and it defeats the other two

## Question

`FactKey` names ten components, and one of them is a `SnapshotId` — the identity of a
pinned workspace state. `nomos-workspace` now produces real ones. Composing the two raises
a question neither crate could ask alone: which workspace state does a fact key name, and
what happens to the store when the workspace moves?

## What Was Found

A `WorkspaceSnapshot` identity is a digest over *every* member. Editing one file therefore
changes it, and a fact key carrying it changes with it — for every fact in the corpus, not
just the edited file's.

The vertical slice was wired to take `workspace.Id()` at materialization time, which is the
obvious reading of "the context comes from the workspace". Over the six-file precision
corpus, editing `alpha/one.rs` then recomputed:

```text
nomos.cap.module.surface of alpha        nomos.cap.syntax.items of alpha/one.rs
nomos.cap.module.surface of beta         nomos.cap.syntax.items of alpha/two.rs
nomos.cap.module.surface of gamma        nomos.cap.syntax.items of beta/four.rs
                                         nomos.cap.syntax.items of beta/three.rs
                                         nomos.cap.syntax.items of gamma/five.rs
```

Eight facts, where two changed. At the scale corpus that is 9,440 facts recomputed per
keystroke, and incremental analysis has stopped existing.

## Why It Is A Third Answer

Two components already answer the questions the snapshot is there for, and they answer them
better.

**What a fact is computed from** is `semantic_inputs` — for a syntax fact, the file's
content. The workspace's contribution to a leaf fact's identity is already in there, exactly
and no more coarsely. The snapshot adds every *other* file in the workspace to that
computation.

**Which analysis state a fact is current at** is the generation. `MemoryFactStore` already
holds a validity interval per fact: materialized at a generation, invalidated at another,
and `Current` compares against the generation being asked about. That mechanism survives an
edit, which is the whole point of it.

The snapshot is a third answer, it is the coarsest of the three, and coarsest wins. It does
not add a distinction; it erases the two finer ones underneath it.

`tests/integration` asserted the erasure directly. One file, byte-identical in two corpora,
equal `semantic_inputs`, unequal key digests, with the snapshot named as the only component
that differed. That test is still there and now asserts the opposite — see *What Closed It*.

## What The Slice Did In The Meantime

It pinned. `Slice` recorded the snapshot the store was opened against, edits advanced the
generation through `Workspace::Apply`, and the pinned snapshot did not follow. That
restored exact-descendant recomputation and was honest about being a workaround.

Pinning was also the sharpest statement of the problem. The snapshot component had two
available settings and neither earned its place: pinned, it never varied and
`GenerationCause::SnapshotReplaced` could never fire, so it was dead weight in every key;
live, it destroyed all reuse. There was no configuration in which it did work the other two
components were not already doing.

## What Closed It

`FactKey` lost its `snapshot`, and `Component::Snapshot` with it. A fact's relation to a
workspace state is now `MaterializedFact::snapshot` — provenance, the tree a measurement was
taken from, recorded beside the fact rather than folded into what it is. It is deliberately
not restamped when a fact is reused: a reused fact is not a repeated observation.

`GenerationCause::SnapshotReplaced` was re-expressed. It used to name the new snapshot and
match it against a key component, which is why it was `WholeWorkspace`-granular — it could
not say anything narrower than "everything filed under the old state". It now carries
`from`, `to`, and the set of subjects that differ, which is what a caller replacing a
workspace state already has: two content-addressed maps of path to digest, and the paths
where they disagree. That makes it `File`-granular, and strictly narrower than what it
replaced — a checkout of two files invalidates two files and the rollups that read them.

An empty `differing` set is not refused. Two states can hold identical members and differ in
variant or configuration, and those have causes of their own. `Describe` reports the count,
so a caller whose diff iterated zero times reads "0 member(s) differ" rather than a clean
result.

The slice no longer pins. `Slice` holds the workspace's current identity — taken from the
`Applied` the door itself reports, at each of the three places a workspace can change — and
`Test_The_Fact_Context_Should_Come_From_The_Workspace` asserts it never disagrees with
`Workspace::Id`. Holding it rather than asking is not a pin but it is not free either:
`Workspace::Id` digests every member, and deriving it per fact key re-encoded 876 KB of
manifest eleven thousand times per run, which cost seventy seconds of a nine-second suite
before the value was held.

The measurement that opened this record was inverted rather than deleted.
`Test_An_Unchanged_File_Should_Keep_Its_Identity_Across_Workspace_States` takes the same two
corpora, the same byte-identical file, and now asserts the key digests are *equal* — with a
negative control beside it, because an equality that holds because keys ignore their subject
would be worthless.

## What Was Considered And Rejected

**Leaving the slice on the live snapshot and accepting the recomputation.** It would keep
the composition honest at the cost of making every incrementality assertion in
`tests/integration` unwritable — the exact-descendants property is the one thing the
precision corpus exists to state, and a slice that cannot state it is not measuring the
kernel any more.

**Treating a workspace snapshot as something other than what `FactKey.snapshot` means.**
There is no second candidate. `SnapshotId` is documented as "the identity of a pinned
workspace state" and `WorkspaceSnapshot` is exactly that; inventing a coarser second notion
of snapshot to put in the key would be a fourth answer.

## Status

Closed by P8-PIN. Three controls were confirmed red before the change was kept: folding the
workspace state back into a fact's identity (six tests, including the scale reuse claim), a
replacement that names no differing member, and a held workspace identity that stops
following the workspace — the pin, wearing a cache.
