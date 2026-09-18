---
id: OD-LEDGER-008
type: decision
title: A writer that does not understand a document must not write it back
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - work-ledger
  - concurrency
  - enforcement
relations:
  - target: OD-LEDGER-006
    type: relates-to
  - target: OD-LEDGER-009
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
---

# A writer that does not understand a document must not write it back

## Question

`LedgerItem` and `LedgerDocument` derived `Deserialize` with serde's default behaviour, which is
to ignore any key the type does not declare. `SCHEMA_VERSION` existed and was read at exactly one
site — filling in the document returned for a file that is not there. `Load` never compared it.
`Save` re-serialized whatever `Load` produced.

Compose those three and a `nomos.exe` built before a field existed reads the current
`work/ledger.json`, drops every key added since, writes the result back, and exits 0. The state
change survives. The data does not. A lossy write and a clean one are the same event at the
surface that reports them.

## The Evidence

Not inferred. An abandonment reason for `P10-ABANDON-REASON` was written at unix 1786326402
through a freshly built binary and was gone by 1786326839, when another session's `work claim`
rewrote the file. The release stood; the reason did not.

It had been happening since the field was introduced. Commit `e88f92d` added
`LedgerItem::abandoned` with `serde(default)` and no `skip_serializing_if`, so a current binary
emits the key on every item — and both `e88f92d` and `6326566` committed a `work/ledger.json`
containing zero of them. The session that added the field was stripping it as it wrote it, in the
commit that added it, and nothing said so.

What produced it is a documented workaround rather than carelessness. `work finish` cannot run
under `cargo run`, because the predicate rebuilds the executable that is the running process, so
sessions copy `target/debug/nomos.exe` and run the copy. A copy taken before a schema change is a
stale writer for the rest of that session. A workaround for a file lock became a silent
data-loss channel on a file two sessions share.

`Test_The_Ledger_Should_Round_Trip_Losslessly` could not see any of it. It builds a `LedgerItem`
in Rust, saves it, loads it and compares, so every key in that file is a key the type declares.
It asserted that the *declared* shape survives, which was never in question.

## The Shape

`OD-LEDGER-006` one level down. That record is about a reason attached to a transition not
surviving it. This is a reason attached to an *item* not surviving the next *writer*, and it fails
the same way, by looking like success.

`OD-LEDGER-009` named the third instance of a reason destroyed by the failure it explains. This is
the fourth, and the most complete, because here the failure has no report at all.

## Three Remedies Were Available

**A schema version compared on load.** Cheap — two lines and an error arm — and it produces the
best sentence an operator could read. Refused as the *guarantee* for one reason: nothing forces
the bump. `e88f92d` added a field and raised nothing, so a guard whose completeness rests on the
discipline that already failed will report clean on the next instance of the defect it was built
for. Absence of a bump would become success. It is also coarse: it answers "is this file newer
than me" and never "is there a key here I am about to destroy".

**A capture map, preserving unknown keys through a `serde(flatten)` map.** It makes an old binary
*correct* rather than merely loud, which is a stronger guarantee than the one adopted here, and it
is refused anyway.

Three costs, the last decisive. It is silent: a stale writer keeps working and nothing anywhere
reports that one is present, when an undetected stale writer is what caused the incident. It
cannot coexist with `deny_unknown_fields` on the same type, so adopting it trades away detection
of a *misspelled* key permanently — measured in a standalone probe: under a capture map,
`{"clam": {…}}` parses with `claim: None` and the misspelling parked in the map, where before it
was at least discarded and under the adopted remedy it is refused outright.

And it promises more than it can keep. Preserving a key preserves bytes, not meaning. Nothing in
this crate can know whether the key it copied through changes the meaning of the keys it does
understand — a later `frozen` or `superseded_by` would be carried forward faithfully by a binary
that then acts as though it were not there. "An old binary round-trips correctly" is a guarantee
larger than what preservation demonstrates.

**Refusing to read a document that carries anything unaccounted for.** Adopted, because it is the
only one of the three that both prevents the loss **and** makes the stale writer visible at the
moment it tries.

## The Decision

**A build that cannot account for every key in the ledger does not write the ledger.**

`#[serde(deny_unknown_fields)]` on every deserialized container reachable from `LedgerDocument`:
the document, `LedgerItem`, `Territory`, `Claim`, `Abandonment`, `VerificationPredicate`,
`VerificationRecord`, `GateOutcome`, and the two enums `ItemState` and `Blocker`, whose struct
variants it also covers.

This is the crate's own rule applied to its own deserializer. `Refusal_From` exists so that an
overlap question nobody can answer refuses rather than grants, because unknown independence is not
safe parallelism. A key this build cannot account for is the same question one layer down: whether
writing this document back is lossless cannot be established, so it is not written.

**The guard is at `Load`, and only at `Load`.** That is the single door — `With_Lock`, `Claim`,
`Renew`, `Release`, `Conflicts` and `Validate_Current` all pass through it — and a document that
parsed under `deny_unknown_fields` is by construction one this build accounts for in full, so
`Save` needs no guard of its own and cannot acquire a second, disagreeing one.

**The schema version explains the refusal and never provides it.** `Load` parses strictly; when
the strict parse fails, and only then, a probe reads `schema_version` out of the text, and a file
newer than this build is reported as `LedgerError::Unrecognized` rather than
`LedgerError::Malformed`. The layering is the point: forgetting to raise `SCHEMA_VERSION` can only
degrade a message, and can never cost a field.

**`Save` stamps the version rather than echoing it.** The one state the version cannot explain is
a document carrying keys a build invented while claiming the older number it read: an older reader
then fails the strict parse, probes the version, finds nothing newer than itself, and reports the
file damaged — sending somebody to repair a file that is correct. The experiment below produced
exactly that state, so it is not hypothetical. `schema_version` is therefore the only field `Save`
does not take from its argument.

**`Unrecognized` is its own arm, not a `Malformed`.** The two remedies are opposites: a malformed
ledger is repaired, and this one is left alone while the *reader* is rebuilt. Reporting the second
as the first sends an operator to edit a file that is correct — `OD-LEDGER-009`'s "two causes
wearing one name" with the causes swapped. The CLI maps it to exit `5`: an agent told the ledger
cannot be used at all stops and fetches a person, and a binary that cannot read the board is
exactly that. No new exit code is introduced and README's table does not move.

## What The Experiment Showed

No test in this process can demonstrate that a genuinely older executable is refused: one
`cargo test`, one binary. So the end-to-end case was run by hand, with two real executables, and
this is what was observed rather than what was expected.

`nomos.exe` was built at `52dbf1a` and copied aside as `stale.exe` before any of this landed. The
change was applied and a second binary built as `fresh.exe`. A copy of the repository's own
`work/ledger.json` was given the shape a `P10-LAPSE-TAKEOVER`-era binary will write — one item
carrying a `displaced` entry, and `schema_version` raised to `2` — and both binaries were pointed
at it through `NOMOS_WORK_DIR`.

**`stale.exe work list` exited 0** and printed the whole board, saying nothing about the field it
could not see.

**`stale.exe work claim` exited 0** and granted the claim. The claim it wrote was correct. Comparing
the file against what it had been handed, one item had lost `displaced` entirely and no other key
set changed. Nothing in the output referred to it.

**The version it left behind was `2`.** The old binary echoed the number it read, so the file then
claimed to be a schema this build had never produced while no longer containing the field that
made it one — the state that would have been reported to the *next* old reader as a damaged file.
That is the observation that decided `Save` stamping over echoing, above.

**`fresh.exe work list` exited 5**, naming the field and both versions:

```
this build understands ledger schema 1 and the file is schema 2: unknown field `displaced`,
expected one of `id`, `title`, `why`, `done_when`, `territory`, `state`, `depends_on`,
`blocked`, `claim`, `verification`, `verified`, `abandoned` at line 2964 column 17. Writing it
back would drop what could not be read, so nothing was written. Rebuild (`cargo build -p
nomos-cli`) and retry
```

**`fresh.exe work claim` exited 5 and the file was byte-identical afterwards** — same md5 before
and after. That is the pairing the item is about: the same document, one binary that drops a field
and reports success, one that refuses and writes nothing.

**The current binary still reads the current board.** `validate`, `list`, `audit` and `show` were
run against the repository's real `work/ledger.json` and all exited 0, with `validate` printing
`ledger is valid (schema 1, and this build understands 1)`. `claim`, `renew` and `abandon` were run
against a copy of it under `NOMOS_WORK_DIR`: all exited 0, no item's key set changed, and the
`"abandoned"` count still equalled the item count. A guard that refused the current file would have
stopped the board rather than protected it.

## What This Does Not Reach

**Binaries that already exist.** A remedy is compiled in. A `nomos.exe` copied last week has none
of this and will go on dropping fields silently until it is replaced. What this decision buys is
that every binary built after it is a safe writer, so the next schema change — `P10-LAPSE-TAKEOVER`'s
`displaced`, in the commit after this one — cannot be lost this way. The copies already in flight
are not saved and this record does not claim they are.

**The copied-binary workaround is documented, not replaced.** Replacing it means changing how
finishing invokes its predicate, which is `OD-LEDGER-003`'s subject and a different failure mode,
and doing it here would be two decisions in one commit under one record. What this item owed is
that the workaround stops being *silent*, and it does: a copy that has fallen behind now fails at
its first verb with a sentence naming the remedy. README says so where an operator hitting exit 5
will look, and `nomos work validate` reports the file's schema and the build's side by side, so one
command answers "is the executable I copied current?".

**A file that claims a newer version while containing nothing new is still read.** The version is
not a guard, by decision, so a document whose every key this build declares parses however it is
stamped — and is then re-stamped to what this build understands. That is lossless in data and
visible in the diff. It is stated because it is the exact boundary of what the version does.

**The surface snapshot cannot hold this.** Serde attributes are not part of a public API, so
`deny_unknown_fields` moves no line in `tests/contract/surface/nomos-ledger.txt`; the two lines
that move are `SCHEMA_VERSION` and the new error variant. Deleting the guarantee is invisible to
every check in this workspace except the tests written for it. That is why the negative controls
matter more here than usual, and why the completeness test walks the document's shape rather than
naming types: a guard enumerated by hand is only as complete as the hand — `OD-COMPLETENESS-001`.

## Consequences

A stale session is stopped, not degraded. Every `nomos work` verb exits 5 until its operator
rebuilds, `list` included. That is the intended outcome and it is a cost: an agent mid-item
discovers it at the next verb rather than at the start. Refusing the read-only verbs too is
deliberate — a session allowed to read a document it cannot account for goes on deciding from a
board it is not seeing whole, which is the condition that produced the incident rather than the
write that ended it.

Additive schema changes are no longer backward compatible in the read direction. A new build still
reads an old file — every added field carries `serde(default)`, and a document written before
`abandoned` existed is asserted to still load — but an old build no longer reads a new one. That
asymmetry is deliberate and it is the whole trade: forward compatibility for this file was buying
nothing except the ability to lose it.

A key nobody declared can no longer be parked in the ledger by hand. Adding one now means adding
it to a type.

## Amendment: The Test Named In The Mutation Record Was Renamed

Version 1 reports the six controls one at a time, each reverted, and three of them name the one
test that went red. The name is written as it stood when the measurement was taken:
`Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key` does not resolve today. It is
`Test_Every_Node_In_A_Ledger_Should_Refuse_An_Undeclared_Key` in
`crates/substrate/nomos-ledger/tests/exclusion_holds/persistence.rs` since `6927e8d0`, the
naming pass that took `object` to `node` across this crate. Only the name moved.

**The sentence is left as measured, because it is a record of a measurement and not a claim of
coverage.** It says which test went red when a control was removed, and that is what the test
was called when it did. `OD-SPEC-017`'s own discriminator is the tense — present tense claims
coverage, past tense reports history — and this one is past. Writing the successor into the
sentence would have the record report a name that did not exist at the moment it says it
measured, which is the one edit `OD-SPEC-017` refused for a path and did not notice it also
applied to a test.

**What the sentence attributes to that test is re-measured rather than assumed**, because two
similar names are exactly the evidence a reader cannot check a rename from. Removing
`deny_unknown_fields` from `Claim` — the third of the three containers the sentence names —
fails `persistence::Test_Every_Node_In_A_Ledger_Should_Refuse_An_Undeclared_Key` and no other
test in `nomos-ledger`, out of 267 across its eight targets. The mutation was restored
byte-identically: `crates/substrate/nomos-ledger/src/claim.rs` hashes to
`b1ecdc9c349a4cce2a6c8c9832c92a0565b3ace3d28f445bd1c30b788eec360b` before and after.

## Status

Accepted, version 2. Implemented in `nomos-ledger`, reported by `nomos work validate`, and
documented in README. Amended once, to declare the name the measurement above was taken under
rather than to re-point it.

Six controls confirmed red, one at a time, each reverted. Removing `deny_unknown_fields` from
`LedgerItem` failed five tests, all of them in this item's own set and no other test in
`nomos-ledger` or `nomos-cli` moving. Removing it from `LedgerDocument`, from `Claim` and from
`Territory` each failed exactly one test —
`Test_Every_Object_In_A_Ledger_Should_Refuse_An_Undeclared_Key`, and nothing else — which is
precisely why that test exists: the three fixtures that carry an undeclared key on an *item* stay
green through all three, so without the shape walk, four of the ten guarded sites could be removed
with the suite still passing. Making `Explain` always answer `Malformed` failed the test that tells
the two causes apart, alone. Making `Save` echo the loaded version failed the test that the stamp
is this build's, alone.
