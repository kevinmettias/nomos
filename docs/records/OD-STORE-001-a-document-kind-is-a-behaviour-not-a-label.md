---
id: OD-STORE-001
type: decision
title: A document kind is a behaviour, not a label — and one of them was misnamed
status: closed
version: 1
authority: canonical-normative-record
tags:
  - document-store
  - naming
relations:
  - target: D-129
    type: affects
---

# A document kind is a behaviour, not a label — and one of them was misnamed

## Question

`nomos-workspace` records a workspace state — a map of path to content digest — into a
document store. It files it under `DocumentKind::Fact`, because `DocumentKind::Snapshot` was
already spoken for by the store's own commit manifest, and `Index::Derive` decodes every
document of that kind as one.

That works, and it makes "which documents in this store are workspace states" a question
about a schema string. Does the store gain a kind for a workspace state, or does one stay a
`Fact`?

## The Criterion

A `DocumentKind` is not a label. Two things in this store are decided by it and nothing
else:

**Authority.** `DocumentKind::Authority` maps each kind to exactly one of observed or
authored, and `Test_Every_Document_Kind_Should_Belong_To_Exactly_One_Authority` holds the
line. A kind whose authority differs from every existing kind's has earned its place by
that alone.

**Index behaviour.** `Index::Derive` treats one kind specially — it decodes it as a commit
manifest and fails the whole index if it does not decode. That is a structural promise the
store makes about a kind, and a kind that carries one cannot be shared.

So: a `DocumentKind` earns its place when the store must *behave* differently for documents
of that kind. Nothing else.

## The Answer

A workspace state stays a `Fact`.

It is observed, which is what `Fact` already means, so it makes no authority distinction.
The store makes no structural promise about it and does not need to — `nomos-workspace`
decodes its own bytes, and nothing in the kernel reads them. A `Workspace` kind would carry
a label and no behaviour.

The cost of adding one anyway is not the variant. It is that the next document shape has the
same argument available to it, and the one after that, until there is a kind per schema and
`DocumentKind` says nothing `SchemaId` did not already say — at which point the two things
that *are* decided by kind, authority and index behaviour, are being decided by a field that
has become a duplicate label.

## What Was Actually Wrong

The name.

`DocumentKind::Snapshot` never meant a snapshot. It meant a commit: the set of documents
written in one operation, tagged with the workspace state, build variant, configuration and
generation they were written under. `DocumentStore::Commit(&Snapshot)` is the give-away —
the method says what the argument is, and the argument's type disagreed.

That is the collision the question was really about, and it is fixed rather than documented.
`Snapshot` is `Commit`, `DocumentKind::Snapshot` is `DocumentKind::Commit`,
`nomos.snapshot.v1` is `nomos.commit.v1`, and `Snapshot::Of` is `Commit::Under` — a commit
is made *under* a workspace state. `SnapshotId` keeps its name, because it identifies a
workspace state and always did.

## The Defect The Name Was Hiding

`Index` held `snapshots: BTreeMap<SnapshotId, DocumentId>` and offered
`Snapshot_Document(SnapshotId) -> Option<DocumentId>`.

Read with the old names, that is a lookup and obviously correct: the snapshot document for a
snapshot. Read with the right ones, it says at most one commit is ever made under one
workspace state — and that is false. A workspace state is a tree. A commit is one write
against it. Analyzing a tree, recording what was found, then analyzing it again for
something else produces two commits and no edit between them.

The map kept the last. The first commit fell out of `Snapshots`, and with it every document
it recorded, which would have surfaced later as
`Test_Every_Document_Should_Be_Reachable_From_A_Snapshot` reporting documents nothing wrote.
No test caught it, because every fixture committed under a distinct `SnapshotId`.

It is now `BTreeMap<SnapshotId, BTreeSet<DocumentId>>` behind `Commits_Under`, which returns
a list rather than an `Option` for the reason the `Option` was wrong.
`Test_Two_Commits_Under_One_Workspace_State_Should_Both_Be_Reachable` states it.

This is the argument for the rename, made concretely: a wrong name does not merely read
badly, it makes a wrong assumption look obviously right.

## What Was Considered And Rejected

**Adding `DocumentKind::Workspace` anyway, for legibility.** It would put two kinds in the
enum that both concern workspace states — `Commit`, which names one, and `Workspace`, which
is one — and the confusion this record exists to remove would have acquired a second
spelling rather than lost its first.

**Keeping `Snapshot` and adding a store-level alias.** Two names for one type is the same
defect with an indirection in front of it.

## Status

Closed. The criterion — a kind earns its place by behaviour — is the part worth carrying
forward; it is what the next request for a document kind should be answered against.
