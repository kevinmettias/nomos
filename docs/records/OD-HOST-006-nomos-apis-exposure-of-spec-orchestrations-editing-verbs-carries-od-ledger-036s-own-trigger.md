---
id: OD-HOST-006
type: decision
title: nomos-api's exposure of spec-orchestration's editing verbs carries the same named revisit trigger OD-LEDGER-036 gave the ledger verbs, and hasn't fired either
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - api
  - spec
  - architecture
relations:
  - target: OD-LEDGER-036
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# nomos-api's exposure of spec-orchestration's editing verbs carries the same named revisit trigger OD-LEDGER-036 gave the ledger verbs, and hasn't fired either

## Question

An external architecture review named `nomos-api` a product surface mixing genuine end-user
product functionality with this repository's own development tooling — specifically, that it
exposes both the full work-ledger verb set and specification editing/commit operations,
neither of which an end-user repository configuring Nomos would ever call, alongside `check`/
`gate`/`agent`, which it would.

`OD-LEDGER-036` already checked the ledger half of this claim directly and settled it: it
finds `nomos-api::work.rs` "already exposes the full verb set, but frames itself as a seam
exercise, not a product surface," decides that framing is currently honest, and names an
exact, checkable revisit trigger — "if `nomos-api` becomes a real, externally-consumed
product surface... its exposure of the full ledger verb set at band 90 stops being an
internal coordination detail and starts being a public commitment." That record's own "What
This Record Does Not Do" section is explicit that it answers only the ledger's exposure, not
`nomos-spec-orchestration`'s. `nomos-api::spec.rs` exposes `Preview`, `Render` and `Commit` —
the same commit-authority shape `OD-LEDGER-036` examined for the ledger's `Claim`/`Finish`
— and no record checks whether that half crosses the line `OD-LEDGER-036` drew.

## What Was Measured

**`nomos-api::spec.rs` genuinely exposes commit-authority verbs, not just read access.**
`crates/host/nomos-api/src/spec.rs` defines nine handlers: `Handle_Spec_Profiles`, `_Sources`,
`_Record`, `_Table`, `_Markdown`, `_Freshness`, `_Preview`, `_Render`, `_Commit`, `_Submit`.
The first six are read-only (list profiles, list sources, read one record, read a table, render
markdown, check freshness). `Preview`/`Render`/`Commit` are the same three-step "stage, preview,
then commit" authority `nomos-spec-change`'s own procedure requires before any specification
edit lands — `Handle_Spec_Commit` is a second, real caller of the exact commit path that writes
into the store, parallel to `nomos-cli`'s own `spec commit`.

**Its own module doc frames this the identical way `OD-LEDGER-036` found honest for the
ledger.** `spec.rs:1-6`: "A third real caller of `nomos_spec_orchestration` verbs" — naming
itself a seam-exercise caller (the orchestration crate's third caller, after `nomos-cli` and
whichever else), the same "proving the seam has a second real caller" framing `OD-LEDGER-036`
already accepted for `nomos-api::work.rs` rather than a claim that specification authoring is
itself a capability an end-user repository would want from Nomos's product API.

**`README.md` marks `nomos-spec-orchestration` the same way it marks the ledger crates.**
Row for `nomos-spec-orchestration` carries the same `[repo tooling]` marker `OD-LEDGER-036`'s
own evidence cited for `nomos-ledger`/`nomos-work-orchestration`/`nomos-surface-provenance` —
this repository's own specification-authoring machinery, not a capability `nomos check`/`nomos
gate` need or an end-user repository's own specification content would ever touch (the
distinction `ARC-SPECDB-001` and the `nomos-spec-change` skill both draw between this
repository's own governing records and anything a user's repository owns).

**No real external consumer exists yet, the same absence `OD-LEDGER-036` found for the
ledger.** Nothing in this workspace outside `nomos-cli` and `nomos-api` itself calls
`nomos-api::spec`'s HTTP-shaped handlers; both are this repository's own tooling calling its
own seam.

## The Decision

**The same settled answer extends to spec-orchestration's exposure, under the same trigger,
which has not fired.** `nomos-api::spec.rs`'s `Preview`/`Render`/`Commit` verbs are, like the
ledger verbs, currently an honest seam-exercise — a second real caller proving
`nomos_spec_orchestration` has one, not yet a public commitment to specification-authoring as
an end-user product capability. This record extends `OD-LEDGER-036`'s reasoning rather than
re-deriving it: both crates are marked `[repo tooling]` for the identical reason (this
repository's own bootstrap/authoring machinery, not something `nomos check`/`nomos gate` or an
end-user's own repository need), both are exposed through `nomos-api` with the identical
"proves the seam has a caller" framing, and neither has the one thing that would make the
exposure a public commitment: a real caller outside this repository's own tooling.

**The trigger is the same one, stated once so both crates share it rather than drifting into
two near-identical but separately-worded conditions:** if `nomos-api` becomes a real,
externally-consumed product surface, its exposure of spec-orchestration's commit-authority
verbs — like its exposure of the ledger's — stops being an internal coordination detail and
starts being a public commitment, at which point both halves need re-examining together, not
separately, since they would cross the line at the same moment for the same reason.

## What This Record Does Not Do

It does not forbid `nomos-api::spec` from existing or from growing more verbs, the same
non-prohibition `OD-LEDGER-036` states for the ledger half.

It does not move, rename, or gate any code. `nomos-api::spec.rs` is unchanged.

It does not decide whether `nomos-spec-orchestration`'s underlying capability — editing and
committing this repository's own governing records — would also be a reasonable product
capability for an end-user repository's own specification content someday. Like
`OD-LEDGER-036`'s equivalent disclaimer, it decides only that nothing in this workspace
exercises it as one today.

## Status

Accepted. Extends `OD-LEDGER-036`'s ownership-and-exposure reasoning to
`nomos-spec-orchestration`'s exposure through `nomos-api`; the shared trigger has not fired for
either half. Revisit both together if `nomos-api` becomes a real external product surface.
