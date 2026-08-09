---
id: OD-ANALYSIS-001
type: decision
title: The snapshot in a fact key is a third answer, and it defeats the other two
status: open
version: 1
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

```
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

`tests/integration` asserts the erasure directly:
`Test_An_Unchanged_File_Should_Still_Be_Re_Addressed_By_A_Change_Elsewhere` takes one file
that is byte-identical in two corpora, shows its `semantic_inputs` are equal and its key
digests are not, and names the snapshot as the only component that differs.

## What The Slice Does In The Meantime

It pins. `Slice` records the snapshot the store was opened against, edits advance the
generation through `Workspace::Apply`, and the pinned snapshot does not follow. That
restores exact-descendant recomputation and is honest about being a workaround rather than
a fix.

Pinning is also the sharpest statement of the problem. In the current design the snapshot
component has two available settings and neither earns its place: pinned, it never varies
and `GenerationCause::SnapshotReplaced` can never fire, so it is dead weight in every key;
live, it destroys all reuse. There is no configuration in which it does work that the other
two components are not already doing.

## What Would Close It

`FactKey` loses its `snapshot`, and a fact's relationship to a workspace state becomes
provenance recorded on `MaterializedFact` rather than a component of what the fact *is*.
`GenerationCause::SnapshotReplaced` then has to be re-expressed — it currently invalidates
by matching `key.snapshot`, which is a search that would have nothing to match — most likely
as a cause that names the members that differ between two snapshots, which is what a caller
replacing a snapshot actually knows.

That is a change to a sealed substrate crate and to the shape of every key already written,
so it is P8-PIN and not a patch to this item.

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

Open. The slice pins, the workaround is annotated at `Slice::pinned` with a pointer here,
and the substrate change is planned rather than performed.
