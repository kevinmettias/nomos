---
id: OD-RULES-031
type: decision
title: A bench-to-oracle rule is built, and what a bench owes is declared by the repository rather than inferred from a filename
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - benchmarks
  - policy
  - ecosystem
relations:
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-026
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# A bench-to-oracle rule is built, and what a bench owes is declared by the repository rather than inferred from a filename

## Question

XVPE's own testing standard states a Benchmark Correctness Rule: every benchmark must
exercise behaviour covered by a correctness test or an equivalent pre-benchmark oracle, and
results from an implementation that does not pass its oracle are invalid and must not
participate in performance comparison or default selection. Nomos is the layer that would
judge such a rule and has none of the kind. Whether one is built — and if so, what decides
which benches owe an oracle, what the subject is, what is reported when the oracle cannot be
seen, and how any of it is verified from a workspace with no benchmarks at all — is the
question this record answers.

## What was measured

Counted directly rather than taken from the item that raised this.

**This workspace has no benchmarks, and no rule mentioning one.** Zero files match
`*/benches/*.rs` anywhere under this repository. Across every rule module under
`crates/rules/nomos-rules/src/checks`, the word *bench* occurs exactly once, and it is a
fixture string inside `rust_text.rs` — a comment in a test case, not a subject.

**The sibling checkout has 266 of them.** Counting a benchmark target the way a cargo
workspace defines one — a `benches/` directory sitting beside a `Cargo.toml` — `F:/repos/xvpe`
holds 84 such crates and 266 bench `.rs` files in them. Of those 266, **112 have no
same-stem `.rs` file under `tests/` or `src/` in the same crate.**

**That 112 is a count of candidates and emphatically not a count of findings**, which is the
measurement that decides this record's shape rather than merely motivating it. The standard's
own words are "exercises behaviour covered by a correctness test": a semantic correspondence.
Same-stem is a filename correspondence, and the two are different claims. A bench named
`throughput_wait_scaling_wakeup.rs` may be covered in full by a test named nothing like it,
and a rule reporting 112 violations on that basis would be reporting its own heuristic back
to itself. Nothing measured here establishes that any of the 112 is a real violation.

**The population is reachable from here, and CI cannot reach it.**
`crates/languages/nomos-lang-rust/tests/corpus/claims.rs` already parses that checkout —
`DEFAULT_ROOT` is the literal `F:/repos/xvpe`, so the corpus is found on a developer machine
whether or not `NOMOS_RUST_CORPUS` is set — and it already parses the bench files
specifically: its own comment names `xvpe-thread-pool`'s `benches/fiber/throughput_*.rs`.
So there is a real mechanism carrying these 266 files into this workspace's reach, and it is
one of the three corpora no CI runner has.

**The resolver this needs already exists at one scale over.** `checks/mirror.rs` reads a name
declared at a site, resolves it against the real parsed source set, and reports a phantom when
it resolves to nothing. Its own module doc records the property that matters most here:
resolution is cross-file, and a run over a truncated source set resolves a real check to
nothing and reports a blocking finding against a declaration that is in fact satisfied.

**`OD-RULES-026` deferred a different question and does not answer this one.** That record
deferred *benchmark history* — a performance requirement judged against past results — for
want of any benchmarking convention or results store, and said a benchmark capability
"becomes reachable the moment this workspace adopts a benchmarking convention for its own
purposes." A bench-to-oracle correspondence needs no history, no results format and no
runtime evidence: it is a question about static structure, answerable from the same syntax
facts every other rule here reads. It is not blocked by that deferral and this record does
not reopen it.

## Decision

**The rule is built.** A standard stating that every benchmark owes a correctness oracle,
with 266 real subjects one checkout away and nobody enforcing it, is the exact condition
this workspace exists to remove. `OD-ROADMAP-001` retired the population-of-zero caution, so
having none here is not a reason; and a decline would have had to rest on the 112 candidates
being uninteresting, which nothing measured supports either.

Four things are fixed here, because each is a place the rule could be built wrong in a way no
test would catch.

**1. What a bench owes is declared by the repository, never assumed by the rule.** The
correspondence convention is a fact the repository under check declares, in the shape
`OD-RULES-011` decided for a rule's parameters: a policy read, resolved per repository, with
no convention compiled into the rule. A repository that declares none gets `NotApplicable`
and no findings — not a pass, and not 266 of them.

Two conventions are named as the closed set this rule admits, and a third is refused:

- **declared-at-the-site** — the bench names its oracle, and the rule resolves that name
  against the real parsed source the way `mirror.rs` already resolves a declared mirror. This
  is the only one that expresses what the standard actually says, because only the author
  knows which test covers the behaviour being measured.
- **same-stem** — the bench's own stem must resolve to a file under the repository's declared
  test roots. A weaker claim, and admitted only because it is the one a repository with 266
  unannotated benches can adopt *today* to get a first, honest signal; a repository choosing
  it is choosing a proxy and the finding text must say so at the finding, not only here.
- **a directory-name assumption compiled into the rule is refused.** `benches/` beside
  `tests/` is a cargo convention, not a universal one, and a rule that hardcodes it is a rule
  that silently means nothing in every repository shaped differently — the defect
  `OD-RULES-011` was written to end.

**2. The subject is the bench target file, addressed by its path.** Not the oracle, and not
the pair. The oracle may live in another file, another directory, or nowhere; keying the
subject on it would make a finding's identity depend on the answer being computed, so a bench
that gained an oracle would report under a different subject than the one that reported it
missing, and no baseline or waiver could follow it across the fix. Every other file-scoped
rule here keys on the path and this one does the same.

**3. Absence is neither a pass nor a failure, and there are three different absences.**
`OD-RULES-001` is the authority; what it forces here is that the three are told apart rather
than collapsed:

- *the repository declared no benchmark policy* — `NotApplicable`. The rule has no claim to
  make, and reporting one would be inventing a convention on the repository's behalf.
- *the policy is declared and this bench does not satisfy it* — a finding, and a real one.
  Under declared-at-the-site that is a bench declaring no oracle or declaring one that
  resolves to nothing; under same-stem it is a stem resolving to nothing.
- *the bench's own syntax fact could not be read* — the `Unread_As_This_Rule` shape every rule
  in this crate already carries: an advisory saying this file could not be judged, which is
  the one answer that must never render as clean.

**4. It is verified in two halves, and the second is corpus-gated and must be registered as
such.** The rule's own judgment is tested the way every rule here is tested: hand-built
payloads, both conventions, all three absences, no corpus required, running in CI. The claim
that it says anything *true about real benchmarks* can only be tested against the 266, and
that assertion is corpus-gated and must be counted in
`tests/contract/tests/corpus_gates.rs` so the size of the hole is declared rather than
discovered. A green CI run is not evidence this rule was exercised against a real benchmark,
and the increment that builds it does not get to claim otherwise.

## What this record does not do

It does not write the rule, register it, compose it into any run, or add a policy family to
any provider. A follow-up increment does that against these four constraints; this record
names the shape, as `OD-CAPABILITY-014` named one before the increment that built it.

It does not decide which convention *this* repository declares, because this repository has
no benchmarks to declare one for. The question becomes live the day it gains its first, and
`OD-RULES-026`'s own note — that a benchmark capability becomes reachable once a benchmarking
convention is adopted here — is where that lands.

It does not decide anything about benchmark results, history, regression thresholds or
performance comparison. `OD-RULES-026` holds all of that, deferred, and nothing here disturbs
it. The standard's second sentence — that results from an implementation failing its oracle
must not participate in comparison or default selection — is a claim about a *results
pipeline* this workspace does not have, and is out of scope for a rule that judges source.

It does not assert that any of the 112 same-stem candidates is a violation. That number is an
upper bound on candidates under the weaker of the two admitted conventions, and this record
is explicit that a finding count derived from it would be the rule reporting its own heuristic
back to itself.

## Status

Accepted. The rule is built, its convention is declared by the repository under check, its
subject is the bench file, its three absences are distinguished, and its real-population
verification is corpus-gated and counted. No code moves here.
