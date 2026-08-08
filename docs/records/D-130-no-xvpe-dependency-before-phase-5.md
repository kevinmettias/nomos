---
id: D-130
type: decision
title: Nomos takes no XVPE dependency before Phase 5, and never by path
status: accepted
version: 1
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
