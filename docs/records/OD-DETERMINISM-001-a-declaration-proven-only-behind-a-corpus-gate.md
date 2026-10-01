---
id: OD-DETERMINISM-001
type: decision
title: A determinism declaration proven only behind a corpus gate is proven nowhere
status: closed
version: 1
authority: canonical-normative-record
tags:
  - determinism
  - contracts
  - verification
relations:
  - target: OD-GATE-001
    type: affects
  - target: ARC-SPECDB-001
    type: relates-to
---

# A determinism declaration proven only behind a corpus gate is proven nowhere

## Question

`nomos-contracts` has carried `Strategy`, `DeterminismStrength`, `ReproducibilityScope`
and `TraceEquivalence` since it was written, together with a six-row table naming which
execution domain claims what. No crate in the workspace implemented any of them.

Two questions had to be answered before any of it could be discharged, and they pull in
opposite directions: whether the contracts crate should grow to support the declarations,
and what would count as having checked one.

## What Was Not Widened, And Why That Is The Decision

**Nothing was added to `nomos-contracts`.** The four types, the trait and
`Declaration_Is_Coherent` were sufficient to declare against and to check against, and the
crate is unchanged by this item.

That is worth recording because the opposite was the obvious move. A verification harness
wants a value-level triple, a registry of declared domains, and somewhere to put both, and
`nomos-contracts` is where the vocabulary already lives. Every one of those would have been
a published protocol commitment. Every type in that crate is reimplemented by peers that
will never compile it — a knowledge service in OCaml, a client in TypeScript, a platform in
another Rust workspace — and
`Test_Contracts_Should_Depend_On_The_Allowlist_And_Nothing_Else` exists to keep the crate
neutral enough that they can. A harness is not a protocol. It went to `tests/integration`,
which links everything and commits no peer to anything.

The rule this leaves behind: **the contracts crate states what a promise means; it does not
state who made one or hold the machinery that checks it.** A later item wanting a registry
of declared domains should read this before adding one there.

## What Counted As Checked

The properties were never absent from the code. `Test_Reading_The_Corpus_Twice_Should_Reach_The_Same_Facts`
has asserted the parser's determinism over 7,500 real files since `P7-RUST`, and
`Test_A_Hundred_Ingestion_Orders_Should_Yield_Byte_Identical_Queries` has asserted snapshot
identity across a hundred ingestion orders since `P7-WORKSPACE`. Both are stronger
instruments than anything this item built.

Both are corpus-gated, and `OD-GATE-001` already measured what that means: without the three
environment variables, 68 assertions return early and print `ok`, and `.github/workflows/gate.yml`
sets none of them. So both proofs are inside the hole, on every machine the gate runs on.

**A declaration whose only evidence is a gated test has never been checked by CI.** That is
`OD-GATE-001`'s finding applied one level up: that record is about assertions that do not
run, and this is about a *claim* whose truth rests entirely on assertions that do not run.
The claim is worse than the assertion, because a claim is what another system reads and
plans around.

So the four declarations are checked twice over: by the gated tests, which have the scale,
and by fixtures held in this repository, which have the reach. Neither replaces the other
and the doc comment on each gated test now says so.

## What Was Built

Four domains declare, and each declaration is checked against the domain's behaviour:

| Domain | Declaration | Where |
|---|---|---|
| Parsing a file into a fact | `StateTemporal` / `CrossPlatform` / `BitIdentical` | `nomos-lang-rust` |
| Scanning a file into a fact | `StateTemporal` / `CrossPlatform` / `BitIdentical` | `nomos-lang-rust-scan` |
| Serving a fact from the cache | `State` / `CrossRun` / `BitIdentical` | `nomos-analysis` |
| Encoding a workspace snapshot | `State` / `CrossBinary` / `BitIdentical` | `nomos-workspace` |

The obligation is derived from the declaration rather than chosen by the author.
`Verify` selects byte comparison or set comparison from the declared `STRENGTH`, and
`Cross_Environment_Owed` maps the declared `SCOPE` to what must happen outside the process:
nothing, a second process, or a second process and agreement with a digest committed to
this tree. Raising a declaration raises what is checked with no second edit, and lowering
one is the sanctioned repair — visible in a diff as a weakened promise rather than as a
deleted test.

One consequence is worth stating because it was nearly got wrong. The digest compared
across processes is taken **at the declared strength**: byte-wise for `StateTemporal`,
over the sorted line set for `State`. Comparing raw bytes for every domain would hold a
`State` domain to byte-stable ordering across processes, which is `StateTemporal` — one
step above what it declared. The check would have passed on the day it was written and
failed the first time a `State` domain legitimately reordered, naming a promise nobody
made. Keeping the axes independent is the whole reason the triple has three of them.

`Test_Every_Fact_Producing_Crate_Should_Declare_A_Strategy` is the half that stops this
recurring. A crate that constructs a `MaterializedFact` and declares nothing fails it —
which matters because a crate that declares nothing has no test of its own to fail. The
producer set is derived by scanning source, never declared, for the reason the corpus-gate
table is derived: a hand-kept list understates the set, and the crate nobody added is the
crate nobody checked.

Five controls confirmed red: an unordered collection reaching the parser's payload (caught
as `StateTemporal` in repetition, and separately as `State` by the cache, each naming its
own strength); a value that varies per process but not within one, which in-process
repetition passed and only the second process caught; a producer with its declaration
removed; a scan matching nothing, which the vacuity guard caught and which would otherwise
have let every assertion pass over an empty set; and a golden digest off by one character.

## What This Does Not Cover

Two of the domain table's six rows are undeclared. Spec-bundle serialization and the
projection engine both live in `crates/spec`, which this item did not claim and did not
touch. `tests/integration/tests/determinism.rs` names them in
`Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered`, so the gap is a statement
in a test rather than an omission somebody has to notice.

The sixth row — progress UI, logs, telemetry, agent execution, declaring `None` — has no
domain in the tree yet. The CLI prints, and nothing about what it prints is a fact.

The `CrossPlatform` and `CrossBinary` claims rest on a digest captured on one platform and
committed here. That is the instrument the contracts crate prescribes, with the reference
stored rather than fetched, and it becomes a real cross-platform comparison the first time
this gate runs on a second operating system. Until then it is a claim checked against one
platform's answer, and saying so is worth more than implying three.

## Status

Closed by `P9-DETERMINISM`. The declarations exist, each is checked against behaviour, and
the two gated assertions that always held the property now name the declaration they were
about.
