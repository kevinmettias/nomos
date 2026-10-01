---
id: D-138
type: decision
title: A capability Nomos needs before XVPE can hold it is named as Nomos's own, and migration is a rename rather than a naming reservation
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - platform
  - dependencies
  - ecosystem
relations:
  - target: D-130
    type: relates-to
  - target: D-135
    type: relates-to
---

# A capability Nomos needs before XVPE can hold it is named as Nomos's own, and migration is a rename rather than a naming reservation

## Decision

When Nomos needs a domain-neutral capability that `D-135` would otherwise send to XVPE, but
XVPE cannot yet absorb it — `D-130`'s Phase 5 gate has not opened, or no XVPE session is
actively building the capability — the capability is built inside this repository under an
ordinary Nomos-owned crate name. It is never given the `xvpe-` prefix, and no crate anywhere
in this repository is named as though it already belonged to XVPE.

Migration, when XVPE is ready to hold the capability, is a rename-and-move of the crate into
XVPE and a corresponding dependency-edge change in Nomos, governed by `D-130` exactly as any
other adoption: never a path dependency, never before Phase 5, only through the quarantined
`nomos-platform-xvpe` adapter. Building the capability early reserves nothing about its
future name and grants no exemption from `D-130`'s boundary test.

The crate's internal design still follows `D-135`'s intent: its public contract carries no
Nomos-specific vocabulary — no crate, rule, finding, claim, or concept reference — so the
later rename is mechanical rather than a rewrite.

## Rationale

`D-130`'s own Alternatives Considered already rejected vendoring a wanted capability under
its eventual name ahead of time, calling it a fork with no upstream path. Vendoring under
the `xvpe-` prefix specifically is the thing that record already declined, not a gap it left
open. The boundary test in `tests/contract/tests/boundaries/graph.rs` enforces this directly:
any workspace member whose transitive dependencies include a name starting with `xvpe-`,
outside `nomos-platform-xvpe`, fails — and a locally defined crate carrying that prefix would
trip the same assertion the moment anything in this workspace depended on it.

`D-135` answers where new domain-neutral code is authored the first time it is written, but
answers it under the assumption that XVPE is available to receive it. It does not by itself
answer the case this record closes: Nomos has a real, present need for a domain-neutral
capability, and XVPE either cannot build it yet or has no session doing so. Leaving that gap
unanswered invites exactly the naming shortcut `D-130` already rejected — staging code under
a name that presumes migration before migration is possible.

An ordinary Nomos-owned name costs one rename at actual migration time. That is cheap
relative to a fork under a name the boundary test would otherwise have to be weakened to
allow, and it keeps `D-130`'s naming guarantee — only `nomos-platform-xvpe` may say `xvpe-` —
true without exception for as long as this record stands.

## Consequences

No change to `D-130`'s boundary test, its Phase 5 gate, or its no-path-dependency rule. No
change to `D-135`'s authorship default for the case where XVPE can actually receive new work.

A future item that builds a domain-neutral capability ahead of an XVPE home names it as an
ordinary Nomos crate and may cite this record for why it is not named `xvpe-<something>`
despite being a migration candidate. This record creates no tracking mechanism for migration
candidates; if that bookkeeping becomes real work rather than a naming question, it is a
separate item once a concrete crate exists to track.

## Alternatives Considered

Amending `D-130` to carve out an explicit `xvpe-`-named staging exception, with the boundary
test updated to allow it and a manifest tracking migration intent, was considered and
rejected here: it revisits reasoning `D-130` already gave for rejecting local staging under
the target name, for a benefit — an accurate name a little earlier — that a one-time rename
at real migration time already provides at much lower cost.

Waiting for XVPE to be able to receive new domain-neutral subsystems before building anything
Nomos needs from that category was rejected: `D-130`'s own Alternatives Considered already
rejected waiting for XVPE to stabilize before starting Nomos at all, for the same reason —
the port-trait pattern makes the dependency optional, so Nomos's own progress does not need
to pause for XVPE's schedule.
