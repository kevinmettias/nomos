---
id: OD-LEDGER-040
type: decision
title: A verification predicate answers for the artifact classes its territory holds, not for the crates those paths happen to sit in
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ledger
  - verification
  - territory
relations:
  - target: OD-LEDGER-039
    type: relates-to
  - target: OD-LEDGER-024
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
  - target: OD-GATE-007
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
---

# A verification predicate answers for the artifact classes its territory holds, not for the crates those paths happen to sit in

## Question

An item reserves files and declares a predicate. Nothing compares the two. So an item can
finish green having never verified part of what it changed, and the ledger records it as
done because the thing it was asked to run exited zero.

The obvious rule is to require the predicate to name every crate the territory touches. This
record refuses that rule, states what the obligation actually is, and says why the cheap
version was not merely incomplete but would have been wrong on the very instances that
motivated the question.

## The Decision

**A predicate need not name every crate its territory touches, and naming them all is not
what makes a predicate adequate.**

The obligation is per **artifact class**. For every class of artifact an item's territory
holds, at least one declared predicate must be capable of detecting an invalid change to that
class. Rust source resolves to compilation, its tests, *and* the nomos checks over that tree,
because those three catch different things. A surface snapshot resolves to the surface
contract test. A governing record resolves to specification validation and projection
freshness. A fixture resolves to a consumer test that actually loads it. Prose resolves,
today, to nothing — and that is a hole this record names rather than closes.

**A reserved path that belongs to no Cargo package is not an exception to be excused; it is
the ordinary case that shows why crates are the wrong unit.** Territory routinely names
records, surface snapshots, generated projections, shared manifests, fixtures and the ledger
itself. Each of those has a verification that judges it. None of them has a `-p`.

## Why The Crude Rule Is Refused

Three reasons, in the order they bite. The third is the one that settles it.

**It cannot be stated over the things territory actually names.** A rule quantified over
crates has nothing to say about the majority of reserved paths, so it would have to be
written as an exemption list, and an exemption list over "everything that is not a crate" is
not a rule.

**It rewards bookkeeping.** An author blocked by the check adds a `-p` until it passes. The
added package is compiled and its tests run, which costs time and proves whatever those tests
happened to prove. Nothing in the mechanism asks whether the addition can detect anything
about the change in hand.

**Measured against three instances it is right once, wrong once and silent once.** This is
the part that is not a prediction. See below.

## The Three Instances

**First: a predicate that did not reach its own territory.**
`P104-COMPARE-COLLAPSES-EVERY-FINDING-A-RULE-MAKES-ABOUT-ONE-SUBJECT-INTO-ONE-4` reserved
`crates/host/nomos-cli/src/gate/report.rs` and declared
`cargo test -p nomos-gate-orchestration -p nomos-api`, which does not build `nomos-cli`. The
implementation added a directory walk nested four levels deep. `nesting-depth` reported it as
blocking and two real-tree tests in `nomos-cli` went red. The declared predicate was green
throughout, and `work finish` would not have seen it either: its lint step is bare clippy,
whose severities `OD-GATE-007` settles in the workspace table with no `-D warnings`, and
`nesting-depth` is a nomos rule that neither `cargo test` nor `cargo clippy` runs. The same
item also reserved `tests/contract/surface/nomos-gate-orchestration.txt`, a path with no
owning package at all.

Crate coverage would have caught this one. That is the whole of its support.

**Second: a predicate that reached its territory by a better route, and still could not see
the change.** `P103-SELFCHECK-RED-SIXTEEN-BLOCKING-FINDINGS` reserved four paths across three
crates — `nomos-lang-rust-compiler`, `nomos-rules` and `nomos-analysis` — and declared
`cargo test -p nomos-cli --test check_command this_workspace`, naming a crate it does not
reserve and none of the three it does. Under crate coverage that item is defective.

It was not. That predicate runs nomos's own self-check over the entire workspace, reaching
every reserved file by scanning the tree rather than by the crate graph, which is a *stronger*
obligation than compiling the three packages.

It still shipped an invalid change. Commit `502fe7b3` renamed a binding from `s` to `leaving`
in `crates/substrate/nomos-analysis/tests/invalidation_order.rs`, and the replacement also
matched the `s` inside ten English possessives, so committed doc comments now read
`store'leaving own dependency edges`. Nothing in the repository judges English: every rule
judges identifiers, types, files or structure, a doc comment is not code so the compiler
cannot see it, and `cargo test` is green over the file. It was found by a person reading the
file, and filed as `P118-A-SINGLE-LETTER-RENAME-REWROTE-TEN-POSSESSIVES-IN-PROSE`.

**Now apply the crude rule to it.** It demands `-p nomos-analysis`. Adding that compiles the
file and runs its tests — and a doc comment is still not code, so the corruption still passes.
The rule fires on a correct item, is satisfied by an addition that verifies nothing about the
defect, and the defect ships anyway. That is the bookkeeping failure above, observed rather
than imagined.

**Third: the same shape again, four days later, with the check sitting in plain sight.**
`P40-MODEL-ROUTING-RESOLVER` reserved
`crates/orchestration/nomos-agent-orchestration/src/backend.rs` and declared
`cargo test -p nomos-agent-orchestration --no-fail-fast`. Crate coverage satisfied, by the
same reading that would have called instance one defective.

Commit `c0b61c1f` promoted `Backend::ALL` from a test-only constant to a public one, because
`Declared_Targets` now derives the declared package set from it. That makes it a declared
universe, and a declared universe is judged by `completeness_universes` in
`nomos-contract-tests` — a package the territory does not mention and the predicate does not
name. The item finished green. `cargo test -p nomos-contract-tests` was red at `HEAD` for
every session in the tree until
`P122-A-DECLARED-UNIVERSE-LANDED-AT-HEAD-CLAIMING-A-MIRROR-THAT-WAS-NEVER-WRITTEN` repaired it.

What makes this one the clearest of the three: the check already existed, already ran on every
full test invocation, and already knew how to fail on exactly this. Nothing connected it to
the artifact. Instance two could be dismissed as a hole nobody had dug yet — no rule judges
English — and this one cannot.

**What the three establish.** Put the crude rule against each of them and it behaves
differently three times.

On instance one it fires and is right, and that is the whole of its support.

On instance two it fires and is wrong. The predicate named `nomos-cli` and the territory named
three other crates, so the rule flags it — yet the item was correct, its predicate reaching
every reserved file by a stronger route than compilation. Satisfying the rule would have added
`-p nomos-analysis`, compiled the file, run its tests, and left a doc comment still not code.
The rule would have been obeyed and the defect would have shipped unchanged.

On instance three it does not fire at all. Crate coverage was satisfied, and the defect
shipped into `HEAD`.

So it is right once, wrong once, and silent once, and its one success is a case where what was
actually wanted — the nomos checks over that tree — is an artifact-class obligation crate
coverage names only by accident.

## The Mirror

`P103-A-LEDGER-PREDICATE-MUST-NOT-INHERIT-UNRELATED-CRATE-OBLIGATIONS` and its two
predecessors asked the opposite question: a predicate too *broad*, inheriting sibling
obligations because they share a crate. All three were declined, the last on 2026-09-14 for
its carrier rather than its substance.

They are one principle and deciding either alone would leave the other contradicting it. A
predicate scoped to crates is simultaneously too narrow — it misses artifact classes inside
the crates it names — and too broad, since it inherits every unrelated failure those crates
can produce. Scoped to artifact classes it is neither.

That second half is not theoretical either. This record's own item declared
`cargo test --no-fail-fast -p nomos-contract-tests` over a territory of two record files, and
was red on three crates it does not touch: two surface snapshots left stale by items that
finished without re-blessing them, and one a peer's in-flight work. The fix was
`P121-TWO-SNAPSHOTS-THE-PATH-ATTRIBUTE-REPAIR-MOVED-ARE-OWNED-BY-NOBODY-AND-REDDEN-THE-WHOLE-CONTRACT-SUITE`,
a cleanup item this one had to wait behind in order to run at all.

## The Convention That Applies Today

Binding on authors now, with no mechanism waiting to be built.

For each artifact class an item's territory holds, name a predicate that can detect an
invalid change to that class. For Rust source that means the nomos checks over the tree and
not merely compilation and linting, because instance one proved those are not the same thing.
For a snapshot, a record, a projection or a fixture, name the check that judges that artifact
— it exists, and it is not a `-p`.

Read the other way: do not add a package to a predicate to make a territory look covered. If
the added package cannot fail on the change in hand, it is cost without evidence, and
`OD-GATE-001` is the record about checks that report success without having looked.

## What Is Not Built Here

No ledger feature. The decision is that crate coverage is refused, so there is nothing
mechanical to add for it, and the artifact-class obligation cannot be enforced mechanically
today because no declared map from artifact class to verifying check exists. Building that
map is a separate item with a separate carrier — `work add`, `work finish`, or a contract
test over the committed board are three different answers — and it is not boarded by this
record.

The prose hole is left open deliberately and is recorded so it is not mistaken for an
oversight: an artifact class with no verification obligation at all is the one case the
convention above cannot help with, because there is no check to name.

## Status

Version 1 closed by `P104-A-VERIFICATION-PREDICATE-NEED-NOT-REACH-THE-CRATES-ITS-OWN-TERRITORY-NAMES-2`,
which held the first instance from 2026-09-14 until a second arrived.

The second is `P103-SELFCHECK-RED-SIXTEEN-BLOCKING-FINDINGS` by way of `P118`, and it is the
instance that decided the shape, because it is the one the obvious rule gets wrong. The third,
`P40-MODEL-ROUTING-RESOLVER` by way of `P122`, arrived unprompted on 2026-09-21 while this
record was being written, and is the one that made the shape unarguable: it satisfies crate
coverage and ships anyway, against a check that already existed.

That the third instance surfaced during the authoring of the record about it is not a
coincidence worth much, but it is worth stating: the item waited seven days for a second
instance and got two, which is the rate a defect class produces when nothing is stopping it.
