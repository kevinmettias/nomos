---
id: OD-AGENT-004
type: decision
title: A restated fact goes stale exactly where nothing checks it, so the rule is route-what-is-checked-elsewhere rather than write-less-prose
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - documentation
  - architecture
  - staleness
relations:
  - target: OD-AGENT-001
    type: relates-to
  - target: OD-GATE-011
    type: relates-to
  - target: OD-HOST-001
    type: relates-to
  - target: OD-PLATFORM-002
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
---

# A restated fact goes stale exactly where nothing checks it, so the rule is route-what-is-checked-elsewhere rather than write-less-prose

## Question

An eighth-round external architecture review named documentation volume as a form of bloat:
`Cargo.toml` carrying architectural essays per member, `README.md` carrying crate and zone
tables, module roots carrying chronological narratives of which item added which increment,
and the same architectural fact therefore standing in a governing record, a README table, a
manifest comment, a module doc and a commit message at once. It recommended compacting the
prose and generating the dependency and package tables from one machine-readable declaration.

`OD-AGENT-001` already decided this shape for agent instruction files: an unchecked copy of a
checked file is a defect, and an instruction file routes to authority rather than restating it.
Whether that principle extends past `AGENTS.md` to manifest comments and module-root prose was
undecided.

## What Was Measured

**The volumes the review cites are accurate.** `Cargo.toml` is 30,191 bytes, of which 276 of
502 lines are comments -- 55 percent prose. `README.md` is 41,559 bytes. Module roots do carry
per-item narratives.

**But volume did not predict staleness, and a natural experiment settled it.** On 2026-09-05,
`OD-PLATFORM-002` gave `nomos_platform::FileSystem` a `Read_Directory` operation
(`P41-PLATFORM-DIRECTORY-ENUMERATION-3`, `093a0e4e`). The next day, the claim that no such
operation exists was still standing at **fifteen sites across eight crates and one test crate**,
plus `OD-HOST-001`, `OD-HOST-002` and `OD-LEDGER-025`.

**Every one of those fifteen was in a module doc comment. None was in `README.md`. None was in
`Cargo.toml`.**

That is the finding. The two artifacts the review names as bloated are the two that stayed
correct, and they stayed correct for a reason that is already in this repository:
`tests/contract/tests/boundaries/readme.rs` and `manifest_bands.rs` assert the README's tables
and the manifest's band assignments against the real workspace, both directions. The prose that
went stale was the prose nothing checks.

**The failure had a propagating form, which volume also does not explain.**
`nomos-surface-provenance::discovery` did not restate the claim independently -- it quoted the
sentence out of another module as its own stated authority. A false claim spread by citation.
Two further sites enumerated the port's operations by name and were wrong about the port's
shape rather than about one operation, having also never learned about `Remove_File`.

**The cost was paid outside the repository.** An external reviewer, reasoning carefully from
committed documentation, read one of the fifteen and recommended against adding directory
enumeration to a port that had gained it the day before. `OD-GATE-011` names this defect class;
this is the first instance where its cost is legible as a wrong answer given to a real reader
rather than as maintenance burden.

## The Decision

**`OD-AGENT-001`'s rule extends, and it extends on the checked/unchecked axis rather than the
volume axis: prose must route to an authority for any fact that a mechanical check already
holds, and may state a fact freely where it is the authority.**

Concretely, in a module doc comment:

- **A fact another artifact checks is routed to, never restated.** Band membership, crate
  ownership, a port's operation set, a record's decision. The doc names the authority; it does
  not reproduce its content.
- **A fact this module is the authority for is stated fully, and length is not a defect.** Why
  this type has these variants, what a function refuses and why, what a decision cost here.
  `nomos-cli::check::sources::Walked_Sources` is the worked example: it holds the real reason a
  recursive walk is not the port's one-level primitive, and it was the one site in the whole
  population that was correct, because it was the authority rather than a copy of one.
- **Quoting another module's prose as your own stated authority is refused outright.** It
  creates a citation edge with no mechanical backing, which is how a false claim propagated
  here. Route to the record, or state your own reason.

**The review's two specific recommendations are declined, on this record's own evidence.**
Compacting `Cargo.toml` and generating the README tables would spend effort on the two
artifacts that demonstrably did not fail, and generating the README tables would remove the
contract tests' subject -- the tables are checked *because* they are authored, and a generated
table asserts nothing about a hand-maintained fourth copy the way `manifest_bands.rs` does
today.

**No new gate step is added by this record.** A rule that detects a restated-rather-than-routed
fact would need to know which facts have mechanical authorities elsewhere, which is not
derivable from text. This record governs how the prose is written; it does not claim a checker
exists for it.

## What Would Decide It Differently

- **A second staleness population found in `README.md` or `Cargo.toml`.** Would falsify this
  record's central measurement and reopen the volume argument.
- **A mechanical way to detect a restated fact.** Would turn this from a writing rule into a
  checkable one, and is the increment worth wanting.
- **A checked artifact going stale anyway**, which would mean the checks are narrower than the
  facts they appear to cover.

## Status

Accepted. Decided on a natural experiment rather than on a principle: one port change on
2026-09-05, fifteen stale restatements the next day, all fifteen in unchecked module prose and
none in the two artifacts the review called bloated. The remedy is routing what is checked
elsewhere, not writing less.
