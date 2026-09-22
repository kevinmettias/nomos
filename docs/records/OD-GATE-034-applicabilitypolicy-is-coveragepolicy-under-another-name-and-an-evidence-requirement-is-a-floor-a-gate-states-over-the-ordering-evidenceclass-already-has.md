---
id: OD-GATE-034
type: decision
title: ApplicabilityPolicy is CoveragePolicy under another name, and an evidence requirement is a floor a gate states over the ordering EvidenceClass already has
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - coverage
  - applicability
  - evidence
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-ROADMAP-003
    type: relates-to
  - target: OD-GATE-016
    type: relates-to
  - target: OD-GATE-015
    type: relates-to
  - target: OD-GATE-014
    type: relates-to
  - target: OD-GATE-017
    type: relates-to
  - target: OD-GATE-011
    type: relates-to
  - target: OD-GATE-023
    type: relates-to
  - target: OD-GATE-029
    type: relates-to
  - target: OD-GATE-030
    type: relates-to
  - target: OD-GATE-031
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-ANALYSIS-012
    type: relates-to
  - target: OD-COMPLETENESS-004
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
---

# ApplicabilityPolicy is CoveragePolicy under another name, and an evidence requirement is a floor a gate states over the ordering EvidenceClass already has

## Question

`ARC-ROADMAP-001` constraint 5 lists what an end-user repository would configure on the
product Gate: `ScopeSelector`, `RuleSelector`, `ApplicabilityPolicy`, `CoveragePolicy`,
`BaselinePolicy`, `SuppressionPolicy`, required phases, evidence requirements, failure
disposition. Seven of those nine have a real type in `nomos-gate-orchestration` now, each built
under a record that measured what the corpus asked for first (`OD-GATE-014`, `OD-GATE-015`,
`OD-GATE-016`, and the phase increments). The crate's own module doc names the two that do not:
`ApplicabilityPolicy`, "which the `ARC-ROADMAP-001` quotation names and nothing in this
workspace defines," and evidence requirements, "which `nomos_contracts::Finding`'s own
`EvidenceClass` classifies but no gate policy reads." An external review at `bc0aaacf`
repeated both as gate gaps.

Two named gaps are not two pieces of work until something says what each name means.
`OD-GATE-016` built `CoveragePolicy` for `WF-001`'s "unsupported-analysis policy" and listed
`ApplicabilityPolicy` beside it in the same quotation without saying what the second would add
to the first. `EvidenceClass` has an ordering, a `Weaker_Of` and an `Is_Mechanical`, and
`P12-PROVENANCE-FLOOR` — an item asking that a consumer be able to state a floor over that
ordering — was declined for reserving two record files against a `done_when` that needed Rust,
not on the merits. Building either name without deciding what it means would mint either a
second authority over coverage, which `OD-GATE-011` files defects about, or a policy the corpus
never asked for.

It is answerable now rather than paused because `OD-ROADMAP-003` records that each of
`OD-ROADMAP-002`'s three pauses has lapsed on its own stated condition, and it is answerable
only in one shape, because that record also states the constraint that survives the lapse.

So: what does the corpus require of a gate about applicability and about evidence, by
requirement identifier? Is `ApplicabilityPolicy` `CoveragePolicy` under another name, or a
distinct policy over per-rule `Applicability` outcomes? What is an evidence requirement policy
— may a gate state the lowest `EvidenceClass` a finding must carry to block, to count toward a
threshold, or to be reported, and what does a finding below that floor become? And which
capability item builds whichever of the two is real?

## What Was Measured

Re-measured 2026-09-21 at `ee4d6111`, against the v14 corpus's requirement files (363 files
under `01_authoring/artifacts/requirements`, one per requirement, each carrying a `statement:`
field in its front matter and a `## Statement` section that repeats it) and against this
workspace's sources at that revision. Every statement was read in full rather than by its first
line — a statement wraps, and a first-line pass misses `CHK-003`'s "agent-required" and
`BASELINE-003`'s "unparseable".

An earlier pass at `d32ed755` reached the same two decisions. This one corrects it in three
places, all of which are the same mistake in different clothes — a population taken from one
spelling of a question: the applicability table omitted `ARCH-ENGINE-005` and
`ARCH-ENGINE-006`, both of which name a gate outright; `Applicability`'s three predicates were
described as partitioning its eleven states, which the test cited for it explicitly denies; and
three source counts were taken from a grep that missed a qualified spelling of the same field.

### Neither name is in the corpus, and neither is the phrase

Counted over all 363 statements: `ApplicabilityPolicy` in none, `CoveragePolicy` in none,
"evidence requirement" as a phrase in none, "floor" in none. `EvidenceRequirement` as one word
occurs in exactly two, `EGRAPH-003` and `EGRAPH-008`. "minimum evidence sufficiency" occurs in
exactly one, `RUNTIME-007`. The populations the two tables below are drawn from, by substring:
"unsupported" 9 statements, "applicab" 45, "coverage" 25, "provenance" 21, "evidence" 89. Both
names under question are therefore this workspace's own vocabulary, written in
`ARC-ROADMAP-001`'s own list, and not text the corpus obliges anyone to honour.

### What the corpus requires of a gate about applicability

| Identifier | What it requires of applicability |
|---|---|
| `WF-001` | "A gate shall define policy: required phases, thresholds, coverage, unsupported-analysis policy, waivers, approvals, and blocking behavior." The one clause granting a gate a policy here at all. |
| `ARCH-ENGINE-005` | An unanalyzed file, target, build variant or capability region "is coverage debt, not a clean result", and "no run, gate, report, API response, or agent context may collapse unsupported, unavailable, unparseable, or failed analysis into pass." |
| `ARCH-ENGINE-006` | A gate, phase, workflow or agent-preparation operation that "evaluates no applicable rules or has no effective providers shall fail or report an explicit no-coverage state." |
| `ADOPT-CONFIG-004` | A run that evaluates no applicable rules or treats unanalyzed coverage as clean shall not report success. |
| `CHK-003` | Every run reports evaluated, excluded, unsupported, unavailable, failed, not-applicable and agent-required combinations. |
| `BASELINE-003` | Scoped gating preserves suppressed, waived, unsupported, unavailable, unparseable, excluded, not-applicable and evaluated states distinctly; "nothing disappears merely because it is outside the blocking scope." |
| `ARCH-ENGINE-003` | "Silence, NotApplicable, MissingCapability, ProviderUnavailable, Unparseable, and AnalysisFailed are distinct states, not interchangeable gaps." |
| `RUNTIME-008` | Dropped samples, incompatible hardware, unstable variance and baseline mismatch produce "explicit evidence limitations or non-pass applicability states rather than a clean result." |
| `CHK-002` | Rules are selected by gate, phase, family, language, severity, provider or package, and "the exact cross-product" runs "after applicability preview." |
| `PKG-002` | Effective rule applicability is *derived* — from the rule's canonical capability requirements, target language features, verified provider capabilities, installed tools, repository configuration, target context and compatibility-test results; "no manually duplicated language-rule support list shall be authoritative." |
| `PKG-004` | Every applicability result exposes rule/version, target, required capabilities, selected providers, guarantees and fallbacks, configuration inputs, verification status, resulting state and explanation; "a matrix is a projection of these records, not the source of truth." |
| `PKG-006` | Provider resolution prefers the strongest compatible guarantee and labels fallback, approximation, partial coverage, unavailable tooling and analysis failure distinctly. |
| `PKG-012` | A RulePackage's normative contract defines "applicability semantics, severity defaults, accepted exceptions, evidence obligations, and suppression semantics" independently of any provider. |
| `PKG-015` | The applicability states, by name, "at minimum"; "unsupported or failed analysis shall never be represented as a pass." |
| `PKG-016`, `PKG-017`, `PKG-018` | One derived compatibility graph, queryable equivalently from either direction; resolution keys include "policy configuration." |
| `PKG-027` | The canonical states are the protocol truth and the human-readable labels are standardized projections of them. |
| `CAL-003`, `CAL-009`, `AGT-016` | Knowledge outputs, contract monitoring and knowledge-retrieval failures shall not modify applicability, severity, thresholds, suppressions or gate disposition. |
| `CONF-003` | RulePackage conformance suites verify applicability. |

All 45 statements in the "applicab" population were read, and all 9 in the "unsupported" one.
Those not tabled above are about resolution, projection, invalidation, versioning or agent
preparation — `PKG-003`, `PKG-021`, `PKG-023`, `PKG-025`, `PKG-029`, `COR-EXEC-007`, `AGT-001`,
`AGT-007`, `AGT-018`, `RUNTIME-006` and the rest — and not one of them asks a gate to hold a
policy over applicability outcomes.

Read together, the corpus has two things and not three. It has *applicability resolution* — an
engine deriving a state per rule and target, `PKG-002` through `PKG-029` — and it has one
gate-side policy clause about what a gate does with the states that come out, `WF-001`'s
"unsupported-analysis policy". Around that one clause sits a family of constraints that are not
policies at all but prohibitions on collapsing: `ARCH-ENGINE-003`, `ARCH-ENGINE-005`,
`ARCH-ENGINE-006`, `ADOPT-CONFIG-004`, `CHK-003`, `BASELINE-003` and `RUNTIME-008` between them
forbid a gate to report an unreached judgment as a pass, to lose a state, or to succeed having
judged nothing. Nothing asks a gate for a second policy *over* applicability outcomes beyond
what to do when a rule could not look.

### What the corpus requires of a gate about evidence

| Identifier | What it requires of evidence |
|---|---|
| `EVID-001` | Evidence is classified as Authoritative, Verified, Observed, Derived, Approximate, Predicted, AgentJudged or HumanAsserted, "with provenance and intended use," and "classification shall not be reduced to one universal confidence number." |
| `EVID-002` | Conflicting evidence stays queryable; resolution policies *may* prefer compiler semantics over syntax approximation, runtime observation over static possibility, and declared architecture over inference only where policy marks the declaration authoritative. |
| `AGT-EXEC-004` | Agent claims of tests, builds, checks or measurements "require attached tool evidence from the corresponding trusted capability. Unsupported claims remain AgentJudged." |
| `RUNTIME-007` | Enforcement roles are Observational, Advisory, ReviewRequired or Blocking; "blocking runtime policy shall require compatible baselines, minimum evidence sufficiency, configured loss limits, and explicit repository or organization authorization." |
| `RUNTIME-006` | A runtime applicability and result record identifies, among much else, "permitted evidence loss" and its enforcement role. |
| `CAL-006` | "Canary results shall not silently affect blocking policy." |
| `PKG-026` | Rule quality objectives are scoped records "rather than unqualified global thresholds," each naming its metric, population, scope, threshold direction and value, enforcement role and evidence period. |
| `FIND-012` | `FieldGeometry` classifies its semantic origin, and propagated and predicted values "shall never be rendered as direct violations." |
| `PLACE-002` | `InsufficientEvidence` is a disposition of its own, and clients "shall not collapse InsufficientEvidence into agreement." |
| `PKG-012` | Evidence obligations are part of a RulePackage's normative contract. |
| `CHK-004`, `FIND-001`, `FIND-002`, `FIND-003`, `CAL-002`, `AGT-008` | A finding preserves its exact evidence, its evidence fingerprint and its provider provenance; deduplication keeps multiple providers visible; `EvidenceChanged` is a finding transition; exports and contextualization preserve evidence classification beside applicability and gate disposition. |
| `EGRAPH-003` | `EvidenceRequirement` identifies "the required evidence type, governing obligation, subject, provider guarantee, scenario or workload, variant, completeness threshold, freshness conditions, and acceptance rule." |
| `EGRAPH-005`, `EGRAPH-007`, `EGRAPH-008` | `VerificationResult` states include `Insufficient` and `AgentOnly`; the service answers "which claims remain agent-judged"; completion and attestation are computed from explicit required claims and `EvidenceRequirement`s — "absence of a finding, one passing test, or one observed successful run shall not by itself establish completeness." |

Read together: the corpus supplies the vocabulary (`EVID-001`), forbids collapsing it to a
number (`EVID-001`) or presenting the weak end of it as a mechanical result (`AGT-EXEC-004`,
`FIND-012`, `PLACE-002`), requires a *blocking* policy to state minimum evidence sufficiency in
the one family that reaches enforcement roles (`RUNTIME-007`, with `RUNTIME-006`'s permitted
evidence loss beside it), insists that what affects blocking policy be stated rather than
inferred (`CAL-006`), and names `EvidenceRequirement` only as the evidence graph's completion
object (`EGRAPH-003`, `EGRAPH-008`) — the graph `ARC-ROADMAP-001` constraint 3 keeps as a
kernel constraint rather than a kernel construction. No requirement says in so many words that
a gate shall state a minimum `EvidenceClass`. `RUNTIME-007` says it of a blocking runtime
policy, and `WF-001`'s "blocking behavior" is the clause under which a gate says the same of
itself.

### What the workspace has

**Per-finding applicability already reaches disposition, twice.** `Finding::Can_Fail_A_Build`
(`crates/contracts/nomos-contracts/src/reporting/finding.rs`) is
`self.gate.Can_Fail_A_Build() && self.applicability.Is_Evaluated()`, and
`Test_A_Finding_From_A_Rule_That_Never_Ran_Should_Not_Fail_A_Build` pins that every one of the
five coverage-debt states fails the second condition. `Partitioned_Findings`
(`crates/orchestration/nomos-gate-orchestration/src/gate_environment/reduction.rs`) filters on
that predicate first, before calibration, suppression or baseline sees a finding. Per-run,
`Claim_Of` (`crates/orchestration/nomos-check-orchestration/src/examined/claim.rs`) reports
`Incomplete` when any selected finding is coverage debt or agent-required, and `CoveragePolicy`
(`crates/orchestration/nomos-gate-orchestration/src/policy/coverage_policy.rs`, `Unset` or
`RequireCompleteness`) turns that into `Indeterminate` under `NoVerdict::IncompleteCoverage`.

**And the states neither path names are decisions rather than gaps.** `Applicability`'s three
predicates do not partition its eleven states, and
`Test_No_State_Should_Answer_Two_Predicates` is a disjointness check whose own doc says so:
`NotApplicable` and `ConfigurationDisabled` answer none of the three. That is not a hole a
policy could fill. Neither state is `Is_Evaluated`, so a finding carrying it cannot block;
neither is coverage debt or agent-required, so neither makes a run's claim incomplete. A rule
that does not bind a subject and a rule a repository switched off are choices somebody made,
which is what `OD-GATE-014` and `OD-GATE-017` own by selection, and `OD-ANALYSIS-012` is the
same discipline one layer down: an empty population is reported apart from a clean one rather
than collapsed into it. All eleven states are already classified by both paths.

**Every widening a per-rule applicability policy could be has already been assigned.**
`OD-GATE-016`'s "What This Does Not Do" leaves "a minimum-`Applicability` threshold, a per-rule
or per-scope coverage requirement" to `CoveragePolicy`'s own later widening, and `OD-GATE-029`
repeats the assignment in its own words — "a minimum-`Applicability` threshold or a per-rule
coverage requirement is still what `OD-GATE-016` left to real evidence" — while deciding a
third variant for coverage itself. That variant is decided and
not yet built: `CoveragePolicy` is two variants at this revision, in the crate and in
`tests/contract/surface/nomos-gate-orchestration.txt`. `OD-GATE-014` and `OD-GATE-017` own
which rules bind which subjects by policy — `ScopeSelector` and `RuleSelector` — whose outcome
is a finding that does not exist, or `ConfigurationDisabled`. Plan-time resolution, the
`PKG-002` engine's shape, is `P41-APPLICABILITY-IN-THE-PLAN`, declined under `OD-GATE-023`
because there is no version of it that does not need the planner `OD-RULES-009` declines.

**`EvidenceClass` is ordered, and the mechanical boundary is a point on that order.**
`crates/contracts/nomos-contracts/src/reporting/finding/evidence_class.rs` derives `Ord` over
the declaration order, weakest first: `AgentJudged`, `HumanAsserted`, `Predicted`,
`Approximate`, `Derived`, `Observed`, `Verified`, `Authoritative`. `Weaker_Of` is `min`, and
`Test_Weaker_Of_Should_Never_Exceed_The_Weaker_Input` and `Test_Agent_Judged_Should_Be_The_Floor`
pin that. `Is_Mechanical` is true for `Approximate` and every class above it —
`Test_Is_Mechanical_Should_Be_True_For_Measured_And_Derived_Classes` — so "mechanical" is
exactly "at or above `Approximate`" under the same ordering, not a second axis.

**Every finding carries a class, and no gate policy reads it.** `Finding::evidence` is an
`EvidenceClass`, not an `Option`. `nomos-gate-orchestration` does not read the field anywhere,
its own tests included: a search of the crate for `.evidence` returns nothing.
`Finding::Is_Mechanical` delegates to the class and has exactly one caller, a test in
`crates/rules/nomos-rules/src/checks/mirror/tests/judgments.rs`; `nomos_model::Evidence::Is_Mechanical`
delegates the same way and its only caller is its own test. A `Finding` at
`GateCategory::Blocking`, `Applicability::Supported` and `EvidenceClass::AgentJudged` fails a
build today, and no configuration can say otherwise.

**Nothing below `Derived` reaches a gate at this revision.** In `crates/rules/`, the `evidence:`
field is written `Derived` at 59 sites, `Verified` at 4 and `Observed` at 1 — counting both
spellings, since two of the `Derived` sites are written
`nomos_contracts::EvidenceClass::Derived` in `crates/rules/nomos-rules/src/checks.rs` and a
single-spelling grep misses them. The one `Observed` is
`crates/rules/nomos-rules/src/checks/review.rs`, the review-finding rule, at
`GateCategory::Advisory`. Workspace-wide, seven `evidence:` sites are written below `Derived`
and five of them sit inside a `#[cfg(test)]` module. The two that do not are a `WorkflowStep`
at `AgentJudged` in `crates/host/nomos-cli/src/workflow/composition.rs` and a
`MaterializedFact` at `Approximate` in
`crates/languages/nomos-lang-rust-scan/src/fact_context.rs` — a declared step and a fact,
neither of them a `Finding`. The only `Finding` below `Derived` anywhere in the workspace is a
fixture at `HumanAsserted` in `crates/kernel/nomos-model/src/identity/finding_occurrence_id.rs`,
written to prove that a finding's identity ignores its evidence class. `OD-RULES-010`'s chain —
a provider's fact at `Verified`, the rule's finding at `Derived` — is why every tool-backed
finding sits at `Derived`. `GateCommand::model`'s own doc says it is read by nothing yet and
gives the reason: no registered rule yields `Applicability::AgentRequired` today, so the
producer of the first `AgentJudged` finding is the executor for the subjects
`OD-CONTRACTS-002` named that state for.

**A new `GateCommand` field costs no caller an edit.** `GateCommand` derives `Default`, 26 files
construct one, and all twelve of them that are not test modules do so through struct update
(`..GateCommand::default()` or `..Default::default()`);
`crates/host/nomos-cli/src/gate/parsing.rs` is the one that spells `coverage:` at all, and it
spells `CoveragePolicy::default()`. `OD-GATE-029`'s own count — 49 constructions across 18
files, four of them stating `RequireCompleteness` and all four tests — counts construction
expressions rather than files and was taken at that record's own revision, so it is not this
number disagreeing with itself.

**A bucket is the never-hide mechanism, and it has three readers.** `GateFindings`
(`crates/orchestration/nomos-gate-orchestration/src/gate_plan/gate_findings.rs`) carries
`blocking_findings`, `calibrated_findings`, `suppressed_findings`, `baselined_findings` and
`baseline_exceeded_findings`. The three whose findings could not block are documented as
"carried rather than dropped" — calibrated and suppressed in those words, baselined by
reference to them — and the fifth blocks and is still not `blocking_findings`, because
`OD-GATE-030` refuses attribution inside an exceeded population. `FindingDisposition` (`gate_compare/finding_disposition.rs`) has one variant per
bucket, and its doc says why that correspondence is load-bearing: `Population_Of` walks the
list, so a bucket with no variant is invisible to `compare` and reads as *removed*.
`Explanation::Found` (`finding_query/explanation.rs`) names `calibrated_by`, `suppressed_by` and
`baselined_by`. `crates/host/nomos-cli/src/gate/report/run.rs` counts the buckets on one line.
`Policy_Digest` (`gate_environment/provenance.rs`) hashes the suppression, baseline and
calibration entries and a tag for `coverage`, and is the identity `OD-GATE-031` gives a run's
resolved policy — so a policy a run judged under that the digest does not cover is a comparison
attributing a policy change to the repository.

**Phase counting is downstream of blocking, already.** `Phased_Outcome`
(`gate_environment.rs`) calls `Evaluated_Phases` with `reduced.findings.blocking_findings`, and
`Judged_Phase` (`crates/orchestration/nomos-gate-orchestration/src/gate_phase.rs`) narrows that
list by each phase's own rules before applying the phase's threshold. Nothing a phase counts
was not already blocking.

**A new gate policy family must be a declared constant.** `OD-ROADMAP-003`, having recorded the
three pauses as lapsed, states what survives: a new gate policy family "is a declared constant
— a `RequiredFact` a rule's descriptor names and `Demanded_Families` reads, a `Provider_Offer`
the composition root registers, a field `Resolve_Gate_Policy` reads off `nomos-gate.json` — and
never a condition that consults store state, cost or prior materialization." `Resolve_Gate_Policy`
and `GatePolicyFile::Resolved_Over` are the reader and the resolution rule that already carry
`coverage`, and its tests pin the rule: a field left at its default in the command takes the
file's value, and a field the command states wins over a file that is silent.

**`P12-PROVENANCE-FLOOR`'s four clauses, against a gate.** Its second — "a consumer can state an
evidence floor, and the comparison uses EvidenceClass's existing ordering rather than a second
notion of strength" — is the one a gate can discharge. Its first and fourth are about a claim
whose provenance is *absent*, which a `Finding` cannot be, since the field is not optional. Its
third, self-referential provenance, is the `P10-STAMP-CONSISTENCY` projection shape and is not a
question a gate reduces. The decline reason itself says where its check belongs — beside
`mirror.rs` in `nomos-rules`, with a fixture for the self-referential case — and that is a
different home from this one.

## The Decision

### 1. `ApplicabilityPolicy` is `CoveragePolicy` under another name, and `CoveragePolicy` is the name that governs

Every reading the name admits already has an owner, measured above, and none of them is a
fifth policy type:

- *What a gate does when a rule could not look* is `WF-001`'s "unsupported-analysis policy."
  `OD-GATE-016` built it as `CoveragePolicy`, `OD-GATE-029` decided its third variant, and both
  records assign its every further widening — per rule, per scope, a minimum applicability — to
  the same type.
- *Which rules bind which subjects by policy* is `ScopeSelector` and `RuleSelector` under
  `OD-GATE-014` and `OD-GATE-017`, and its outcome is a deliberate absence the coverage path
  already excludes from debt.
- *Deriving the state itself* is `PKG-002` through `PKG-029`: the `Applicability` type,
  capability resolution, and the declined plan-time form. It is not a gate policy and never
  was one; a gate consumes its output.

The never-collapse family — `ARCH-ENGINE-003`, `ARCH-ENGINE-005`, `ARCH-ENGINE-006`,
`ADOPT-CONFIG-004`, `CHK-003`, `BASELINE-003`, `RUNTIME-008` — is not a fourth reading. Each of
those is a prohibition on reporting, discharged by `Finding::Can_Fail_A_Build`, by `Claim_Of`
and by the buckets, and `OD-COMPLETENESS-004` already settled the report surface that answers
them. A prohibition is not a policy: there is nothing for a repository to configure in "shall
not report success."

Per-finding outcomes reach disposition through `Finding::Can_Fail_A_Build`; per-run outcomes
through `Claim_Of` and `CoveragePolicy`. A type named `ApplicabilityPolicy` would have to decide
one of those three things a second time, in a second place, and that is the defect class
`OD-GATE-011` names: two artifacts each independently readable as the authoritative answer to
the same question. The name is retired as a type. Where a reader meets it, this record is what
it resolves to.

**`ARC-ROADMAP-001`'s list is left as it is.** Three reasons, none of them convenience. The list
is a quotation of concerns an end-user repository would configure, in a boundary record whose
own "What This Record Does Not Do" says it "does not claim the 363-requirement corpus has been
reconciled" and is "a structural survey by family and representative title, not a
per-requirement audit"; two of its nine entries, "required phases" and "evidence requirements,"
were never type names either, and the concern this one stands for is real — the gate does hold a
policy over applicability, spelled `CoveragePolicy`. Deleting one word from that sentence would
edit a published record that `OD-GATE-016` and `nomos-gate-orchestration/src/lib.rs` both quote
verbatim, leaving two stale quotations of a corrected source and a reader of either with no
route to the resolution. And the route is what resolves a name, not the deletion: this record
relates to `ARC-ROADMAP-001`, and the capability item below retires the crate doc's admission by
citing this record where the admission stands. A later amendment of `ARC-ROADMAP-001` for its
own reasons may carry the correction; nothing here requires one.

### 2. An evidence requirement is a floor a gate states over `EvidenceClass`'s own ordering

**A gate may declare the lowest `EvidenceClass` a finding must carry to block.** The comparison
is `finding.evidence >= floor` under the `Ord` `evidence_class.rs` already derives, weakest
first. There is no second notion of strength: a gate that wants only mechanical findings to
block states `Approximate`, because `Is_Mechanical` is that point on the same order and nothing
else. `EVID-001` forbids reducing the classification to one number, and a floor does not — it
names a class, and the eight classes stay distinct on both sides of it.

**It is a declared constant, and the constraint `OD-ROADMAP-003` states is why.** The floor is a
key in `nomos-gate.json`, read by `Resolve_Gate_Policy` and resolved over the command by the
rule `GatePolicyFile::Resolved_Over` already applies to `coverage`. It may not become a
condition: not "require `Verified` when the store already holds a stronger fact," not "lower the
floor when materializing the evidence would cost more," not "apply the floor when the last run
failed." Each of those reads an input the declaration does not have, and the moment one is
written the field has stopped being a declared fact and become the planner `OD-RULES-009`
declines.

**Counting toward a phase threshold follows blocking, by construction.** `Judged_Phase` counts a
phase's share of `blocking_findings` — the list calibration, suppression and baseline have
already reduced to still-blocking — so a finding under the floor is out of every phase's count
the moment it is out of the blocking bucket. A second floor for thresholds is refused: it would
let one finding be blocking to a phase and not to the run, which is the two-encodings shape
again, one layer down.

**There is no floor for reporting, and a finding below the floor is never dropped.** `CHK-003`,
`BASELINE-003` and `ARCH-ENGINE-005` require the states a run reached to stay distinct and
visible, and every policy this crate holds already keeps a finding it could not let block in a
named bucket. A finding below the floor goes to a bucket of its own in `GateFindings`, disjoint
from the five that exist, with a `FindingDisposition` variant so `compare` reports a finding that
moved there when a floor was raised rather than reading it as removed, an `Explanation::Found`
field naming the floor it fell under, and a count in the CLI's own summary line. It is **not**
re-labelled `Advisory`: `Finding::gate` is the rule's wiring truth and a policy does not rewrite
what a rule declared. It is **not** filed as calibrated, suppressed or baselined: those are
dispositions a person authored, matched by rule or by `rule`/`subject` identity and carrying a
rationale, and this is a mechanical statement about a class of evidence with no per-finding
author. A reader who cannot tell "this rule is advisory" from "this finding's evidence was too
weak under this gate" has lost exactly the distinction the bucket exists to keep, and
`PLACE-002`'s refusal to let `InsufficientEvidence` collapse into agreement is the corpus saying
the same thing about its own nearest state.

**The floor is consulted where `Can_Fail_A_Build` is, as a third condition of the same kind.**
`OD-GATE-015` described that predicate as "two conditions, with no suppression hook between them
and no third condition a baseline or waiver could occupy," and a suppression rightly went
elsewhere: it is a human disposition about a specific finding. The floor is not. It is a fact
about the finding itself — its gate category, its applicability, and now its evidence class — so
`Partitioned_Findings` reads it beside the other two, before calibration, suppression and
baseline, and the three matchers never see a finding the floor already took. It does **not** go
into `Finding::Can_Fail_A_Build` in `nomos-contracts`: a `Finding` does not know which gate is
reading it, and `OD-GATE-016` applied the same discipline when it left that function untouched.

**Unset means no floor, and migrates nobody.** The representation default is the state every
existing caller and CI's own `gate run --root .` are in today — every class may block — under the
same compatibility rule `OD-GATE-029` states for coverage and `OD-COMPLETENESS-004` settled
before it: a field nobody wrote keeps the behaviour it had. What a *newly authored* gate should
be given is `OD-GATE-029`'s question and not this record's; it was decided for coverage after
measuring the authoring surfaces, and the same measurement is owed for the floor before a default
is chosen. Whether `Unset` and a stated "no floor" are kept apart the way `OD-GATE-029` decided
`Unset` and `AllowPartial` should be is part of that same later decision.

**The floor joins the run's identity.** It is covered by `Policy_Digest`, so two runs judged
under different floors are incomparable under `OD-GATE-031` rather than compared with the
difference attributed to the repository. A floor the digest did not cover would be exactly the
false causal story that record exists to stop.

**Only findings are floored.** A fact's class is capability resolution's concern — `PKG-006`,
strongest compatible guarantee — and a rule that judges a fact already carries the weaker of the
two classes under `OD-RULES-010`'s chain. The gate reads `Finding::evidence` and nothing else.

**What an evidence requirement is not.** `EGRAPH-003`'s `EvidenceRequirement` — a required
evidence *type* against a governing obligation, with a completeness threshold, freshness
conditions and an acceptance rule — is the evidence graph's completion object, built for
`EGRAPH-008`'s attestation and nothing here. If `ARC-ROADMAP-001`'s phrase borrowed that noun,
this record narrows it at the gate: the half a gate can hold is the floor above, and the graph's
half stays where constraint 3 put it.

**The population below the floor is zero today, and the floor is built anyway.** Nothing below
`Derived` reaches a gate at this revision, so a floor of `Derived` or lower changes no run in
this workspace. It is built now for the reason `OD-GATE-016` built `CoveragePolicy` before a
caller had coverage debt to opt into: the cheapest moment to decide what an `AgentJudged` finding
does to a build is before one exists, and the first will arrive from the `AgentRequired` executor
with `GateCommand::model` as its route. The falsifier does not wait for that. The crate's own
tests already build `Finding`s by hand, and a hand-built finding at `Blocking`, `Supported`,
`AgentJudged` under a `Derived` floor is the case the increment must prove — in the new bucket
and not in `blocking_findings`, with the comparison then removed and the named test watched to
fail.

### 3. The capability item

One item builds the floor, named `P123-GATE-034-EVIDENCE-FLOOR-FIRST-INCREMENT` and authored
against this record. Its territory, by path, is what the measurement above found the change
reaches — the crate's own seams, the three readers of a bucket, the run identity, the crate doc's
admission, and the surface snapshot the new type stales:

- `crates/orchestration/nomos-gate-orchestration/src/policy/evidence_floor.rs` — new; the policy
  type, `Unset` or a named `EvidenceClass`, with its own tests
- `crates/orchestration/nomos-gate-orchestration/src/policy.rs` — the module and re-export
- `crates/orchestration/nomos-gate-orchestration/src/policy/gate_policy_file.rs` and
  `crates/orchestration/nomos-gate-orchestration/src/policy/gate_policy_file/tests.rs` — the
  declared spelling and `Resolved_Over`
- `crates/orchestration/nomos-gate-orchestration/src/gate_command.rs` — the field
- `crates/orchestration/nomos-gate-orchestration/src/gate_environment.rs`,
  `crates/orchestration/nomos-gate-orchestration/src/gate_environment/reduction.rs`,
  `crates/orchestration/nomos-gate-orchestration/src/gate_environment/provenance.rs` and
  `crates/orchestration/nomos-gate-orchestration/src/gate_environment/tests.rs` — the partition,
  `Policy_Digest`, and the falsifier
- `crates/orchestration/nomos-gate-orchestration/src/gate_plan/gate_findings.rs` — the bucket
- `crates/orchestration/nomos-gate-orchestration/src/gate_compare.rs` and
  `crates/orchestration/nomos-gate-orchestration/src/gate_compare/finding_disposition.rs` — the
  disposition variant and the population walk
- `crates/orchestration/nomos-gate-orchestration/src/finding_query.rs` and
  `crates/orchestration/nomos-gate-orchestration/src/finding_query/explanation.rs` — the
  explanation field
- `crates/orchestration/nomos-gate-orchestration/src/lib.rs` — the paragraph admitting the two
  gaps, rewritten to cite this record for both
- `crates/host/nomos-cli/src/gate/report/run.rs`,
  `crates/host/nomos-cli/src/gate/report/explain.rs` and the test modules under
  `crates/host/nomos-cli/src/gate/report/tests/` — the count and the explanation, since a bucket
  the report does not count is a hidden one
- `tests/contract/surface/nomos-gate-orchestration.txt` — the snapshot

Its `done_when` names the falsifier above, states that the field is a declared constant under
`OD-ROADMAP-003`'s surviving constraint and consults no store state, cost or prior
materialization, states that no CLI flag authors a floor in this increment — the same absence
`OD-GATE-016`'s own first increment declined to fill, in its words "no CLI flag or config file
constructs a `RequireCompleteness` policy", for the reason `OD-GATE-015` gives — and states
that the crate doc no longer names `ApplicabilityPolicy` as undefined. Its predicate reaches `nomos-gate-orchestration` and `nomos-cli`, which is where its
obligation lives; the surface snapshot is checked by `nomos-contract-tests` and named in the
`done_when` rather than covered by the predicate, and the gate's own lint step runs at `finish`
regardless. Adding the field forces no edit outside these paths, measured above; if execution
finds one, `work widen` is the repair.

## What This Does Not Do

- **It builds nothing.** The type, the bucket, the spelling, the digest and the crate-doc
  rewrite are the item above. Nothing here is built by authoring it, and the item is not
  authored here either.
- **It amends neither `ARC-ROADMAP-001` nor `OD-GATE-016`.** The first is left as it is for the
  reasons given; the second already assigns `CoveragePolicy`'s widenings and this record adds
  nothing to that assignment.
- **It does not decide the authoring default for a new gate's floor**, nor whether `Unset` and a
  stated absence of a floor are distinguishable. That is `OD-GATE-029`'s question, asked again
  for this policy once an authoring surface exists to ask it of.
- **It does not build `AllowPartial`**, which `OD-GATE-029` decided and nothing has built.
- **It touches neither `Finding::Can_Fail_A_Build`, `EvidenceClass`, `Applicability` nor
  `Claim`.** Every type the increment reads exists and is unchanged by it.
- **It does not widen `CoveragePolicy`.** A per-rule or per-scope coverage requirement and a
  minimum-`Applicability` threshold stay where `OD-GATE-016` and `OD-GATE-029` left them.
- **It does not build `EGRAPH-003`'s `EvidenceRequirement`**, and it does not decide when the
  evidence graph is constructed. Constraint 3 of `ARC-ROADMAP-001` owns that.
- **It does not take up `P12-PROVENANCE-FLOOR`'s other three clauses.** Absent provenance and
  self-referential provenance are not questions a `Finding` poses to a gate, and that item's own
  decline reason names the crate they belong in.
- **It does not floor facts.** What a provider's fact may be materialized at is capability
  resolution's own question, and the workspace's one below-`Derived` fact —
  `nomos-lang-rust-scan`'s `Approximate` scan — is evidence that the two questions have
  different populations.
- **It does not decide a per-rule floor.** `PKG-012` puts evidence obligations in a RulePackage's
  contract, and `PKG-026` is the corpus's one statement against unqualified global thresholds —
  of rule quality objectives rather than of evidence, but near enough that a per-rule floor is
  the natural next shape. One gate-wide floor is the smallest real instance, the same discipline
  each of `OD-GATE-015`'s three concerns followed.

## What Would Decide It Differently

- **A corpus revision naming a gate-side applicability policy distinct from
  unsupported-analysis policy.** Would reopen decision 1 against the new text; nothing in the
  363 requirements measured here does.
- **A gate-side need that `CoveragePolicy` cannot express even under the widenings `OD-GATE-016`
  reserves for it** — for instance a repository wanting a finding judged under a fallback
  provider not to block. Measure first whether that is a widening of coverage, a widening of this
  record's floor, or a floor over `Guarantee` rather than over either; only the third would be a
  new decision.
- **The first finding below `Derived` reaching a gate from something other than an agent.** Would
  test whether one gate-wide floor is enough or whether `PKG-012`'s per-rule evidence obligations
  are needed at the gate; that is a widening of the floor, not a different floor.
- **A finding arriving without a class** — a peer protocol producing findings the kernel cannot
  classify. Impossible while `Finding::evidence` is required; if the field ever becomes optional,
  `P12-PROVENANCE-FLOOR`'s fourth clause revives at the gate, and "absent" must read as
  unexamined rather than as the weakest class.
- **A second consumer of the same comparison outside the gate** — `AGT-003`'s validator rejecting
  "unverifiable claims," say. Two callers of one `>= floor` comparison would put the helper below
  the gate crate, beside `Weaker_Of`.
- **A floor that has to read something.** `OD-ROADMAP-003`'s surviving constraint is falsified
  the day this field cannot be stated without consulting store state, cost or a prior run, and
  the argument then belongs at `OD-RULES-009` rather than here.

## Status

Accepted. `ApplicabilityPolicy` is retired as a name for a type; `CoveragePolicy` governs what a
gate does about applicability, and its widenings stay assigned where `OD-GATE-016` and
`OD-GATE-029` put them. An evidence requirement is a floor a gate states over `EvidenceClass`'s
existing ordering, declared as a constant in `nomos-gate.json` under `OD-ROADMAP-003`'s
surviving constraint, deciding what may block and, through blocking, what a phase counts;
nothing is dropped below it, and a finding under it is reported in a bucket of its own and
covered by the run's policy identity. `P123-GATE-034-EVIDENCE-FLOOR-FIRST-INCREMENT` builds it.
Nothing is built here.
