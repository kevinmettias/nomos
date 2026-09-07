---
id: OD-LEDGER-038
type: decision
title: An item can be ended honestly and leave other items silently unreachable
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - dependencies
relations:
  - target: OD-LEDGER-019
    type: relates-to
  - target: OD-LEDGER-020
    type: relates-to
  - target: OD-LEDGER-023
    type: relates-to
  - target: OD-EXECUTOR-007
    type: relates-to
  - target: OD-EXECUTOR-008
    type: relates-to
  - target: OD-GATE-022
    type: relates-to
---

# An item can be ended honestly and leave other items silently unreachable

## Question

`OD-LEDGER-020` gave a declined dependency its own refusal arm, `DependencyDeclined`, so a
dependent pointed at a dead end reads `stranded` instead of the misleading `waiting`. That
closed the case where the ledger itself can see the problem: the dependency's `state` is
`Declined`, the comparison is mechanical, and the label is correct the instant it is asked
for.

Two related failures sit outside what that arm can see, because in both of them the
dependency's state is not `Declined` — it is `Done`, exactly what `depends_on` was built to
wait for.

**A decision structurally cannot satisfy a dependent that needs code.** `ItemKind::Decision`
"settles an architectural question and records the answer... whether or not it also touches
code" — kind alone does not say which. When a decision's own `done_when` asks only that a
question be answered, finishing it makes every dependent read `Ready`: correctly, by the
letter of `depends_on`, and uselessly, because the code the dependent actually needs was
never written and nothing said so.

**A decline's cascade is invisible at the moment it happens.** `nomos work decline` does not
inspect `depends_on` at all — `Decline_Item` writes the transition and returns
(`crates/substrate/nomos-ledger/src/store/verbs.rs:75-102`). Every direct and indirect
dependent still shows up correctly as `stranded` the next time anyone runs `list`, because
`OD-LEDGER-020`'s arm fires — but nobody is told so at decline time, and a session that does
not think to re-run `list` over the whole board does not learn what it just orphaned.

A third mechanic compounds both: a re-authored successor's id is not reliably discoverable
from the id it replaced.

## What was measured

**Shape one, live on the board right now.** `P42-EXECUTOR-WORKRESULT-IMPLEMENTATION` was
declined for under-reserved territory and re-authored three times — `-2`, `-3`, `-4` — each
decline naming its successor, but only at the *end* of the reason:

> Re-authored as P42-EXECUTOR-WORKRESULT-IMPLEMENTATION-2 with all five added, unchanged in
> kind, done_when and predicate.

`ItemState::Describe` — what a listing and a refusal both render — cuts a `Declined` reason
to its first line. None of the three reasons name the successor there; all three name it in
the sentence a listing never shows. `P42-EXECUTOR-WORKRESULT-IMPLEMENTATION-4` finished
(commit `fe0fac58`, this repository's own history), carrying the real implementation. Three
items authored against the *original*, declined id are still on the board today, reported
`stranded` by `nomos work list`, and none of them has been repointed to `-4`:
`P42-EXECUTOR-ENVELOPE-IMPLEMENTATION`, `P42-SECOND-HARNESS-EXECUTOR-2`, and
`P40-AGENT-WORKRESULT-VALIDATED-PATH-3`. The mechanism `OD-LEDGER-020` built is doing exactly
what it was built to do — the label is correct — and the work it names is still undone,
because being told `stranded` and being told what to point at instead are different sentences,
and only the first one is currently visible in a listing.

**Shape two, the decision-before-code instances.** `P42-EXECUTOR-ENVELOPE-MECHANISM` and
`P42-EXECUTOR-WORKRESULT-MECHANISM` are `Done`, recorded as `OD-EXECUTOR-007` and
`OD-EXECUTOR-008`. Both records are explicit that they decide a shape and do not carry it:
`OD-EXECUTOR-008` grounds a `WorkResult` shape; the code that populates one did not exist
until the `P42-EXECUTOR-WORKRESULT-IMPLEMENTATION` chain above was filed and finished
separately, in a later session, as its own four-item chain. `OD-GATE-022` is the same shape a
third time and is not yet closed: it decides that `Compare_Gate_Runs` takes two already-
produced `GateRunResult`s and says outright, in its own "what this record does not do"
section, that it does not wire `compare` into any surface. `Compare_Gate_Runs` has zero
callers today; `P73-GATE-COMPARE-HAS-NO-CALLER` is filed and `Ready`, unclaimed.

**Shape three, a decline's cascade.** Declining `P41-RUN-PLANNER` stranded three dependents
directly and left four more waiting behind those, in one write, with nothing said at decline
time about the seven items just made unreachable — the finding that produced the first,
narrower version of this record (`P73-DECISION-BEFORE-CODE-STRANDS-ITS-DEPENDENTS`) and the
reason it was declined and re-authored to cover both shapes rather than one.

## The decision

### 1. Ending an item warns about its dependents; it does not refuse

Neither `nomos work finish` nor `nomos work decline` refuses because live dependents exist.
Both are legitimate ways to end an item even when dependents are affected — the `P41-RUN-
PLANNER` decline was correct on its own merits, and a decision that answers a question
honestly without also shipping code is not thereby wrong. The ledger cannot judge whether
leaving a dependent's precondition mechanically-satisfied-but-substantively-undone is
acceptable; only the person ending the item can, and a refusal would block work that is
often right.

What is missing is visibility at the moment it matters, not a gate. `nomos work finish` and
`nomos work decline` are to print, on success, every item on the board whose `depends_on`
names the one just ended, using the same reachability computation `list` and `audit` already
use for `Blocking_Refusal` — so the fanout is read once, at the point of the decision,
instead of requiring a second, separate sweep of the whole board to discover later. This is
new plumbing across `nomos-work-orchestration`'s `WorkOutcome` and `nomos-cli`'s report
layer and is not built by this record: it is filed as `P85-LEDGER-038-FANOUT-WARNING-2`,
`Capability`, `Required`, so the decision does not repeat the exact defect it names — a
remedy asserted in prose with no item behind it.

### 2. A decision item may be depended on; `depends_on` binds to `done_when`, not to `kind`

`ItemKind::Decision` is explicitly allowed to touch code, so refusing every `depends_on`
edge onto a `Decision` item on the basis of its kind alone would be both over- and
under-broad: some decisions do carry an implementation in the same item, and some
`Capability` items could in principle promise nothing but a written answer. Kind is not the
signal.

The binding rule instead: an item declares `--depends-on X` only when `X`'s own `done_when`,
read as written, actually delivers what the dependent needs in order to begin — not because
`X` is topically related. When a decision's `done_when` asks only that a question be
answered and recorded (as `OD-EXECUTOR-007`'s, `OD-EXECUTOR-008`'s and `OD-GATE-022`'s each
do, each saying so in its own "what this record does not do"), nothing may depend on it for
code, and the implementation is a separate item filed for that purpose.

### 3. A stated remedy is not honest until it is a filed item

An item ends — by `decline`, naming a follow-up as its remedy, or by `finish`, as a
`Decision` whose `done_when` implies further work — honestly only when that follow-up exists
as a real item id on the board at the moment the ending is recorded, not as prose promising
one later. This record's own close is held to the same rule: `P85-LEDGER-038-FANOUT-WARNING-2`
is filed alongside it, and the concrete instance measured above is repointed, not merely
diagnosed, in the same commit that closes this item — matching the discipline
`OD-LEDGER-020` decision 5 already set for exactly this act.

### 4. A re-authored successor is named where truncation cannot hide it

A `decline` reason that names a successor must name it in its first sentence. `ItemState::
Describe` cuts a `Declined` reason to its first line for every caller that renders it in a
listing or a refusal — `OD-LEDGER-014`'s reason for existing — so a successor named only at
the end of a multi-sentence reason is, for every practical reading path on this board, not
named at all. This is a convention for `nomos work decline --reason`, not new ledger
mechanism: enforcing prose shape mechanically is rejected below for the same reason
`OD-LEDGER-020` rejected a `work re-point` verb — it is a real improvement on its own merits
and is not what this record's measured defect needs to close.

`P42-EXECUTOR-ENVELOPE-IMPLEMENTATION`, `P42-SECOND-HARNESS-EXECUTOR-2` and
`P40-AGENT-WORKRESULT-VALIDATED-PATH-3` are declined and re-authored, each depending on
`P42-EXECUTOR-WORKRESULT-IMPLEMENTATION-4` in place of the declined
`P42-EXECUTOR-WORKRESULT-IMPLEMENTATION`, territory and `done_when` otherwise unchanged: the
work each describes was correctly scoped throughout, and only the edge was wrong.

## What this record does not do

**It does not build the fanout warning.** Decision 1 is filed as
`P85-LEDGER-038-FANOUT-WARNING-2`, real implementation territory in
`nomos-work-orchestration` and `nomos-cli`, left for that item to claim and finish.

**It does not add schema linking a declined item to its successor.** A structured
`superseded_by` field on `Declination` was considered — see below — and rejected for now in
favor of the first-sentence convention, which needs no schema change and closes the measured
instance immediately.

**It does not repoint every stranded item on the board**, only the three the measured
instance names. Any other dependent pointed at a declined id and not yet repointed is a
separate finding for whoever next runs `list --state stranded` over the whole board.

## What was considered and rejected

**Refusing `finish` or `decline` when dependents exist.** Rejected: both worked examples in
"what was measured" are cases where ending the item was the right call despite the fanout,
and the ledger has no way to judge acceptability — only visibility is missing, not a gate.

**Refusing `--depends-on` naming any `Decision`-kind item.** Rejected: `ItemKind::Decision`'s
own documentation allows a decision to carry code, so kind is not a reliable discriminator,
and a blanket refusal would block the legitimate cases along with the defective ones.

**A structured `superseded_by: Option<ItemId>` field on `Declination`, walked automatically
by `Claim_Refusal` to name the live tip of a decline chain.** A real mechanism, and a
stronger fix than a prose convention — but new schema, new serialization, and a new refusal
rendering path, none of which the measured instance needs: naming the successor in the first
sentence closes it today. Worth its own item if the first-sentence convention proves
insufficient across a longer chain than the three-hop one measured here; not decided by this
record.

**A `work re-point` verb that edits `depends_on` in place.** Out of scope for the same reason
`OD-LEDGER-020` gave it: real on its own merits, not what either measured shape needs, since
the remedy — decline the dependent, re-author it — already exists and was already the
convention this record's own decision 4 uses to close its instance.

## Controls

| Weakening | What it produces |
|---|---|
| refuse instead of warn | legitimate finishes and declines blocked on a fact the ledger cannot judge |
| refuse `--depends-on` on `Decision` kind alone | blocks decisions that do carry code, and does not catch a `Capability` item that under-delivers the same way |
| leave decision 1 as prose with no filed item | this record repeats the exact defect it names |
| leave a successor named only at the end of a decline reason | `ItemState::Describe`'s first line never shows it, on this board or any future one |
| diagnose the measured instance without repointing it | three items stay `stranded` on a board this record was written to make legible |

## Status

Open. Closed for decision 1 when `P85-LEDGER-038-FANOUT-WARNING-2` finishes; decisions 2
through 4 are closed by this record and its own repointing commit.
