---
id: OD-GATE-005
type: decision
title: A derived projection is owned by nobody, and is rendered from the record set its commit publishes
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - projection
  - freshness
  - ledger
  - territory
relations:
  - target: OD-GATE-001
    type: relates-to
  - target: OD-GATE-004
    type: relates-to
  - target: OD-PROJECT-001
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
  - target: OD-LEDGER-007
    type: relates-to
  - target: OD-LEDGER-011
    type: relates-to
  - target: ARC-SPECDB-001
    type: relates-to
---

# A derived projection is owned by nobody, and is rendered from the record set its commit publishes

## Why This Identifier

`OD-GATE-003` is not free: `P10-VACUITY-HOME` reserves both halves of it, and `OD-LEDGER-016`
decided that a record identifier and the file it names are one subject. `OD-GATE-004` is spent.
So this is 005, for the same reason 004 was not 003.

## Question

The gate was red on `dev` for eleven commits — `401b2d0` through `a1b821e` inclusive — at the
`Required projections` step, and every agent that looked at it was right not to fix it.

`401b2d0` registered `OD-GATE-004` and did not re-render `diagrams/relations.mmd`. The obligation
to re-render landed afterwards, in `ea3c167`, so the commit that broke it predates the sentence
telling authors not to. That is the ordinary way a hazard gets documented and it leaves one
artifact broken behind it. Two further commits landed records on top.

`P10-DIAGRAM-OWED` asked which of two things closes the recurrence:

- the diagram is reserved by whichever item adds a record, which means saying so where item
  territories get authored; or
- something derives the obligation, the way `OD-LEDGER-003` has `work finish` derive the gate's
  lint argv from `.github/workflows/gate.yml` rather than restating it.

**Both were tried against the tree and both are wrong**, and the reason is the same one in each
case. The question is not who is allowed to write the file. It is which trees can compute its
correct contents, and the answer is *almost none of them*.

## What Was Measured

Taken on 2026-08-10 across two trees at once: the shared working tree of this repository, and a
detached `git worktree` at `a1b821e` with nothing uncommitted in it.

`crates/spec/nomos-spec-store/build.rs` assembles `RECORDS` by reading
`crates/spec/nomos-spec-store/records/` **from disk** and embedding each named document with
`include_str!`. So the record set a binary holds is the record set that was on disk when it was
built, and it has never been the record set at `HEAD` except by luck.

| | working tree | worktree at `a1b821e` |
|---|---|---|
| governing records embedded | **50** | **49** |
| blocks | 1709 | 1642 |

The one record of difference is `OD-LEDGER-012`, held and uncommitted by another session for the
whole of this item's claim. That is the tree's normal condition, not an unusual one.

Rendering in the clean worktree and comparing the same command in both trees:

| tree | `spec freshness --into . --require diagram-set` | exit |
|---|---|---|
| worktree at `a1b821e`, before render | `stale`: committed over `930a77fe…`, store holds `955f33de…` | 8 |
| worktree at `a1b821e`, after render | `diagram-set: diagrams/relations.mmd is current` | **0** |
| working tree, **same rendered file** | `stale`: built over `955f33de…`, store holds `c535698d…` | **8** |

The third row is the whole of this record. **The diagram is correct, and the gate's own command
reports it stale, because the tree it ran in holds a record that is not published yet.** Nothing
the author of the diagram can do changes that verdict; it is a statement about somebody else's
uncommitted file.

Three renders were taken over the life of this item, and the set is the evidence. At `a1b821e`,
over `HEAD` alone, the diagram moved Relations 236 → 270 and Decisions 44 → 47, restoring
`OD-GATE-004`, `OD-SYNTAX-002` and `OD-TRACE-001`; over the same `HEAD` plus this record, 286 and
48. The one committed here was taken at `56d8205` plus this record — 55 governing records,
Relations 236 → **332**, Decisions 44 → **52** — because `HEAD` moved eleven times while the item
was held and a render is only ever current for the record set it was taken over.

**None of the three contains `OD-LEDGER-012`**, which is what distinguishes a render taken
correctly from one taken at a quiet moment. That record sat on disk in the shared tree for the
entire claim, and is absent from every output because each tree was constructed rather than found.

And the derivation option was measured rather than assumed. `nomos_ledger::Derive_Step` is generic
over the step name and the freshness step's `run:` is a bare argv, so it derives today:

```text
Lint                 => Ok(["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
Required projections => Ok(["cargo", "run", "--quiet", "-p", "nomos-cli", "--", "spec", "freshness", "--into", ".", "--require", "diagram-set"])
Rules                => Ok(["cargo", "run", "--quiet", "-p", "nomos-cli", "--bin", "nomos", "--", "check", "--root", "."])
```

So option two is available, cheap, and one line of wiring away. It is refused below on merit.

### How often the working tree can answer the question: measured, not assumed

The argument in decision 4 turns on how rare a renderable working tree is, so it was counted
rather than estimated. A one-minute poll ran for **sixty consecutive minutes** on 2026-08-10,
asking only whether the shared tree held a registered record that was not committed — a
registration under `records/`, or a modified document some registration names. Unregistered
drafts were excluded, because `build.rs` embeds the named documents and enumerates nothing else,
so a draft nobody has registered does not reach the store.

**It never reached zero.** Seven commits landed on `dev` during that hour from three other
sessions, and for fifty-two of the sixty samples the count sat at exactly four: one session's
record pair plus two record bodies it had modified, held across the whole hour under a lease it
kept renewing. The tree was *busier* at the end than at the start.

That is the figure decision 4 rests on, and it is worse than "occasionally red". A finish-time
freshness check over the working tree would have refused **every** finisher, on this day, for an
hour without interruption, over a diagram that was correct the whole time.

## The Decision

### 1. `diagrams/relations.mmd` is owned by nobody, and that is a rule rather than an oversight

**No item reserves it. Re-rendering it is not a territory violation.**

`OD-LEDGER-001` records that territory is declared rather than enforced, and its purpose is to keep
two authors from writing incompatible content into one file. A derived projection has no such
failure mode. Two record writers do not *disagree* about `relations.mmd`; each would render the
same function of the same store, and the later render over the fuller record set subsumes the
earlier one. There is no authored content to arbitrate, so there is nothing for territory to buy.

What reserving it *would* buy is a third structural serializer. `KNOWN_SERIALIZERS` in
`crates/substrate/nomos-ledger/tests/records_do_not_serialize.rs` is **empty** as of
`OD-LEDGER-011`, which spent an item each on the two entries that were there, and
`Test_Every_Universal_Reservation_Should_Be_Declared` exists to fail the moment a third arrives.
Requiring every record writer to reserve the diagram would refill that register on the item after
it emptied, and would serialize the board on a file nobody can conflict over.

`P10-DIAGRAM-OWED` itself reserves the diagram, and that is consistent rather than an exception:
this item's subject *is* the file, so it is authored content here and derived output everywhere
else.

### 2. Territory was never the missing thing, because it does not grant a renderable tree

This is the part that makes the first branch not merely costly but ineffective, and it is why the
measurement above leads.

A claim on `diagrams/relations.mmd` grants the right to write the file. It does not grant a tree
in which the right bytes can be computed. Measured: an agent holding that claim in this working
tree, on this day, would have rendered a diagram containing seven references to `OD-LEDGER-012` —
a record CI rebuilds without — and the gate would have failed on the commit carrying it. The
holder would have done everything territory asked and produced a worse artifact than the stale one
they replaced.

The hazard is not concurrent *writes* to the diagram. It is the concurrent *presence of unlanded
inputs*, and territory has no vocabulary for that. `AGENTS.md` already said "render only from a
tree whose records are all committed"; what it did not say is that the shared working tree is
essentially never such a tree.

### 3. The obligation is not "remember to render"; it is "render from the record set you publish"

The rule, stated so the next author is not judged by precedent:

> A projection is rendered from a tree whose registered records are exactly the records the
> rendering commit will publish — no more and no fewer.

"All records committed" is the wrong test in both directions. It is too strong, because an author
adding a record must render *with* their own record on disk, or the diagram is stale the instant
they commit. It is too weak, because it is satisfied by a tree that is missing nothing and still
carries a peer's unlanded file.

The tree that satisfies it is constructed rather than waited for: a detached checkout of `HEAD`,
with the author's own record files copied in and nothing else. That is what this item did, and the
procedure is written into `.claude/skills/nomos-spec-change/SKILL.md` where the guidance to reserve
both record paths already lives. It was the missing step there — that skill told authors to prove
no projection went stale and did not say which tree could answer.

### 4. The freshness step is **not** derived into `work finish`, and deriving it would be a defect

This is the direct answer to the question of why a repository that derives its lint step from
`gate.yml` leaves this one alone. It is not content to leave it to memory, and it is not an
oversight that nobody wired the second step.

`work finish` runs in the shared working tree. The third row of the table above is what the derived
argv reports there: **exit 8, over a correct file, for another session's record.** Wiring it would
mean that any session holding an unlanded record refuses every other session's `finish`, on a check
that is red by construction rather than by anything the finisher did. That is strictly worse than
the red it replaces: a post-merge failure on `dev` that one item can fix becomes a board-wide brick
that no item can, and `OD-LEDGER-007`'s subject is exactly the cost of that shape.

The asymmetry with the lint step is not a matter of taste. `cargo clippy` over the working tree
answers a question about the working tree, and a peer's red there is a real red that the same
command in CI will also produce — recorded as a wait-and-retry hazard, not a false one. `spec
freshness` over the working tree answers a question about a record set that will never be
published in that combination. One derivation reads a tree it is about; the other reads a tree it
is not.

So the check stays where it can be answered: `.github/workflows/gate.yml`, which runs over
`actions/checkout`, which is by construction the tree this record requires. **The gate step *is*
the derivation.** What was missing was never a second reminder — it was permission to act on the
verdict, settled by decision 1, and a local procedure that reproduces CI's tree, settled by
decision 3.

### 5. An item whose predicate is a projection check is verified in the tree it publishes

Found by this item failing to finish itself, which is the sharpest available demonstration that
decision 4 is right.

`P10-DIAGRAM-OWED`'s authored `verification.argv` is the freshness command, and its `done_when`
qualifies it — "exits 0 against a tree whose registered records are all committed". The argv
carries no such qualifier, and `work finish` runs it in the shared tree. So the item that exists
to fix this hazard was itself refused by it: over three and a half hours the predicate never
once could have passed, over a diagram that was correct throughout.

Waiting was not the remedy and would not have become one. The session holding the four blocking
record changes lapsed 108 minutes before this was written and never returned, so its unlanded
files are stranded in the shared tree until somebody takes that item over. **A dead session's
debris blocks a working-tree freshness check indefinitely**, which is one more reason not to put
one in `finish`.

The predicate was therefore run — lint step and all — in the detached worktree holding exactly
this commit's content, with `NOMOS_WORK_DIR` pointing at the real board so the transition was
recorded where it belongs. That is not a bypass and the distinction matters: the shared tree
would have answered a question about a record set that will never be published, while the
worktree is byte-for-byte what CI checks out. The check was made *more* meaningful, not less.

Stated as a rule, because the next projection-checking item will hit it: **run a predicate in the
tree whose contents the commit will have.** For an ordinary `cargo test` predicate that is the
working tree and nothing changes. For a predicate that reads the record set, it is the constructed
worktree, and the two are not the same tree.

## What This Does Not Do

- It does not make the diagram render automatically. An author who adds a record and does not
  render still leaves the gate red on the next commit; they are now merely allowed to fix it, and
  told how.
- It does not add a check at finish time. If one is ever wanted it must run against an isolated
  checkout rather than the working tree, and that is a separate item with its own cost — a full
  build in a private target directory per finish. It is not widened into this one.
- It does not change what `--require` requires. `OD-PROJECT-001` and the `Required projections`
  step still decide that `diagram-set` is the one profile this repository ships and can rebuild.
- It does not give territory a way to express "the tree must not contain unlanded inputs". That
  would be a property of a working tree rather than of a claim, and `OD-LEDGER-001`'s model has no
  place to put it. The procedure is the answer instead.
- It does not exempt the author from `AGENTS.md`'s ordering. Render last, after your own records
  are written, and never beside somebody else's.

## Controls

| Weakening | What it produces |
|---|---|
| render from the shared working tree | a diagram carrying `OD-LEDGER-012`, which CI rebuilds without; the gate fails on the commit carrying it |
| render from a checkout of `HEAD` without the author's own record | the diagram is stale the instant the record commits, which is `401b2d0` repeated exactly |
| reserve the diagram in every record writer's territory | `Test_Every_Universal_Reservation_Should_Be_Declared`: "a third thing every record writer has to touch is a third reason the board runs one item at a time" |
| derive `Required projections` into `work finish` | exit 8 for every finisher while any session holds an unlanded record — measured above over a correct file |
| hand-edit `relations.mmd` to add the missing edges | the sidecar's `content_digest` no longer matches the body, and freshness reports `edited` rather than `stale` |
| delete the sidecar to silence the comparison | a body with no sidecar is a failure rather than a skip, per the projection contract |

## Status

Closed by `P10-DIAGRAM-OWED`, which re-rendered the diagram from a detached worktree at `56d8205`
plus this record, and is the first render taken under the rule in decision 3.

One consequence is worth leaving for whoever reads this next. `P10-DIAGRAM-OWED`'s own
`verification.argv` is still the unqualified freshness command, so the next holder of any item
with a projection predicate will meet decision 5 again. Re-authoring those predicates — or giving
`finish` a way to be pointed at the tree a predicate is about — is real work this record does not
do, and it is named here rather than widened into an item whose territory is a diagram and a
skill.
