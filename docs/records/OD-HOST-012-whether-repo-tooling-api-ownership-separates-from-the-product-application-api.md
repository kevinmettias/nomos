---
id: OD-HOST-012
type: decision
title: The transport allowlist is an incremental projection frontier and not a repo-tooling containment boundary, and repo-tooling handlers leave nomos-api when a second host wants the product half alone
status: accepted
version: 1
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

**The split, counted.** `nomos-api` declares twenty-nine `pub fn Handle_*` entry points.
Twenty-one are repo tooling -- eleven `Work` verbs and ten `Spec` verbs -- against eight
product handlers: `Check_Run`, `Correction_Run`, three `Gate` verbs, `Workflow_Run`, and two
`Agent` verbs. The crate is 72 percent repo tooling by handler count. The review's
characterization is if anything understated.

**The review's central inference is wrong, checked directly.** It reads the allowlist as a
repo-tooling containment mechanism. `ServedMethod` has four variants: `GatePlan`, `GateRun`,
`GateExplain` and `Correction`. It excludes `work` and `spec` -- and it also excludes
`Check_Run`, `Workflow_Run`, `Agent_Execute` and `Agent_Judge_Role`, which are product
handlers by any reading. The allowlist is not drawing the repo-tooling line. It is the
frontier of what has been deliberately projected so far, exactly as `OD-HOST-007` describes,
and `work`/`spec` are outside it along with four product verbs.

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
- **`ServedMethod` closing.** If the four product families outside the allowlist are all
  projected and the surface stops being a frontier, the allowlist becomes a pure
  repo-tooling boundary and the review's original reading becomes the correct one.

## Status

Accepted. The review identified a real asymmetry and misattributed its cause; the correction is
that `ServedMethod` is an incremental frontier excluding four product verbs alongside the two
repo-tooling families, not a containment line. Revisit on a second host wanting the product
half alone.
