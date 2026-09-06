---
id: OD-RULES-020
type: decision
title: A dependency model names its zones and its allowed edges rather than ordering everything on one number line
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
  - layering
relations:
  - target: OD-PROJECT-004
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# A dependency model names its zones and its allowed edges rather than ordering everything on one number line

## Question

`tests/contract/tests/boundaries/bands.rs` gives every workspace member one integer, and
`Test_Dependencies_Should_Run_Strictly_Downward` requires every dependency's band to be
strictly less than its dependent's. That mechanically forbids upward and circular edges,
which is worth keeping. What it also does, as a side effect nobody chose on purpose, is
force a total order onto crates that have no real precedence between them — two
architectural peers get two different numbers whenever one legitimately depends on the
other for anything at all, because the arithmetic has no way to say "these two are equals,
and one may call the other" without also claiming one sits in a lower layer of the whole
system than the other.

## What Was Measured

**This has already happened twice, mechanically, for the identical reason.**
`7e0b621a` (`P13-GATE-RUN-SEAM-CRATE`) moved `nomos-gate-orchestration` from band 40 to
band 41 "above `nomos-check-orchestration`'s band 40, so it may depend on it" — its own
commit message states the arithmetic directly as the reason for the move, not an
architectural discovery. `nomos-workflow-orchestration` repeated the pattern this session
without anyone deciding it should: it started at band 40 alongside `nomos-check-
orchestration`, `nomos-work-orchestration` and `nomos-spec-orchestration`, needed to
depend on `nomos-check-orchestration`, `nomos-correction-orchestration` and `nomos-gate-
orchestration` for its own real dispatch composition, and moved to band 41 and then band
42 across two items in the same session as each new dependency landed — three renumbers
for one crate, none of them a decision about what the crate *is*.

**The crates these renumbers concern are not actually stacked.** Measured directly against
every orchestration crate's own `Cargo.toml`: `nomos-check-orchestration`, `nomos-work-
orchestration` and `nomos-spec-orchestration` depend on none of their own band-mates.
`nomos-gate-orchestration` and `nomos-correction-orchestration` each depend on exactly
`nomos-check-orchestration`. `nomos-workflow-orchestration` depends on all three of
`nomos-check-orchestration`, `nomos-correction-orchestration` and `nomos-gate-
orchestration`. That is a small, real, acyclic graph — not a ladder five crates deep — and
the current model has no way to express "these six are peers, and three of them name
specific others" without inventing a five-number staircase to carry three real edges.

**Many bands already hold more than one crate, which is the zone shape trying to emerge
through the arithmetic.** Ten crates share band 23, nine share band 25, three share band
40. `bands.rs`'s own comments already state the real rule for those groups in prose —
"same band, so neither may name the other" — a peer relationship the number pair merely
happens to express by being equal, not by anything the ordering itself states. The system
already behaves like zones with an internal peer rule; only the edges between different
numbers pretend to be a strict stack.

## The Decision

**Numeric bands are replaced by eleven named zones, each with an explicit, declared set of
zones it may depend on.** A crate belongs to exactly one zone; two crates in the same zone
are peers by default — neither may depend on the other — unless a specific edge between
them is declared by name, the same way `tests/contract/tests/boundaries/graph.rs` already
declares `PLATFORM_ADAPTER` and `KNOWLEDGE_ADAPTER` as named exceptions rather than
deriving them from a number. This is the direct answer to "how two peers in one zone are
treated": the default is mutual exclusion, exactly as today, and an exception is a fact
someone wrote down rather than an arithmetic side effect of one crate needing to reach
another.

**The zones**, populated from the current band table and `OD-PROJECT-004`'s own
relocations:

| Zone | Crates | May depend on |
|---|---|---|
| Protocol | `nomos-contracts` | nothing but `serde` |
| Substrate | `nomos-model`, `nomos-store`, `nomos-platform`, `nomos-platform-std`, `nomos-workspace`, `nomos-scope-verification`, `nomos-capability`, `nomos-analysis` | Protocol |
| Specification | `nomos-spec-model`, `nomos-spec-store`, `nomos-spec-bundle`, `nomos-spec-ingest`, `nomos-spec-validate`, `nomos-spec-project`, `nomos-spec-orchestration` | Protocol, Substrate |
| Capability Contract | the ten `nomos-cap-*` crates | Protocol, Substrate |
| Provider | `nomos-package`, every `nomos-lang-*`, `nomos-repo-policy`, every `*-package` manifest crate | Protocol, Substrate, Capability Contract |
| Rules | `nomos-rules` | Protocol, Substrate, Capability Contract — never Provider by name, `nomos-rules`' own `Cargo.toml` already states why |
| Agent | `nomos-corrections`, `nomos-agent-contracts`, `nomos-agent-executor-claude-code`, `nomos-model-backend-ollama` | Protocol, Substrate |
| Application Service | `nomos-check-orchestration`, `nomos-gate-orchestration`, `nomos-correction-orchestration`, `nomos-workflow-orchestration` | Protocol, Substrate, Capability Contract, Provider, Rules, Agent, and the three named same-zone edges the measurement above found real: gate → check, correction → check, workflow → check, correction, gate |
| Repo Tooling | `nomos-ledger`, `nomos-work-orchestration`, `nomos-surface-provenance` | Protocol, Substrate — `OD-PROJECT-004`'s own population, carried over rather than re-derived |
| Host | `nomos-cli`, `nomos-api`, `nomos-api-transport`, `nomos-mcp` | every zone above |
| Verification | `nomos-contract-tests`, `nomos-integration-tests` | every zone above; it observes the workspace, the workspace does not observe it |

**A same-zone edge is declared per pair, not per zone.** `Application Service` is the one
zone this measurement found real internal edges in; the three above are named explicitly
rather than opened as a blanket "anything in this zone may depend on anything else in it,"
which would silently permit a fourth crate to grow an edge nobody decided on. A new
same-zone edge is a decision with the same weight as widening `PLATFORM_ADAPTER` — named,
not inferred from the crate compiling.

**Zone-to-zone edges are a small, fixed table, checked as a lookup rather than an
inequality.** `Application Service` may reach seven other zones by name; nothing about
that requires a shared number line, and nothing in it changes when a crate's role does not
change. Moving `nomos-workflow-orchestration` to depend on a fourth Application Service
sibling tomorrow would add a fourth named pair to the table above; it would not touch any
other crate's classification, which is the property `7e0b621a` and this session's
workflow-orchestration renumbers both lacked.

## What This Does Not Do

**No code moves under this record.** `bands.rs`, `README.md`'s band table, `standards.json`
and every other place the numeric table is copied stay exactly as they are; this record
states the target shape, not the migration. Building `ZONES` and `SAME_ZONE_EDGES` as real
data next to (or in place of) `BANDS`, rewriting `Test_Dependencies_Should_Run_Strictly_
Downward` as a zone-and-edge lookup, and re-deriving every crate's classification from this
table rather than from this record's own population are each a future item's own territory.

It does not audit every crate's `Cargo.toml` against the table above exhaustively. The
zone-to-zone edges are populated from `nomos-rules`' own stated Provider exclusion and from
the six orchestration crates' real dependencies, measured directly; the remaining
classifications follow the existing band clusters they are drawn from and are stated as
this record's own proposal, not as a line-by-line audit result. A future item building the
real check is where any misclassification this record made would surface and get fixed,
the same way a wrong band entry surfaces today.

It does not decide whether the eleven zones named above are the final set. `OD-PROJECT-004`
already reserved `Repo Tooling`'s population; if that record's own move happens, the
crates land where this record already put them. A zone with population one (`Protocol`) or
population four (several) is not itself a defect this record found reason to flatten
further.

## Status

Accepted. Eleven named zones replace the numeric band table's role, each with a declared
set of zones it may reach and, for the one zone measured to need it, a named list of
same-zone edges; no crate's classification moves and no test is rewritten here.
