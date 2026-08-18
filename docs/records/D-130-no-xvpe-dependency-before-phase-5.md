---
id: D-130
type: decision
title: Nomos takes no XVPE dependency before Phase 5, and never by path
status: accepted
version: 2
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
