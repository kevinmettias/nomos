---
id: OD-RULES-023
type: decision
title: Write authority is a declared allow-list over an existing fact; ownership needs its own declaration first
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
relations:
  - target: OD-RULES-003
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
  - target: OD-CONNECTOR-001
    type: relates-to
---

# Write authority is a declared allow-list over an existing fact; ownership needs its own declaration first

## Question

`Check_Dependency_Direction` proves the shape this workspace's rule system is for: a
declared architecture beside an observed fact, judged into a `Finding`. A person required
two named extensions of it — declared component ownership against the observed public and
dependency surface, and declared write authority against observed mutation paths, "the
property `nomos-store` was built around and which nothing currently checks." Both need a
real mechanism decided, not assumed from the shape of the first rule.

## What Was Measured

**Write authority is already, accidentally, true — and checkable with a fact this crate
already reads.** `nomos-store`'s own README row states "one write door per authority."
Measured directly: exactly one crate, `nomos-workspace`, declares `nomos-store` as a real
dependency (`grep`'d across every `Cargo.toml` in the workspace), and its own
`workspace.rs` is the only site anywhere that imports `nomos_store::{Commit, DocumentStore,
...}`. `Check_Dependency_Direction` already reads `nomos.cap.dependency.edges` — the
identical fact — to judge zone crossings. A second judgment over the same fact, checking a
crate's dependents against a declared allow-list rather than a declared zone, is the
identical shape `tests/contract/tests/boundaries/graph.rs`'s own `PLATFORM_ADAPTER` and
`KNOWLEDGE_ADAPTER` already use: name every crate permitted to depend on the authority
crate; anything else that does is the second write door this property exists to catch.

**A naive reading of "ownership" is falsified by real data, not merely doubtful.** The most
direct reading — no two crates define a top-level type of the same name — was checked
against every committed surface snapshot in `tests/contract/surface/`. `Refusal` is
independently defined in nine different crates; `Visibility`, `Scope`, `Reading`,
`PayloadRefusal`, `Member`, `ManifestError`, `ItemKind`, `Item` and `Disposition` each in
three. This is not drift: it is ordinary, idiomatic Rust — a small crate naming its own
local concept `Refusal` and relying on `nomos_foo::Refusal` versus `nomos_bar::Refusal` to
disambiguate, exactly what the module system exists to let it do. A rule built on this
reading would report roughly two dozen findings on its first real run, every one of them a
false positive against this workspace's own established style, not a real ownership
conflict.

**Nothing in this workspace declares ownership as a structured claim today.** README's own
"Owns" column is prose, read by a person, not parsed by anything — `OD-PROJECT-001` already
decided this file stays hand-authored for exactly that reason. `OD-CONTRACTS-001`/
`OD-CONTRACTS-005`'s own admission tests reason in prose about why one type belongs in one
crate, checked by a person reading the record, not by a rule reading a table.
"Ownership" has no `ZONES`-shaped declaration to check an observed fact against yet — unlike
write authority, which only needed a second table over a fact this crate already reads,
ownership needs the table itself designed first: what a crate claims (a type name? a
capability id? a file path prefix?), and what "the actual public and dependency surface"
means as the observed half of that same claim.

## The Decision

**Write authority is a declared allow-list, checked as a third rule over
`nomos.cap.dependency.edges` — real, ready to build.** A table names, for each crate that is
the sole intended write door for some authority, exactly which crates may depend on it
directly: `nomos-store` → `nomos-workspace`, the only real entry this workspace has today. A
crate outside the named set that depends on the authority crate is the second write door
`nomos-store`'s own design was built to prevent, reported the identical way
`Check_Dependency_Direction` already reports a zone violation. This needs no new fact family
— `nomos.cap.dependency.edges` already carries everything the check reads — and no change to
`nomos-store` itself, whose "one write door per authority" property this rule only makes
mechanical rather than trusted.

**Ownership is not decided here.** The item that named it asked for a rule against "the
actual public and dependency surface," and this record's own measurement found the one
concrete reading anyone could check today — type-name uniqueness — actively wrong for this
workspace's real style. Building the rule anyway, over a definition already shown to
misfire, would ship the exact false-coverage failure `OD-COMPLETENESS-001` and this
session's own `NoUndeclaredFillerTemplate` rule both exist to catch elsewhere: a check that
runs and reports, having checked a definition of the property that was not the property. A
real definition — what a crate claims, in what structured form, checked against which
observed fact — is a question of the same shape and the same weight `OD-RULES-020` answered
for zones, not a detail to improvise while building the rule that reads it.

## What This Record Does Not Do

**No code changes here.** The write-authority table, its rule, its composition into a real
`nomos check` run, and the finding count measured before composition — this item's own
`done_when` for that half — are a follow-up item's own territory, named completely rather
than discovered mid-claim: `crates/rules/nomos-rules/src/checks/architecture_authority.rs`
(new), the composition site in `nomos-check-orchestration` that wires
`Check_Dependency_Direction` today, and whichever contract-test file would carry a
`WRITE_DOORS`-style declaration if one is kept outside `nomos-rules` the way `PLATFORM_
ADAPTER` is.

It does not design ownership's own declaration format. It names the one reading measured
and rejected, and the two questions a real design needs to answer — what is claimed, and
what observed fact it is checked against — without answering either, the same way this
record's own predecessor left plugin, transport and deterministic-subprocess boundary
shapes named but undecided in `OD-EXECUTOR-002`.

It does not reduce this item's own scope from two rules to one. Write authority is decided
and ready; ownership is a second decision this record did not reach, not a requirement this
record is declining.

## Status

Accepted. Write authority: a declared allow-list over the existing dependency-edges fact,
ready for a follow-up item to build. Ownership: the one mechanical reading available today
is measured and found wrong; a real definition is undecided and left open.
