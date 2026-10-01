---
id: OD-LEDGER-012
type: decision
title: A lapse is taken over by a verb of its own, and the claim it replaces is kept
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-006
    type: affects
  - target: OD-LEDGER-009
    type: affects
  - target: OD-LEDGER-008
    type: relates-to
  - target: OD-LEDGER-005
    type: relates-to
  - target: OD-LEDGER-016
    type: relates-to
---

# A lapse is taken over by a verb of its own, and the claim it replaces is kept

## Question

`MAXIMUM_LEASE` exists so that an agent which died holding a claim stops holding territory
without anybody editing `work/ledger.json` by hand. For every item except the one it died on,
that is what happens. For that one item it is not.

`Claim_Refusal` rejected any state that is not `Ready` before it consulted the lease. `Renew`
and `Release` match on the holder, and the holder of a lapsed claim is by definition a process
that died — that is the only way a lease lapses on its own. The item was unreachable by
anybody until a person edited the file, which is the outcome the lease was introduced to make
impossible.

`OD-LEDGER-009` settled that deliberately rather than leaving it as a consequence, and named
what stood in the way: `Claim` overwrites `claim`, `claim` is the only thing on a lapsed item
recording that the work was ever started, and `OD-LEDGER-006` decided that must survive. A
takeover had nowhere to put the claim it replaces.

So this record is not about whether a lapsed item should be recoverable. It is about where the
claim goes.

## The Criterion

`OD-LEDGER-006` refused to synthesize an `Abandonment` for a lapse because
`Abandonment::reason` is *the words the holder gave* and a lapse has none. The rule underneath
that refusal is the one applied here:

**What survives a transition is what somebody actually knew at the time.**

A lapse knows who held the item, when they took it, and when the lease ran out. It does not
know why they stopped. A record of a lapse therefore carries the first three and has no field
for the fourth — not an empty one, not a filled-in one. There is nothing to invent because
there is nowhere to put an invention.

That criterion decides the three shapes this item was opened with. A synthesized
`Abandonment` is refused, unchanged: `work show` prints an abandonment as `abandoned by X at
unix N: <reason>`, and a manufactured "the lease expired" would read on that surface as a
sentence the dead agent typed. Filler wearing a record's clothes, which this repository
refuses elsewhere for placeholder records.

## The Decision, In Two Parts

### The claim that lapsed is kept, as itself

`LedgerItem` gains `displaced: Vec<Claim>` — every claim on this item that lapsed and was
displaced by a takeover, oldest first.

**The claim itself, not a summary of it.** A `Takeover { previous, taken_by, taken_at }`
struct was the obvious alternative and carries one fact that is real and two that are not:
who took it over and when are the `holder` and `acquired_at` of the claim that *replaced* it,
which is already on the item. A summary is also a second shape that can drift from `Claim`,
and a claim cannot drift from itself. What that costs is stated rather than hidden: "when was
this displaced" is read off the neighbouring record — the live `claim`, or the next entry in
`displaced` — rather than said in place. A later reader who finds that indirection expensive
can reverse it deliberately, which is why the cost is written down here.

**A list, for the reason `abandoned` is a list.** An item taken over twice was taken over
twice, and keeping only the most recent discards the earlier holder — this same defect one
scale down.

**The move and the install are one operation.** `LedgerItem::Replace_Lapsed_Claim` pushes the
previous claim onto `displaced` and installs the replacement, and there is no ordering of its
statements in which the second happens and the first does not. This is
`ReleaseOutcome::Record_On`'s technique and it exists for the same measured reason: an
implementation that spells a rule out at its call site is free to spell one half of it. It
returns `false` and changes nothing when there is no claim to displace, or when the claim has
not lapsed, so an item whose predecessor record is already missing is refused rather than
given a claim written over a hole — and a holder who is merely slow is never displaced by the
method even if a caller asks.

**It is emitted whether or not it holds anything.** `displaced` carries no
`skip_serializing_if`, for symmetry with `abandoned` and for a second reason worth writing
down: because `abandoned` always serializes, `grep -c '"abandoned"' work/ledger.json`
equalling the item count is this repository's standing check that no stale writer has been
through the file. A field whose presence depends on its content cannot be counted that way.
The shape of the document is kept independent of what is in it.

### Taking over is a verb, and `claim` never becomes one

`nomos work takeover --item <id> --holder <name> [--lease 2h]`, backed by
`FileLedger::Take_Over`. A plain `claim` on a lapsed item still refuses, and that refusal is
what `OD-LEDGER-009` bought and this record keeps.

The two are separate answers because a `claim` that silently began displacing lapsed holders
would reintroduce the exact loss `OD-LEDGER-009` guarded against. The record would exist, and
nothing would make the agent creating it notice that it had. Taking over another agent's
abandoned work is a decision, and a decision belongs in a verb somebody typed.

**A takeover re-establishes independence by the same code a claim does.** A lapsed claim stops
excluding, so another item may since have been claimed over exactly the ground this one
reserves. The dependency and territory checks are lifted out of `Claim_Refusal` into
`Contested_By`, which both call, rather than copied — `Refusal_From`'s reason, again.

**`Take_Over` is not on `ExclusionLedger`.** A lapse is a property of a ledger that outlives
its writers. The run-scoped reservations a correction scheduler holds and the session-scoped
leases delegated agents hold both die with the process that made them, so neither has a case
where the holder is gone and the record is not. Putting this on the shared trait would oblige
two instances to implement an operation about a failure they cannot have.

**It decides under the lock.** A takeover is a read, a decision and a write, so it goes
through `Decide_Under_Lock` with the other three verbs that change the board. `OD-LEDGER-015`
is the record of what deciding outside it cost them for eleven weeks, and a fourth verb
arriving after that record and repeating the shape would be the same defect with a witness.

## The Refusal Now Names Its Remedy

`ClaimRefusal::Lapsed` carries the item, the holder whose lease ran out, and when it ran out,
and `Describe` names `nomos work takeover`. It replaces `NotClaimable` for this one case,
which said only that the item was `Claimed` — true, and indistinguishable from `Done`. The two
have opposite remedies, so one word for both left the operator with no next step.

`OD-LEDGER-005` is why this is worth a variant rather than a better string. `ready` was the
first word on this board to mean something other than what a reader took it to mean and
`claimed` was the second; the remedy both times was to compute the label from the same function
`claim` refuses with, so a listing cannot hold an opinion of its own. `work list` still says
`lapsed`, and it now says it *because* `Claim_Refusal` said so. It previously decided
lapsedness itself, in a branch above the refusal it consulted — a second implementation of the
predicate that, after this record, decides whether `takeover` will succeed. That branch is
gone. Every label is unchanged for every input; what changed is that one function decides.

It is not retryable. Waiting does not turn a dead holder into a live one; somebody has to
decide to take the work. A takeover refused because the lease is *still live* is retryable, and
is the one refusal here that means wait.

## What Is Renamed Rather Than Reversed

No test's assertion is reversed, and saying otherwise would be the same overstatement this
item is about.

`Test_A_Lapsed_Item_Should_Refuse_A_New_Holder_And_Keep_The_Old_One_Visible` becomes
`Test_A_Lapsed_Item_Should_Refuse_A_Plain_Claim_And_Name_The_Takeover`, because "refuse a new
holder" is false once a takeover installs one, and a test name that lies is worse than one that
is long. Its subject — `claim` on a lapsed item is refused — is not reversed; `claim` still
refuses, and this record keeps that refusal deliberately. Its one moved assertion goes from
`ClaimRefusal::NotClaimable` to `ClaimRefusal::Lapsed`, and its own message gets *more* true
rather than less: the refusal now names the holder and the remedy. Its assertion that the
lapsed claim was not replaced is kept word for word.

That test's previous doc comment is what settles it: it "stops short of the claimability,
because an assertion either way would pin the behaviour before that decision is made". The
decision is made here. `Test_A_Lapsed_Item_Should_Be_Taken_Over_And_Still_Name_Its_Previous_Holder`
covers the case it could not, because the operation did not exist.
`Test_The_Holder_Should_Still_Recover_Its_Own_Lapsed_Claim` is untouched and is now also the
control on `Take_Over` not having quietly loosened `Renew`'s holder match.

## What This Reopens

`OD-LEDGER-006`, section "What This Found And Did Not Fix", line 106: "A lapse does not make
the item takeable again." Amended at version 2. That section left the defect on the ledger
deliberately and said an assertion either way would pin the behaviour before the decision was
made. The decision is made here, and it is neither of the two shapes that paragraph
anticipated: a lapse does not return the item to `Ready`, and what it costs an agent that is
merely slow is bounded by that agent's own lease rather than by a policy.

`OD-LEDGER-009`, section "A lapsed item stays `Claimed` and nobody else may take it". Amended
at version 2. The first half stands — a lapsed item stays `Claimed`, and a takeover does not
change its state — and the second half is replaced by this record. That record's *title* is
untouched and remains literally true: a lapsed item is still not **claimable**, and `claim`
still refuses it. What exists now is a different verb.

Neither amendment touches the decisions those records are *about*. `OD-LEDGER-006` remains
that a reason attached to a state survives and one attached to a transition does not; this
record is that rule producing an answer for a transition nobody was present for.

## The Coupling Territory Could Not See

Landing this required two files that the item's own territory did not name, and the reason is
worth recording because it is the second instance of one shape in a single day.

`LedgerItem` has no `Default`, so adding a field breaks every exhaustive struct literal in the
workspace — one of which was in `tests/gate_covers_finish.rs`. And raising `SCHEMA_VERSION`
falsified `said.contains("this build understands 1")` in
`tests/stale_writer_is_refused.rs`, a string literal spelling the constant's *value* in a file
whose territory is disjoint from the constant's. Both files were authored one commit earlier by
the sibling item this one was sequenced behind. Neither was held by anybody. Both were invisible
to the board.

They were invisible for a structural reason rather than by oversight. Territory is paths, and
the coupling here is not a path: it is a type's field list against every literal that
constructs it, and a constant's value against every string that spells it. `Disjoint` is the
right answer to the question territory asks and the wrong answer to the question that mattered.
`OD-LEDGER-016` closed the same shape for record identifiers — a stem and the file it names
compared `Disjoint` while being one subject — and this is that shape again with a type and a
constant in place of a record. The general form: **a reservation over paths cannot see a
coupling whose two ends are not paths**, and narrowing an item to files is exactly the
operation that exposes one.

Two remedies were available and only one of them generalizes. Adding the paths is what this
item did, and it fixes this instance. Making the test read `SCHEMA_VERSION` instead of spelling
`1` is what stops the next instance, and it is done: the assertion is now
`&format!("this build understands {SCHEMA_VERSION}")`, so the next bump cannot reach it. The
adjacent literal `schema 1` in the same test is left alone deliberately and commented, because
it reads the fixture's own version, which no bump changes. There is no equivalent trick for the
struct literal; a field added to a type without `Default` will always cost a line at every
construction site, and the remedy there is to expect it rather than to discover it.

## Consequences

An item held by a process that died returns to the pool without a person editing a file, at
the cost of one deliberate command. That was `MAXIMUM_LEASE`'s stated purpose and it is now
true for the item the lease was held on.

An agent that is merely slow, rather than dead, can be displaced. The lease is the whole of the
protection, and renewing it is the whole of the remedy — which is what `Renew` is for and why
it is deliberately cheaper than re-claiming. What a displaced agent loses is the claim; what it
does not lose is the record that it held one.

`displaced` is emitted on every item, empty, exactly as `abandoned` is. Items written before
this field existed have none, and `serde(default)` leaves it that way rather than backfilling a
history nobody observed.

The schema moves, `1` to `2`, which is why this lands after `OD-LEDGER-008` rather than before
it. A build that predates this field now refuses the ledger and says so, instead of dropping
`displaced` from every item it touches — which is precisely the loss this field exists to
prevent, applied to the field itself. Sessions run a copied `nomos.exe`, so the first thing
this record costs is a rebuild and a re-copy; that refusal is the new correct behaviour and not
a fault.

No `Validate` rule is added about `displaced`. A rule asserting that a displaced claim precedes
the claim that replaced it would be a guard with no demonstrated failure behind it, which is an
abstraction with no consumer. If a real inversion is ever observed, that observation opens the
item.

## Status

Accepted. Implemented in `nomos-ledger`, exposed as `nomos work takeover`, and reported by
`nomos work show` and `nomos work list`.
