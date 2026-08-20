---
id: OD-RULES-007
type: decision
title: Whether heterogeneous rule populations justify a structured Rule Program/IR and global rule-compilation layer, or wait for two rules that actually need one
status: open
version: 2
authority: canonical-normative-record
tags:
  - rules
  - architecture
  - planning
relations:
  - target: OD-RULES-004
    type: relates-to
  - target: OD-RULES-005
    type: relates-to
  - target: OD-RULES-006
    type: relates-to
  - target: OD-CAPABILITY-008
    type: relates-to
  - target: OD-ANALYSIS-007
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: D-135
    type: relates-to
---

# Whether heterogeneous rule populations justify a structured Rule Program/IR and global rule-compilation layer, or wait for two rules that actually need one

## Question

An external architecture review of this workspace argued, from reading
`code-standards`/`nomos-proto`'s README and design comments, that the prototype is "almost a
miniature proof" of an architecture where a rule compiles into a typed program (loops,
branches, procedures, fixpoints, joins) and the complete selected rule set is compiled
together into one globally optimized execution plan — and proposed registering that as a
governing decision ahead of the product Gate contract.

Measured directly against `nomos-proto` rather than accepted on read: no rule compiler, rule
IR, or whole-program optimizer exists anywhere in it. It is independent Go judgment functions
plus a shared execution driver/harness (`driver.Run_File_Judgment` /
`Run_Module_Judgment`) plus self-describing declarative metadata (`checkspec.Check`,
`rulespec.Rule`, `checkspec.Inputs`). One of its own standards documents explicitly disclaims
whole-program analysis as out of scope for a single check. What the prototype does
demonstrate, confirmed against its own numbers — `kernel/checks/checkspec/inputs.go`: 31 of
277 registered checks read the whole corpus, 35 resolve per-check options, 22 read
`standards.json` limits, 24 walk the tree themselves; commit `5857241f8`: a shared driver
extracted once sixteen checks had independently duplicated the same collection/parse/
judge/coverage loop — is that check dependencies are heterogeneous and that shared execution
structure emerges once duplication is real. That is evidence for explicit dependency
declaration and demand-driven shared planning. It is not evidence for a rule program IR.

`nomos-rules` holds four rules today, `Check_Completeness_Mirrors`, `Check_Naming_
Convention`, `Check_Dependency_Direction` and `Check_Unread_Reaches_A_Finding`
(`P13-CONTROLFLOW-REACHABILITY-WIRE`). None exhibits rule-local control flow, a reusable
subcomputation, or a correction/recheck loop that a capability requirement plus the (not
yet built) analysis planner cannot already express — `Check_Dependency_Direction` and
`Check_Unread_Reaches_A_Finding` are both a plain per-source loop over an already-decoded
fact (`Payload_Of`/`Violations_In`), the same shape the other two already have. Building a
Rule Program IR now would be designing a general execution model from a population of four
rules, none of which needs it — the same shape `OD-RULES-005`, `OD-RULES-006` and
`OD-CAPABILITY-008` already declined to build ahead of, and the specific mistake `D-135`
names: inferring genericity from a wish rather than a demonstrated concrete need.

## Current Position

`ARC-ROADMAP-001` is unaffected by this question either way and is not reopened by it. Gate
consumes canonical outcomes — findings, coverage, applicability — produced by whatever a rule
already is; it does not depend on whether a rule's evidence came from a syntax capability, a
future dependency-graph capability, or eventually something needing a richer program
representation. A deep future rule can demand a new capability without changing Gate, and Gate
work already in progress (`P13-GATE-ORCHESTRATION-1`) is correctly built against
`nomos_rules::RuleRegistry` as it exists today, not against a hypothetical IR.

The corrected three-way split this record leaves standing:

1. **Needed now, and already underway or roadmapped:** explicit rule requirements (already
   real — `Syntax_Requirement()`), fact dependency tracking, applicability, coverage, and the
   shared analysis planner named on `ARC-ROADMAP-001`'s near-term tier.
2. **Allowed to emerge when a real rule demands it:** new capability families — a
   dependency-graph capability, resolved symbols, effects, and so on — each an ordinary
   analysis/capability addition, not a phase. `OD-ANALYSIS-007` already governs the general
   version of this question for program-semantics capabilities specifically.
3. **Deferred, per `ARC-ROADMAP-001`:** architecture discovery/inference, feature topology,
   path tracing, Atlas-style projections, and — this record's own subject — a Rule Program IR
   and whole-rule-set compilation.

## A Fourth Rule Arrives: Checked Against The Six Triggers

`Check_Unread_Reaches_A_Finding` (`crates/rules/nomos-rules/src/reachability.rs`,
`P13-CONTROLFLOW-REACHABILITY-CAPABILITY`/`P13-CONTROLFLOW-REACHABILITY-WIRE`) is this
record's own population growing from three to four, checked directly against the six
conditions in "What Would Decide It" below rather than assumed to still not apply:

- **A shared multi-step derived computation that cannot be captured as an ordinary fact the
  planner materializes once and several rules read.** No. Its one fact
  (`nomos.cap.controlflow.reachability`) is read once, decoded, and mapped in
  `Payload_Of`/`Violations_In` — the identical split `naming.rs` and `dependency.rs` already
  have, not a computation a planner would need to cache or reuse across rules.
- **Rule-local control flow that must be visible to the planner to schedule or cache
  correctly.** No. The judgment is a flat loop over `payload.sites`, one `Finding` per site —
  no branching, iteration, or procedure a planner-visible representation would help with.
- **A correction/recheck loop with reusable semantics across more than one rule.** No. This
  rule only raises findings, the same as the other three; nothing here corrects or rechecks.
- **A reusable subworkflow or procedure two or more rules genuinely share**, beyond the
  ordinary `Payload_Of`/`Violations_In` read-and-map shape all four rules already have in
  common. No new sharing beyond that already-generalized split.
- **A planner-visible native operation with declared effects an ordinary capability
  requirement cannot express.** No. `Payload_Of` reads one fact through `FactReader`, the
  same primitive every rule in this workspace already uses.
- **An optimization opportunity fact-demand union alone cannot obtain** — common-subexpression
  elimination, traversal fusion, batched external invocation. No. This rule's own redundancy,
  if any, is in fact acquisition, already deduplicated by the one shared `MemoryFactStore`
  `nomos-check-orchestration::run::Run` materializes into (`Materialize_Reachability` runs
  once per check, the same as `Materialize_Syntax` and `Materialize_Dependencies`), not in
  judgment structure a rule program IR would optimize.

None of the six triggers fire for this rule either. The wait for two or more real rules
demonstrating one of them remains exactly where this record already left it — the population
grew, and the conclusion did not move, checked rather than assumed.

## What Would Decide It

Two or more real rules — native, or from a future rule package — exhibiting at least one of:

- a shared multi-step derived computation that cannot be captured as an ordinary fact the
  planner materializes once and several rules read;
- rule-local control flow (branching, iteration, procedures) that must be visible to the
  planner to schedule or cache correctly, rather than being opaque inside one judgment
  function;
- a correction/recheck loop with reusable semantics across more than one rule, rather than
  each rule's correction being a one-off;
- a reusable subworkflow or procedure two or more rules genuinely share, the way
  `nomos-proto`'s sixteen duplicated driver loops did before extraction;
- a planner-visible native operation with declared effects that an ordinary capability
  requirement cannot express;
- an optimization opportunity — common-subexpression elimination across judgments, traversal
  fusion, batched external invocation — that fact-demand union alone cannot obtain, because
  the redundancy is in judgment structure rather than in fact acquisition.

Any one of these, demonstrated by real rules rather than argued from a taxonomy, is the
trigger. Until then, a Rule Program IR is speculative architecture with no case to check its
shape against, the same standing `OD-RULES-005` and `OD-CAPABILITY-008` already hold for their
own questions.

## Status

Open. This question is deliberately left open rather than resolved either way: no rule in
this workspace today — four, not three, re-measured rather than reaffirmed by count alone —
needs anything a capability requirement and the (not yet built) analysis planner cannot
already express. Revisit when two or more real rules exhibit one of the triggers above.
