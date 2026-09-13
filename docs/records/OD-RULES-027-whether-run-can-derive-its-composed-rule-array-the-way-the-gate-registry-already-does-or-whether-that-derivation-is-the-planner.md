---
id: OD-RULES-027
type: decision
title: Whether Run can derive its composed rule array the way the gate registry already does, or whether that derivation is the planner
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - orchestration
relations:
  - target: OD-RULES-009
    type: relates-to
  - target: OD-GATE-017
    type: relates-to
  - target: OD-GATE-023
    type: relates-to
---

# Whether Run can derive its composed rule array the way the gate registry already does, or whether that derivation is the planner

## Question

`run_context.rs` builds a hand-written array of seventy `ComposedRule` entries, and
`Composed_Rules` derives its answer from that array rather than from `nomos_rules::DESCRIPTORS`.
`P41-RUN-STOPS-NAMING-EVERY-RULE` complained about exactly this and was stranded behind
`P41-RUN-PLANNER`'s decline, so the complaint has stood unanswered rather than been refused.
`OD-GATE-023` disposed the rest of that stranded cluster by measuring each item's own claim
instead of trusting the dependency edge, and reached this one last: the edge was inherited
here too, but unlike the five gate items its claim was neither overtaken nor planner-free on
inspection, so it was left to a measurement of its own. This is that measurement.

The gate side already did the derivation being asked for: `composition::Registered` loops over
`nomos_rules::DESCRIPTORS` directly. So the question is not whether such a derivation is
possible in this workspace — it is whether `Run` can do it, given that each of its entries
closes over something different, and whether supplying that difference is the demand planner
`OD-RULES-009` has declined eight times under another name.

## What Was Measured

Censused all seventy entries by what their closure actually calls, at `3dcd84e7`:

| Shape | Entries |
|---|---|
| `Check(sources)` — reader ignored | 42 |
| `Check(sources, reader)` | 20 |
| `Check(&capabilities.<slice>, reader)` | 6 |
| `Check(reader)` — sources ignored | 2 |

The six are three over `dependency_sources` and one each over `lint_sources`,
`policy_sources` and `review_sources`.

**So the per-entry capture that looks like the obstacle is six entries, not seventy.** Sixty-two
of seventy close over the same `sources` value, and the forty-two fact-free ones differ from
the twenty fact-reading ones only in whether they name the reader they were handed.

**Three of the four shapes collapse into one signature.** A descriptor carrying
`fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>` serves shapes 1, 2 and 4 directly — a
fact-free rule takes a reader it does not read, and the two reader-only rules take a source
slice they do not read. Both are wrappers with no decision in them, and a plain `fn` pointer is
`const`-compatible, so `DESCRIPTORS` can carry one without ceasing to be a table.

**Only shape 3 needs anything else**, and what it needs is a rule-to-slice mapping of six
entries with `sources` as the default.

## The Decision

**The derivation is available, and it is not the planner.**

`Run` can loop over `DESCRIPTORS` the way `composition::Registered` already does. Two things
have to be true first, and both are small:

1. **A descriptor carries its check as a uniform `fn` pointer.** That is `nomos-rules`' own
   concern and names nothing above it — the wrappers that widen a fact-free or source-free
   check to the common signature sit beside the checks they wrap.
2. **`nomos-check-orchestration` keeps a six-entry exception mapping from `RuleId` to the
   capability slice its check reads, defaulting to `sources`.** That mapping cannot live in
   `nomos-rules`: a capability slice is an orchestration concept, and a descriptor table naming
   one would be a lower band describing an upper band's shape.

### Why the mapping is not a demand planner

This is the part `P41-RUN-PLANNER`'s decline makes worth stating rather than assuming.

A planner *computes* demand: it reads what was selected, what is already materialized and what
each rule would need, and decides at runtime what to produce. `OD-RULES-009` declines that, and
has declined it repeatedly.

A six-entry table saying "this rule reads the dependency slice" computes nothing. It is a
declared constant about a rule, fixed at the moment the rule is written, and it is precisely
the same kind of thing `OD-GATE-017` already accepted and named: "a fixed, hand-written mapping
from `RuleId` to the fact(s) it needs … composition, not choice." That record licensed a
seventy-entry version of this mapping for a different axis. A six-entry one for this axis is
smaller and no different in kind.

The distinction that keeps it honest: the mapping may not grow a *condition*. The moment an
entry reads "this slice, unless that one is already materialized," it has stopped being a
declared fact about the rule and become the planner, and `OD-RULES-009` is where that has to be
argued rather than slipped in under this record.

### What this buys, stated so the cost is comparable

Seventy hand-maintained entries become six plus a loop. A rule added to `DESCRIPTORS` today
must also be added to this array or it is declared and never runs — the exact defect
`Resolve_Rules` exists to catch after the fact. After the derivation, a rule that reads
`sources` needs no orchestration edit at all, and one that reads a slice needs one line.

## What This Record Does Not Do

It does not build it. The change reaches `nomos-rules`' descriptor table, every check's
wrapper, and `run_context.rs`'s array and `Composed_Rules`; that is a capability item's own
territory, and this record is the answer it was waiting on rather than the work.

It does not touch `OD-GATE-017`'s rule-to-fact mapping, which is a different axis — which facts
to *materialize* — and stays exactly as that record left it.

It does not reopen `OD-RULES-009`. Nothing here computes demand, and the paragraph above says
what would.

## Status

Accepted. The derivation `P41-RUN-STOPS-NAMING-EVERY-RULE` asked for is available without a
planner: sixty-two of seventy entries close over one value, three of four shapes collapse into
one `fn` signature, and the residue is a six-entry declared mapping of the kind `OD-GATE-017`
already licensed. That item is re-authored on this answer rather than left stranded behind a
decline that never addressed it.
