---
id: OD-HOST-012
type: decision
title: The transport allowlist is an incremental projection frontier and not a repo-tooling containment boundary, and repo-tooling handlers leave nomos-api when a second host wants the product half alone
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - host
  - api
  - architecture
  - repo-tooling
relations:
  - target: OD-HOST-006
    type: relates-to
  - target: OD-HOST-007
    type: relates-to
  - target: OD-LEDGER-036
    type: relates-to
  - target: OD-PACKAGE-015
    type: relates-to
  - target: OD-HOST-011
    type: relates-to
---

# The transport allowlist is an incremental projection frontier and not a repo-tooling containment boundary, and repo-tooling handlers leave nomos-api when a second host wants the product half alone

## Question

An eighth-round external architecture review observed that `nomos-api` exports the `work` and
`spec` handler families alongside `check`, `gate`, `correction`, `workflow` and `agent`, while
`nomos-api-transport` keeps `work` and `spec` off the wire through a closed `ServedMethod`
allowlist with tests that specifically reject `nomos.work.list` and `nomos.spec.commit`. It
read this as logical separation achieved by downstream refusal rather than by ownership, and
argued the need for such a test is itself evidence the dependency boundary is too broad.

`OD-LEDGER-036` already holds that the work ledger is repository bootstrap machinery rather
than a product workflow target, and `OD-HOST-006` already carries that record's trigger for the
spec half. Neither decides where the handlers live.

## What Was Measured

**The split, counted.** Measured 2026-09-21: `nomos-api` declares thirty `pub fn Handle_*`
entry points. Twenty-one are repo tooling -- eleven `Work` verbs and ten `Spec` verbs --
against nine product handlers: `Check_Run`, `Correction_Run`, four `Gate` verbs,
`Workflow_Run`, and two `Agent` verbs. The crate is 70 percent repo tooling by handler count.
The review's characterization is if anything understated.

**The review's central inference is wrong, checked directly.** It reads the allowlist as a
repo-tooling containment mechanism. `ServedMethod`'s members are exactly the operations
`ServedMethod::REGISTRY` declares, and what decides membership is what a call causes on the
host rather than which family a verb belongs to. It excludes `work` and `spec` -- and it also
excludes handlers that are product handlers by any reading: `Agent_Execute`,
`Agent_Judge_Role` and `Workflow_Run` are held outside it by name in
`Test_The_Agent_And_Workflow_Operations_Should_Stay_Refused`, and
`Test_Every_Refused_Operation_Should_Still_Be_Exported` keeps that refusal from going quiet
should one of them be renamed away; both are in
`tests/contract/tests/boundaries/transport_registry.rs`. The allowlist is not drawing the
repo-tooling line. It is the frontier of what has been deliberately projected so far, exactly
as `OD-HOST-007` describes, and `work`/`spec` are outside it along with the product verbs
refused on grounds of their own.

**So the tests prove something narrower than the review claims.** A test rejecting
`nomos.work.list` does not exist because a repo-tooling handler leaked into a product
dependency. It exists because `ServedMethod` is a closed set and `OD-LEDGER-036` names these
two families as ones that should never join it -- a permanent exclusion pinned by a test,
inside a surface that is incremental for everything else.

**`OD-PACKAGE-015`'s test, applied.** A `nomos-repo-tooling-api` crate today would be depended
on by `nomos-cli` alone. `nomos-cli` also depends on `nomos-api` for the product half, so no
consumer would depend on one without the other. No independent versioning, no isolation
boundary a module could not draw. All three clauses fail.

## The Decision

**The repo-tooling handlers stay in `nomos-api` for now, and the allowlist tests are
reclassified rather than removed.**

The tests stand, and their stated meaning changes: they pin `OD-LEDGER-036`'s permanent
exclusion, not a containment of leakage. A test asserting that `nomos.work.list` is unserved is
asserting a decision about the product's boundary, which is worth pinning regardless of which
crate the handler lives in.

**The move is licensed the moment a second host wants the product half alone.** This is a
`OD-PACKAGE-015` third-clause question and nothing else: when a consumer exists that needs
`check`/`gate`/`correction`/`workflow`/`agent` and does not want twenty-one ledger and
specification verbs compiled into it, the boundary is earned and the split should happen then.
`nomos-mcp` and `nomos-lsp` are the two plausible candidates and neither is that consumer
today, because both reach the product surface through `nomos-api-transport` rather than
`nomos-api` directly.

**What is refused explicitly.** Moving repo tooling to CLI-only, the review's third option, is
refused on `OD-HOST-002`'s ground: a surface holds no state its canonical services cannot
reconstruct, and `nomos work` and `nomos spec` have real orchestration seams whose whole point
is that a host is not the only thing that can reach them. Deleting the API-side reach would
re-privilege the CLI for a family that has already been de-privileged.

## What Would Decide It Differently

- **A host depending on the product handlers alone**, which fires `OD-PACKAGE-015`'s third
  clause and makes the split correct rather than speculative.
- **A repo-tooling handler reaching the wire.** Would mean the exclusion failed and the
  ownership boundary, not the allowlist, is what is needed.
- **`ServedMethod` closing.** If the product operations outside the allowlist are all
  projected and the surface stops being a frontier, the allowlist becomes a pure
  repo-tooling boundary and the review's original reading becomes the correct one.

## Status

Accepted. The review identified a real asymmetry and misattributed its cause; the correction is
that `ServedMethod` is an incremental frontier excluding product verbs alongside the two
repo-tooling families, not a containment line. Revisit on a second host wanting the product
half alone.

## Amendment, Version 2: The Enumerated Four Were A Frozen Size, And `f0052c39` Made Them False

Version 1 wrote that "`ServedMethod` has four variants" and listed them, that the allowlist
"also excludes `Check_Run`", and that `work`/`spec` sit outside it "along with four product
verbs". `f0052c39` admitted `nomos.check.run`, which makes the size wrong, the enumeration
incomplete, and the named exclusion of `Check_Run` false outright. The split this record
counted has moved too: `nomos-api` exports thirty handlers today rather than twenty-nine.

None of that touches the argument. The allowlist is an incremental projection frontier and not
a repo-tooling containment boundary *because* it excludes product verbs as well as
repo-tooling ones, and it still does -- which is what
`Test_The_Agent_And_Workflow_Operations_Should_Stay_Refused` now pins by name, where version 1
had only a list of four that admitting a fifth silently falsified. The counts are replaced by
the closed set and what decides membership in it; the one measurement that carries weight is
dated, the shape this record's sibling `OD-HOST-017` already used.

Amended by `P123-FOUR-HOST-RECORDS-FROZE-A-REGISTRY-SIZE-INTO-PROSE-AND-A-SIXTH-OPERATION-MADE-THEM-FALSE`.
