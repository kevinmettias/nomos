---
id: OD-LEDGER-010
type: decision
title: Opening an item on held ground is how the board is used, so add guarantees a name and a valid document and nothing else
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-LEDGER-005
    type: relates-to
  - target: OD-LEDGER-007
    type: relates-to
---

# Opening an item on held ground is how the board is used, so add guarantees a name and a valid document and nothing else

## Question

The doc comment on `Add` in `crates/host/nomos-cli/src/work.rs` read:

```
/// Records a new item, refusing one whose territory is already spoken for.
```

It never did that. `nomos_ledger::Validate` reaches territory through
`Overlapping_Claims`, which begins by filtering the document to items satisfying
`Has_Active_Claim`, and an item arriving at `Add` carries `claim: None` by construction —
`Parse_Add` builds it that way and there is no flag to say otherwise. The comparison
therefore has nothing to say about a new item and has never fired for one. `Add` rejects a
duplicate identifier, appends, and calls `Save`.

The sentence and the function arrived in the same commit, `914a091`, so the claim has been
false for as long as the command has existed.

It is demonstrated on this board rather than argued from the code. `60d0c5b` and `7efc2a9`
each added an item reserving `crates/substrate/nomos-ledger`,
`crates/spec/nomos-spec-store` and `tests/contract` while another session held all three
under a live claim on `P10-SEEDING-SERIALIZES`. Both were accepted. Both are commits
touching `work/ledger.json` and nothing else, which is what an opened item looks like.

## The Decision

**The sentence was wrong, not the behaviour. `add` refuses a duplicate identifier and a
document that would not validate, and refuses nothing on account of territory.**

The question is not rhetorical, and answering it "obviously the comment" without stating
why would leave the next reader where this one started. There is a real argument for the
other side: two items on one territory is the condition the ledger exists to detect, and
detecting it earlier is usually better than detecting it later.

It falls the other way for three reasons, in ascending order of weight.

**Opening blocked work is the ordinary case, not the exceptional one.** Both commits above
opened items *because* their territory was held — that is when a defect in the code you are
looking at is noticed. A refusal would forbid the normal way an agent records what it found
while it could not act on it, and the alternative it would push people toward is not
"wait", it is "do not write it down".

**`add` is the door that stays open when the board is closed.** `work/ledger.json` is in
nobody's territory, which is the arrangement that makes `add` available while every item on
the board is held. `OD-LEDGER-007` measured that condition four times in one afternoon —
one claim refusing every other item — and `OD-LEDGER-011` measured 42 of 78 pairs still
blocked by ordinary contention after the serializers were removed. A territory refusal in
`add` would shut the last open door at exactly the moment it is the only one.

**Exclusion belongs where the editing is.** `claim` is the verb an agent runs immediately
before it starts changing files, and it is the verb whose refusal is retryable and carries
the holder's name. `add` writes a sentence about work that may not start for days, against
a board that will have moved by then. A refusal computed at authoring time would be a
statement about a world that no longer exists when the work begins — and it would say
nothing at all about the case it appears to protect against, since two items opened hours
apart on free ground can still be claimed together tomorrow only because `claim` checks
again.

So what `Add` guarantees is smaller than the sentence promised, and the corrected comment
states it exactly: the identifier is unused, and the document that results still satisfies
its own invariants. Validation happens before the write, so an item that would break one
never lands.

## The Class This Belongs To

A doc comment asserting a control that does not run is the defect this ledger keeps
recording under other names. `OD-COMPLETENESS-001` is a coverage claim written where nothing
resolves it; `OD-LEDGER-015` is `With_Lock`'s doc comment describing a lock three callers
bypassed. The cost is not the wrong prose. It is that the next person to reason about
whether two items may safely be opened against one territory reads the sentence, is told a
check protects them, and stops looking.

That is why this is a record and not an edit. The behaviour is unchanged and the diff is a
comment and a test file; what is written down is the reason the guarantee is where it is,
so that "should `add` refuse held territory?" is a question already answered rather than
one re-derived by whoever next notices the gap.

## What Holds It

`crates/host/nomos-cli/tests/add_guarantees_what_it_says.rs`, one assertion per clause of
the corrected sentence, driven through the binary for the reason `list_tells_the_truth.rs`
gives — the defect was in what the surface said about the logic, and a caller who is misled
reaches this program through its command line.

The accepted case and its deferral are asserted together, because either alone is
misleading:

```
$ nomos work claim --item T-1 --holder agent-a
T-1 held by agent-a until unix 1786346963                          (exit 0)
$ nomos work add --item T-2 --territory src/shared.rs
added T-2 reserving 1 path(s)                                      (exit 0)
$ nomos work claim --item T-2 --holder agent-b
refused: T-1 overlaps territory held by agent-a until unix 1786346963
                                                                   (exit 3)
```

`Test_The_Item_Added_On_Held_Ground_Should_Still_Be_Refused_A_Claim` is the second of those
three lines through to the third. Without it,
`Test_Adding_An_Item_On_Held_Ground_Should_Be_Accepted` reads as a ledger that waves two
agents onto one file — which is the reading the old sentence was written to prevent, and
the reason deleting it alone would not have been enough.

The two clauses that are promised are asserted on their refusals and on the bytes:

```
$ nomos work add --item T-1 …
T-1 is already on the ledger                                       (exit 4)
$ nomos work add --item T-3 --territory src/c.rs --depends-on T-9
ledger is invalid:
  T-3 depends on T-9, which is not in the ledger                   (exit 1)
$ nomos work add --item T-4 --territory src/d.rs --territory SRC/D.rs
ledger is invalid:
  T-4's territory lists `src/d.rs` and `SRC/D.rs`, which name the same subject; whoever
  wrote it probably believed they were reserving two things        (exit 1)
```

Each refusal is checked against the ledger file read back byte for byte, not against its
item list. A rewrite that happens to preserve the items is still the write `Save` documents
itself as not having performed, and the "validated before the write" clause is about the
write and not about the outcome. Exit 4 rather than 3 for the duplicate is asserted too:
a taken name is not a race somebody wins by waiting, and the exit code is what agents
branch on — `OD-LEDGER-009` records the same distinction being got wrong in `claim`.

Two controls stop the set going vacuous. `Test_An_Item_Added_On_Free_Ground_Should_Claim`
moves the territory apart and claims the added item, so the first pair cannot be satisfied
by a ledger that refuses every claim. `Test_A_Well_Formed_Item_Should_Land` adds a valid
item to the same board the two refusals use, so those cannot be satisfied by an `add` that
has stopped accepting anything.

## What This Does Not Hold

**Nothing here goes red if the comment drifts back.** The tests state that the behaviour is
what the corrected sentence describes; they cannot state that the sentence still describes
it. A doc comment is prose, and the only mechanism in this workspace that judges prose
against behaviour is `nomos_rules::mirror`, which resolves the mirror a *declared universe*
names and does not look at free-standing functions.

This is the residue of the class, and it is stated rather than closed. Closing it means
deciding what it would mean for an arbitrary function's doc comment to be checkable, which
is a larger question than the one this item settles and is not obviously answerable at all
— most doc comments describe things no predicate can express. What is bought instead is
cheaper and real: the corrected sentence is true, it is true in a way somebody can run, and
the assertions sit in the same crate as the function, so a reader who distrusts the prose
finds the answer without leaving the directory.

## Status

Closed by `P10-ADD-PROMISE`.
