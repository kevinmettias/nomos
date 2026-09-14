---
id: D-130
type: decision
title: Nomos takes no XVPE dependency before Phase 5, and never by path
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - platform
  - dependencies
relations:
  - target: ARC-SPECDB-001
    type: affects
---

# Nomos takes no XVPE dependency before Phase 5, and never by path

## Decision

No Nomos crate depends on the sibling XVPE workspace before Phase 5. The platform port
traits exist from Phase 0, so adopting an XVPE implementation later is a new implementation
behind an existing seam rather than a refactor.

When adoption happens it is by git reference and commit SHA into one quarantined crate,
`nomos-platform-xvpe`. It is never a `path` dependency.

## Rationale

XVPE does not currently compile. Its own error log ends with a dataflow crate failing on
62 errors, and the in-flight refactor is concentrated in exactly the crates Nomos would
want: dataflow, scheduling, serialization. A `path` dependency would make Nomos's
buildability a function of another product's refactor.

Several crates that sound reusable are not. `xvpe-composer-headless` is an offscreen GPU
renderer, so "headless" there means "no window system" rather than "no graphics", and
depending on it would put a GPU stack under a command-line tool. `xvpe-telemetry` pulls a
desktop OS backend. `xvpe-replay` is game-world rewind. There is no storage crate and no
package crate; those are specified rather than built.

A commit SHA is the difference between adopting a body of work and adopting whatever that
work becomes.

## Consequences

`deny.toml` bans the GPU, windowing and UI crates on the default feature set, so a headless
build cannot acquire them transitively even by accident.

A boundary test asserts that only `nomos-platform-xvpe` may name anything beginning with
`xvpe-`. That crate does not exist yet; naming it in the test now means the exception is a
decision somebody wrote down rather than a line added later to make a failing test pass.

Genuine candidates stay identified rather than forgotten: hazard detection over read and
write sets, cycle witnesses over strongly connected components, dirty propagation and
topological layering for fact invalidation, and the graph algorithms.

## Alternatives Considered

Vendoring the wanted algorithms was rejected for now. It creates a fork with no upstream
path, and the algorithms are not yet on any critical path.

Waiting for XVPE to stabilize before starting Nomos was rejected. The port traits make the
dependency optional, so there is nothing to wait for.

## Amendment: The Build-Instability Premise Resolved; The Gate Did Not

The rationale above cited a specific, dated fact: a dataflow crate failing on 62 errors,
with the in-flight refactor concentrated in dataflow, scheduling and serialization.
Checked directly against xvpe dev at commit `557dada32ed0c33c7c2fde09b5320d3fdbc94d91`,
`cargo check -p xvpe-dataflow -p xvpe-scheduling -p xvpe-task-host -p xvpe-clock -p
xvpe-serialization` succeeds, warnings only. All five crates compile clean, including the
three this record named as blocking. The genuine candidates already identified above live
inside them: `hazard_detector.rs`, `cycle_witness.rs` and `topological.rs` sit in
`xvpe-scheduling`; dirty propagation sits in `xvpe-dataflow`. The specific instability this
record was written against no longer describes the tree.

That does not lift the gate, because two of this record's other conditions are unchanged
and neither is a build-stability question. xvpe's workspace — 112 members at this
commit — still contains no storage crate and no package crate; a clean compile cannot
adopt a crate that was never written. `xvpe-telemetry`'s `Cargo.toml` still names
`xvpe-os-backend-desktop` as a direct dependency, exactly as it did when this record was
accepted. A third fact sharpens the same point rather than adding a new one: xvpe as a
whole is organized as `crates/engine/{foundations,simulation,runtime,presentation}` —
rendering, audio, physics, animation, camera, HUD, materials and scene crates sit beside
the ones Nomos would actually want, inside one workspace. A `path` dependency does not
select the crates that happen to compile today; it selects the workspace, and this is a
full game engine's workspace, not a narrow platform library's.

The phase gate is therefore not narrowed to a compile-stability condition on the named
crates. Compiling is necessary — an XVPE that does not build would be disqualified on that
fact alone — but this reverification is the demonstration that it is not sufficient: the
exact crates this record called out now compile cleanly, and the record's other reasons to
wait are untouched by that. Phase 5 stands as the gate, unconditioned on whichever phase
number the tree happens to be build-stable at, because what Phase 5 is waiting for — a
storage crate, a package crate, and an adoption path that does not pull a desktop OS
backend or a rendering engine in behind it — is not a fact about whether `cargo check`
currently exits zero.

Checked 2026-08-18 against xvpe dev HEAD `557dada32ed0c33c7c2fde09b5320d3fdbc94d91`.

## Amendment: The Adapter Count Was Retired; The Path Clause And The Phase Gate Were Not

This record made two mechanical decisions about the XVPE crossing, and only one of them has
been retired. Read together they looked like a single rule, which is how the other came to
lapse without anybody deciding that it should.

**What was retired.** The boundary test this record's Consequences section describes is gone.
`Test_Only_The_Platform_Adapter_May_Name_The_Sibling_Workspace` does not resolve: it was
removed on 2026-09-10 by the owner's decision, and `tests/contract/tests/boundaries/graph.rs`
carries the tombstone where it stood. What it enforced was an adapter *count* — that exactly
one named crate may reach an `xvpe-` dependency. `OD-PLATFORM-003` holds the reasoning: nomos
is built on top of XVPE as the knowledge workbench is, XVPE is the engine and this workspace
an application over it, so a rule forbidding that dependency does not describe an architecture
worth keeping. The tombstone records why it was retired rather than widened — an allow-list of
nine crates would have left a rule that still reads as a boundary while enforcing nothing,
which is worse than no rule.

**That retirement is accepted, and this record's quarantine clause is withdrawn with it.** More
than one crate here may name `xvpe-`, and `nomos-platform-xvpe` is no longer the sole permitted
crossing.

**What was not retired.** Two clauses survive, and neither is a claim about how many crates may
cross.

The first is the *form* of the dependency: adoption is "by git reference and commit SHA", and
"never a `path` dependency". Retiring the quarantine did not decide the pinning. They answer
different questions — one asks how many crates may cross, the other asks whether this
workspace's buildability and reproducibility are a function of another product's working tree
at whatever revision that tree happens to be sitting at, with nothing recording which revision
that was. The second question's answer does not change with the number of crossings, so
nothing about the 2026-09-10 decision reaches it.

The second is the **Phase 5 adoption gate**, which stands unless a record supersedes it
deliberately. It is not superseded here.

### The gate conditions, dispositioned one at a time

The amendment above dispositioned this record's build-instability premise and deliberately did
not let the gate fall with it. The same discipline applies now: the conditions are checked
individually rather than lapsing together because a different clause was retired. Measured
2026-09-14, this workspace at `8503ddb3`, xvpe `dev` at
`82a3c8fccf4ef7f3759f36d3f320a91d0f96341c` with a clean tree.

| Condition, as this record stated it | 2026-08-18 | 2026-09-14 |
|---|---|---|
| xvpe does not compile; dataflow fails on 62 errors | resolved | resolved, and still so |
| xvpe's workspace contains no storage crate | true, blocking | **still true.** Across 262 member manifests the only `[package]` name suggesting one is `xvpe-collections-persistent`, under `foundations/data/collections` — an immutable-collections data structure, not a store |
| xvpe's workspace contains no package crate | true, blocking | **still true.** No `[package]` name matches a package or registry crate |
| `xvpe-telemetry` names `xvpe-os-backend-desktop` as a direct dependency | true, blocking | **resolved.** Its dependencies are now `xvpe-collections-pool`, `xvpe-primitives` and `xvpe-thread-pool`, and the only manifests naming the desktop backend are two demo apps and the two os-backend crates themselves |
| a `path` dependency selects the workspace, and this is a game engine's workspace | true | **changed, and narrower in practice.** The tree is now 262 members across eleven top-level groups, against 112 when this was written. But what this workspace actually reaches is twelve `xvpe-` crates, and no member reaches a GPU, windowing or UI crate |

**Two conditions still block, one has resolved, and one is narrower than feared.** Phase 5 has
therefore not opened, and the adoption now standing in the tree preceded it. That is a fact
about sequence, not a licence: this record does not retroactively admit what arrived early.

### What the tree carries against the surviving clause

Sixteen dependencies across eight workspace members — seven crates under `crates/` and the
contract test crate, itself a member — are spelled as a relative path climbing out of this
repository into a sibling directory named `xvpe`. Each is the form this record's surviving
clause names as the one adoption never takes.

Replacing them is available rather than theoretical: the sibling checkout has a real remote at
`https://github.com/kevinmettias/xvpe.git`, so a `git` and `rev` dependency of exactly the
shape decided above can be fetched today, and a pinned revision is what makes the adopted
commit a recorded fact rather than an ambient one.

No manifest moves by this record, the same as everywhere else in it.
`P98-SIXTEEN-PATH-DEPENDENCIES-CROSSED-A-BOUNDARY-NO-DECISION-RETIRED` holds that work and
carries the alternative this record does not foreclose: if replacing them is refused for a
measured reason, the refusal is adjudicated and this clause superseded explicitly, rather than
left contradicted by a tree that outvoted it.

Checked 2026-09-14 against this workspace's own manifests: the sixteen path dependencies across eight members that the surviving clause refuses, and the sibling remote that makes replacing them available rather than theoretical.
