---
id: OD-RULES-032
type: decision
title: Five of a sibling's 261 declared rules are answered by a Nomos rule, and a lexical matcher finds neither those five nor much else
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - conformance
  - ecosystem
  - measurement
relations:
  - target: ARC-CONFORMANCE-002
    type: relates-to
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-RULES-031
    type: relates-to
---

# Five of a sibling's 261 declared rules are answered by a Nomos rule, and a lexical matcher finds neither those five nor much else

## Question

Eighty-five rule identifiers ship in `nomos-rules`. None has ever been measured against
another repository's own declared list of what must hold, and one is on disk: the `xvpe`
checkout this build already requires declares 261 machine-readable rules under
`docs/arch/standards`. This repository's bar for building anything is a real second party —
`OD-CAPABILITY-002` licenses a bundled contract only while there is one provider on each
side, `OD-PACKAGE-011` stands open for want of a second case, and `OD-RULES-010`'s whole
reasoning is that a rule was a real second party from the day it was written. What that
second list actually says about Nomos's rule set has never been counted.

## Method

Every one of the 261 rule documents was classified from its own declared normative
statement — the first prose assertion in its body — rather than from its identifier. The
full body of both sides was read for every pair claimed below. 85 Nomos identifiers were
taken from the `RuleId` constants under `crates/rules/nomos-rules/src/checks`.

The line drawn for "inside Nomos's domain" is stated so a later reader can re-apply it
rather than re-derive it: a claim about **declared structure, naming, reachability or
dependency shape, judgeable from source plus a declared policy**. Claims about runtime
measurement, benchmark methodology, UI rendering, numeric backend accuracy, or the content
(as opposed to the path shape) of a document are outside it.

## The census

**261 rules. 5 answered, 42 unanswered and inside the domain, 214 outside.**

**The five pairs**, each read on both sides:

| xvpe rule | severity | xvpe gate | Nomos rule |
|---|---|---|---|
| `every-source-file-is-reachable` | MUST | blocking | `no-orphan-modules` |
| `engine-folder-tiers-and-dependency-direction` | MUST | blocking | `dependency-direction` |
| `composer-dependency-direction` | MUST | blocking | `dependency-direction` |
| `memory-ordering-in-security-code` | MUST | review | `atomic-ordering-choices-are-justified`, `relaxed-not-used-when-ordering-matters`, `seqcst-justified-explicitly` |
| `simulation-path-panics` | MUST NOT | review | `panics-are-justified-documented-and-validated` |

**Two of the five are declared review-enforced** — `memory-ordering-in-security-code` and
`simulation-path-panics`. That is the pairing with the most consequence in it: a `MUST` whose
own corpus records that no tool reaches it, while Nomos ships rules that do. The other three
are `blocking`, meaning xvpe already reaches them; there Nomos's rule is a second opinion
rather than a first.

**The 42 unanswered-but-inside** cluster where the reading predicts: strategy-surface naming
(`contract-trait-is-named-surface-strategy`, `generic-wrapper-type-is-named-surface`,
`strategy-type-parameter-is-named-strategy`, and the placement rule
`concrete-strategies-live-under-strategies`), declared-constant honesty
(`all-three-constants-declared`, `scope-requires-a-strength-claim`,
`notapplicable-gated-to-none-strength`, `strategies-declare-honestly`), boundary routing
(`gpu-api-boundary`, `platform-boundary-routing`, `mechanism-subsystem-boundary`,
`backend-vs-composer`, `product-hosts-name-no-backends`), third-party isolation
(`no-single-library-dependence`,
`source-outside-wrappers-does-not-import-third-party-libraries`), doc-path mirroring
(`mirrored-structure`, `crate-level-doc-path`, `file-level-doc-path`, `module-level-doc-path`,
`rename-move-mirroring`), and `cargo-target-reachability`, which is
`every-source-file-is-reachable`'s sibling for a different reachability graph and which
`no-orphan-modules` does not answer.

`benchmark-correctness-provenance` is in that 42, and `OD-RULES-031` decided a day earlier to
build the rule that answers it. It is counted unanswered here because nothing ships.

**The 214 outside** are benchmark methodology and cadence, profiling and frame budgets,
numeric-tower and backend accuracy, UI geometry and theming, E2E scene design, and the
content of xvpe's own reference and feature documentation. None of them is a claim about
source shape.

## The shared ancestor did not produce convergence

**14 of the 261 cite `(upstream: code-standards)` in their bodies** — naming the very
corpus Nomos's own rule set was ported from. 11 are `MUST` or `MUST NOT`; 10 are `review`
-gated. **Nomos answers exactly one of the 14** (`simulation-path-panics`).

That is the measurement worth keeping. Two rule sets descended from one ancestor, one of them
ported into executable rules and the other into declared prose, and a shared parent bought a
single overlap out of fourteen acknowledged inheritances. Common origin is not evidence of
coverage, and a later session proposing to port "the rest of code-standards" should read this
number first.

## A lexical matcher was tried and does not work

Re-run here rather than taken on trust: token overlap over identifier and title, stopwords
removed, Jaccard at or above 0.25. **19 candidate pairs. 2 true, 17 false. Three of the five
real pairs missed**, including the two strongest — `every-source-file-is-reachable` and
`memory-ordering-in-security-code`.

The false positives are instructive because they are not near-misses:
`trace-verification` against `certificate-verification-is-not-disabled` on the token
*verification*; `generic-parameter-ownership` against `parameter-count` on *parameter*;
`user-authored-source-is-bounded` against `nesting-depth` on *depth* and *nesting*;
`orphan-audits` — a benchmark-map audit rule — against `no-orphan-modules` on *orphan*. Five
separate xvpe verification rules all collide with `certificate-verification-is-not-disabled`
on one shared word.

The item that raised this reported 23 at its own threshold. The count moves with the
threshold and the tokenizer; the conclusion does not, and the failure is not one a threshold
fixes. `every-source-file-is-reachable` and `no-orphan-modules` share no token at all and are
the same rule. **Do not repeat this approach.** The correspondence is semantic, and 261
readings is what it costs.

## What this record does not do

It does not decide whether Nomos should read that corpus, or what relationship the two rule
sets have. `ARC-CONFORMANCE-002` decided that — the corpus is a fact Nomos reads through a
declared per-repository policy, Nomos stays out of `enforced_by` — and this record neither
anticipates nor contradicts it.

It writes no rule, changes no rule identifier, declares no capability contract, and edits
nothing in the `xvpe` tree, which this workspace does not own.

It does not claim that any of the 261 is violated, or that the 5 paired Nomos rules would
pass or fail over that tree. Nothing here was run against xvpe's source; this is a comparison
of two declared lists.

It does not treat the 42 as a work queue. They are what the census found inside the domain,
not a decision that any of them should be built, and each would need its own population
measured the way `OD-RULES-031` measured one.

## Status

Accepted. 5 answered, 42 unanswered inside the domain, 214 outside; 2 of the 5 pairs sit on
rules xvpe declares nobody enforces; 14 rules cite the shared ancestor and Nomos answers one
of them; and the lexical shortcut is recorded as tried and refused. No code moves here.
