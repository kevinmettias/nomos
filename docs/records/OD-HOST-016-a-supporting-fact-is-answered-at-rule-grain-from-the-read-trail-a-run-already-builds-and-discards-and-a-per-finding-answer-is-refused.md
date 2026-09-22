---
id: OD-HOST-016
type: decision
title: A supporting fact is answered at rule grain from the read trail a run already builds and discards, and a per-finding answer is refused
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - editor
  - diagnostics
  - analysis
  - capability
  - provenance
relations:
  - target: OD-HOST-010
    type: relates-to
  - target: OD-HOST-015
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-003
    type: relates-to
  - target: OD-ANALYSIS-001
    type: relates-to
  - target: OD-ANALYSIS-002
    type: relates-to
  - target: OD-ANALYSIS-009
    type: relates-to
  - target: OD-ANALYSIS-011
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-CAPABILITY-004
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-GATE-020
    type: relates-to
  - target: OD-RULES-027
    type: relates-to
---

# A supporting fact is answered at rule grain from the read trail a run already builds and discards, and a per-finding answer is refused

## Question

`OD-HOST-010` built three of a diagnostic's five walk-outward targets and named two
undecided rather than declined. `OD-HOST-015` decided the fourth -- which corpus requirement
a finding bears on -- an hour before this record, and refused the record join it was written
to consider. This is the fifth and last: **which fact, at what guarantee, backed this
judgment.** `OD-HOST-003` named it as "the guarantee behind it" before
`nomos-check-orchestration` had a caller-callable `Run`; `OD-HOST-002` named it as state
family 4; `OD-HOST-010` named it again and named the two shapes an answer could take. Three
records have named it and none has decided it.

`OD-HOST-010`'s own "Supporting facts are not" paragraph is the prior measurement. Two of its
citations have moved since -- `run_context.rs:590` became
`crates/orchestration/nomos-check-orchestration/src/run_context/judging.rs:33` when that file
was split on 2026-09-15 -- and one fact it named as evidence has been repaired:
`crates/rules/nomos-rules/src/checks/lint.rs:153` now files
`format!("{}:{}", diagnostic.file, diagnostic.line)`, closing the line-drop that record
recorded and did not fix. Everything else it measured still holds, and the re-measurement
below adds two numbers nobody had taken: how many composed rules read a fact at all, and
whether any rule reaches the `Guarantee` of the offer that answered it.

This record measures, decides the mechanism, says what an answer may honestly claim, says
what an absent trail means so it is not read as a judgment made without evidence, and names
the territory a building item reserves. It changes no code.

## What was measured

Measured at `9bae4312d6f4862e5e3195c4153db526686559d2` and re-verified unchanged at
`3df66fe8d79cdcbdaaf7bc9d30c14ba0991f8bd2`, in a detached worktree, by scripts over
`crates/rules/nomos-rules/src/rule_descriptor.rs` and the rule modules, then re-read by hand.

### One reader per run, and its trail is never read

A `nomos check` run builds exactly **one** `nomos_analysis::Reader`, at
`crates/orchestration/nomos-check-orchestration/src/run_context/judging.rs:33`, inside
`Judged_Findings`, and passes `&mut` of it to every selected rule in turn through
`Findings_For_Selected_Rules`. Materialization builds none: that is the only `Reader::On` in
the crate's non-test code. Workspace-wide, exactly two non-test `Reader::On` sites exist, the
other being `crates/languages/nomos-lang-rust/src/rollup/materialize.rs:137`.

`Reader::Into_Dependencies` has **zero** callers in `crates/orchestration/` and one in the
whole workspace's non-test code: `nomos-lang-rust/src/rollup/materialize.rs:151`, which hands
the trail to `MemoryFactStore::Materialize` as the invalidation edges of a derived fact --
the use `OD-ANALYSIS-002` built it for ("The edges are observed, never listed").

So `OD-HOST-010`'s finding is true and understated. The run's trail is not merely a flat list
for the whole run rather than attributable to one finding; **it is never consumed at all.**
`Trail::Note` clones a `FactKey` on every read and appends it
(`crates/substrate/nomos-analysis/src/reading/trail.rs`), `Trail::Note_Miss` records a miss as
`ReadOutcome::Degraded(Applicability::DependencyUnavailable)`, and the whole accumulation is
dropped when `Judged_Findings` returns. The cost of collecting it is already being paid.

### How many composed rules read a fact at all

`nomos_rules::DESCRIPTORS` (`crates/rules/nomos-rules/src/rule_descriptor.rs:181`) holds
**71** descriptors. Classified by what each declares in `RuleDescriptor::requires`:

| population | count | what a fact trail can say about it |
|---|---|---|
| declares no required family (`SubjectKind::SourceText`) | 31 | nothing, ever -- the judgment is a closure whose reader parameter is named `_reader` and discarded |
| declares only an optional policy family | 15 | the read happened; a miss is silent by `OD-CAPABILITY-004` |
| declares at least one non-optional family | 25 | the read happened, and a miss is already a `Finding` |

The 31 and the closure count agree exactly: 31 entries name `_reader`, 2 name `reader`, 38
pass a named `fn`. `Test_A_Source_Text_Rule_Should_Require_No_Fact` and
`Test_A_Fact_Reading_Rule_Should_Require_At_Least_One_Fact`, both in that file, hold the two
halves of that correspondence on every run, which is why the first row can be derived rather
than declared.

**So 40 of 71 composed rules read a fact at all, and 31 can never have a fact trail.** That
is the number that changes what an honest walk-outward answer looks like: for 44% of this
build's rules the honest answer is not "no fact recorded" but "this rule judges source text
and no fact was ever involved", and the two must not share a word.

Twenty-six rules touch a policy family whose absence is deliberately silent -- the 15 above
plus 11 of the 25 that read one beside a required family. `Naming_Policy_Payload`
(`crates/rules/nomos-rules/src/checks/naming.rs:111`) folds "every way the read can come back
empty (absent, refused, malformed) into `None`"; `Resolve_Declared_Fixture_Locations`
(`crates/rules/nomos-rules/src/checks.rs:279`) returns `Vec::new()` on any `Require`
failure; `Materialized_Limits_Payload` (`checks/structure.rs:122`) returns `None` and the
caller uses a built-in default. Every one of those is correct under `OD-CAPABILITY-004` and
`OD-RULES-011`, and every one of them means a finding today cannot say whether it was judged
against the repository's declared limit or against nomos's own compiled-in number. This is
the concrete thing the gap costs, on this repository, today.

### The Requirement floor is private, and is not a per-rule constant

Sixteen `*_Requirement()` functions exist in `nomos-rules`, under fourteen distinct names
(`Words_Policy_Requirement` and `Limits_Policy_Requirement` are each written twice, in two
modules). **None is `pub`**, and `tests/contract/surface/nomos-rules.txt` carries no
`Requirement` at all, so no floor any rule states is nameable from outside that crate.

The floors are real information and they differ. `Syntax_Requirement`
(`crates/rules/nomos-rules/src/lib.rs:141`) asks `Syntactic / Sound / Unknown / File`;
`Lint_Requirement` (`checks/lint.rs:89`) asks `SemanticallyResolved / Sound / Unknown /
Project`; `Dependency_Requirement` (`checks/dependency/reading.rs:38`) asks
`SemanticallyResolved / Sound / Sound / Project`; `Architecture_Requirement` asks at
`nomos_cap_architecture::Ceiling()`.

And one of them is not a per-rule value at all. `Syntax_Requirement_For`
(`crates/rules/nomos-rules/src/lib.rs:174`) takes the source's own
`preferred_syntax_provider` and returns `need.Preferring(provider)`, so the naming family's
floor varies per subject within one rule call. A field on the descriptor table could not hold
it.

### The Guarantee is not discarded at resolve time; it is discarded at judgment time

This is the measurement that decides whether "at what guarantee" is recoverable without new
machinery, and the answer is that it is.

`Reader::Require` (`crates/substrate/nomos-analysis/src/reader.rs`) calls
`self.registry.Resolve(need)` into a local, takes the chosen offer, and drops the
`Resolution` when it returns. What dies there is the floor (`Requirement::minimum`) and the
offers passed over (`Selection::alternatives`, "Every other usable offer, strongest first").
What survives does so twice:

- in the `FactKey`, as `guarantee: GuaranteeDigest` -- `GuaranteeDigest::Of` is
  `Digest_Of_Parts` over the four guarantee bytes
  (`crates/substrate/nomos-analysis/src/fact/guarantee_digest.rs`), so it is one-way and
  **cannot be read back**; it can confirm a candidate and never yield an answer;
- on the `MaterializedFact` the rule receives, as `guarantee: Guarantee` in full, beside
  `evidence: EvidenceClass` and `snapshot: SnapshotId`
  (`crates/substrate/nomos-analysis/src/fact/materialized_fact.rs:28`).

`Require` and `Require_Any` both return `&MaterializedFact`. **So every one of the 40
fact-reading rules already holds the guarantee of the offer that answered it, and not one
reads it.** `.guarantee` occurs six times in `crates/rules/nomos-rules/src/`, every occurrence
in a test fixture or in `checks/test_support.rs`. Nothing in a judgment path reads it.

Nor is the guarantee reflected in what a rule does say about provenance. Sixty
`evidence: EvidenceClass::` literals appear in that crate's non-test code -- 57 `Derived`, 2
`Verified`, 1 `Observed` -- and not one is derived from the fact that was read.
`Finding::evidence` is documented as "How this claim was come by"
(`crates/contracts/nomos-contracts/src/reporting/finding.rs`), `nomos-lsp` already publishes
it to editors as `WalkOutward::evidence`, and it is a rule author's constant.

That the answer is not constant is measurable. `nomos-check-orchestration`'s `Registered()`
makes 17 `registry.Offer` calls, and `nomos.cap.syntax.items` has **three** offers at three
different guarantees: `nomos-lang-rust` at `Syntactic / Sound / Unknown / File`,
`nomos-lang-rust-scan` at `Approximate / Unsound / Unknown / File`, and `nomos-lang-go` at
`Syntactic / Sound / Sound / File`. `nomos.cap.dependency` has two. A Rust file's syntax fact
and a Go file's syntax fact, read by the same rule in the same run, differ in declared
completeness, and nothing a diagnostic carries says so.

`Require_Any`, the call that would report having answered from below the floor, has **one**
non-test caller in the workspace -- `nomos-lang-rust/src/rollup/materialize.rs:213` -- and
**no rule uses it**. Every rule calls `Require`, so `Applicability::SupportedWithFallback`
never arises from a rule read today.

### The store is addressable only by a key nobody outside a rule ever held

`MemoryFactStore`'s entire public surface (`tests/contract/surface/nomos-analysis.txt`) is
`New`, `Materialize`, `Current(&FactIdentity, at)`, `Historical(&FactKey)`,
`Dependencies_Of(&FactKey)`, `Live()` and `Materializations()`. There is **no enumeration and
no search**. Every read is addressed by a key the caller must already hold, and a `FactKey` is
nine components (`crates/substrate/nomos-analysis/src/fact_key.rs`), three of which are
private to the rule's own call: which capability it asked for, what bytes it hashed into
`semantic_inputs`, and which `Requirement` it stated -- the last deciding, through
`Registry::Resolve`, both the provider and the guarantee digest.

The caller that would want the answer already holds the store.
`nomos_lsp::NomosDiagnosticProvider` keeps `workspace`, `store` and `reassessment` across
calls and judges through `Run_Reassessing`
(`crates/host/nomos-lsp/src/nomos_diagnostic_provider.rs`). It has the store and no key.

*(`OD-ANALYSIS-009`'s second amendment states that `nomos-lsp` "constructs `let mut workspace
= None;` and `let mut store = MemoryFactStore::New();` fresh, inline, on every call" in a
`server.rs` that no longer exists in that crate. That is stale at this revision and is named
here as found evidence, in the manner `OD-HOST-010` named the lint line, not corrected here.)*

### A reused finding was produced by a rule that did not run

`RuleReassessmentCache::Reusable`
(`crates/orchestration/nomos-check-orchestration/src/run_context/rule_reassessment_cache.rs`)
returns a prior call's findings without invoking the rule when no family it reads has changed,
and `nomos-lsp` is the caller that keeps such a cache across an editor session. So on most
`didSave` notifications the findings an editor shows were produced by rules that performed no
read in that call at all. A trail collected during a call is empty for every one of them.

That file also settles what `requires` means, and says so directly: `Effective_Requires`
substitutes `[RequiredFact::SyntaxItems]` for a rule declaring none, "Declaring the dependency
here rather than widening `DESCRIPTORS` itself keeps that table's own claim exact: a
source-text rule truly needs no fact *materialized* to be judged, which is a different
statement from what tells this cache the rule has gone stale." **`RuleDescriptor::requires` is
a materialization demand, not a read trail** -- two different relations, the same distinction
`OD-HOST-015` drew one record ago between a descriptor's `record` and an assessment's.

### `requires` is a declared universe with no check against reality

Six sites read `RuleDescriptor::requires`: `capabilities.rs:107` unions it into what a run
materializes, `rule_reassessment_cache.rs` invalidates from it, `declared_rules.rs:112`
reports it, and `crates/host/nomos-lsp/src/walk_outward/governing_rule.rs:48` publishes it to
editors as `GoverningRule::required_facts`. Three tests constrain it, all in
`rule_descriptor.rs`: the two subject-kind correspondences above and
`Test_Every_Required_Family_Should_Resolve_To_A_Capability`, which checks that a variant maps
to a capability id. `tests/contract/tests/rule_descriptors.rs` compares the descriptor table
against the composed run in both directions and says nothing about `requires`.

**Nothing anywhere compares what a rule declares it requires against what it actually reads.**
That is `OD-COMPLETENESS-001`'s exact shape -- "a declared universe must have a check comparing
it against the reality it claims to enumerate" -- and it is already shipping to editors: a
reader of `GoverningRule::required_facts` is reading a declaration, correctly labelled by that
module, that could be misread as the facts the finding rests on.

### A finding's subject is not the subject its rule read

Fifty-six `subject:` lines construct a `Finding` in that crate's non-test code. Forty-four use
the source's own subject. **Twelve do not**: seven digest a qualified name
(`checks/finding_shape.rs:50`, `checks/function_shape.rs:228`, `checks/mirror/verdict.rs:52`,
`checks/naming/abbreviations.rs:411`, `checks/naming/go/data_names.rs:86`,
`checks/naming/go/function_names.rs:219`), one digests the source *text*
(`checks/mirror/unread.rs:16`), two digest a declared address
(`checks/guarantee_exerciser.rs:257,278`), one a crate root
(`checks/role_surface_pair.rs:92`), and two the empty path (`checks/goals.rs:238`,
`checks/requirement_trace.rs:103`). Every one of those rules keys its own `Require` on
`source.subject` -- `checks/naming/reading.rs:85` is the naming family's, verbatim.
`OD-ANALYSIS-011` already recorded that a finding's `subject_name` is not the preimage of its
`subject`; this is the same asymmetry on the other side.

## The decision

**A supporting-fact answer is produced at rule grain, from the read trail the run already
builds and discards, reduced and resolved while the store is still in hand, carried on
`CheckOutcome::Judged` and cached beside the findings it belongs to. A per-finding answer is
refused, and so are a descriptor-side `Requirement` field and a `FactKey`-reconstruction
capability.**

### 1. A `Reader` per rule, and the trail is kept

`Judged_Findings` stops building one `Reader` for the whole run. `Findings_For_Selected_Rules`
builds one per descriptor it runs, calls `Reader::Into_Dependencies` after
`RuleJudgment::Judges` returns, and keeps the result against that rule's identifier.

This is behaviour-preserving by construction: a `Reader` holds a store, a registry, a context
and a trail, and only the trail is per-reader. The same reads happen against the same store
and resolve the same way, so no finding moves. Nothing in `nomos-analysis` changes:
`Into_Dependencies`, `Dependency`, its `key` and `outcome` fields, `ReadOutcome`, `FactKey`,
`MemoryFactStore::Current` and `nomos_contracts::Guarantee` are all already public, which
`tests/contract/surface/nomos-analysis.txt` states. The substrate has been ready the whole
time; what was missing was a caller.

### 2. The trail is reduced, and the reduction drops the subject deliberately

A raw trail is one `Dependency` per read, and a rule reading one syntax fact per source over
this repository produces on the order of a thousand of them. Retaining that per rule is not
what a reader needs and is not what this decision keeps.

At the end of each rule's call the trail is reduced to the **distinct** provenance tuples it
contains: the capability, the provider and its version, the read outcome, and -- for a read
that was answered -- the `Guarantee` and `EvidenceClass` read off the `MaterializedFact` via
`MemoryFactStore::Current` at the run's own generation, plus a count of the reads behind each
tuple. For a rule reading one family that is one or two tuples, bounded by the number of
distinct offers that actually answered, not by the number of files.

The reduction resolves through the store *inside the run*, so the answer that leaves is
`Guarantee` and `EvidenceClass` by value and no caller ever needs a key or a store. The
`FactKey`'s own digest components -- `semantic_inputs`, the `GuaranteeDigest`, the variant and
the configuration -- are deliberately not carried out: they are identity, not provenance,
which is `OD-ANALYSIS-001`'s whole subject and what `MaterializedFact::snapshot`'s own doc
means by "Provenance, not identity". A digest answers no question a reader can ask.

**The subject is dropped, and that is the load-bearing refusal.** Keeping it would offer a
per-finding join by subject equality, which reads as exact and is false for 12 of the 56
finding constructions measured above -- reporting "no supporting fact" for a finding whose
rule provably read one, silently, in exactly the direction `OD-COMPLETENESS-001` warns about.
It is the same failure `OD-HOST-015` refused one record ago for the record join: two fields
that share a type and mean different things, joined on the coincidence. A mechanism cannot
claim a precision its own grain does not have, so the grain is stated and the subject goes.

### 3. What the answer is, and the four shapes it takes

The answer for a finding is one of four, and they are four words rather than one word and an
absence:

- **`NotFactBacked`** -- the rule's descriptor is `SubjectKind::SourceText`. 31 of 71 rules.
  Derived from the descriptor, not declared, and
  `Test_A_Source_Text_Rule_Should_Require_No_Fact` already guarantees the correspondence. The
  honest claim is "this rule judges source text; no fact was involved and none could have
  been", never "no fact recorded".
- **`Read`** -- the reduced tuples, which may be several and may be all misses. What it may
  honestly claim is exactly: *the rule that produced this finding, in the call that produced
  it, read these capabilities from these providers at these guarantees, and these reads
  missed.* It may not claim that any particular one of them backs this particular finding,
  and the wording carries that. Several facts is the ordinary case, not an exception.
- **`RaisedByMaterialization`** -- a finding from `Capability_Findings`
  (`run_context/judging.rs`), raised by the dependency, lint, policy or review materialization
  rather than by a rule's judgment. It carries a `RuleId` and never passed through a
  `RuleJudgment::Judges` call, so no rule trail exists for it. Naming it is what stops it from
  landing in the next bucket.
- **`Unrecorded`** -- the run did not keep a trail. A finding read back from a
  `CheckOutcome` a caller did not ask for one from, or deserialized from a projection that
  predates the field.

The hazard this vocabulary exists for is that an absent trail reads as a judgment made without
evidence. Three things prevent it. The first shape says the absence is structural. The third
says whose absence it is. And for the second, a rule whose read genuinely failed already
reports it without any of this: `Require_Fact` in `checks/lint.rs`, `checks/policy.rs`,
`checks/review.rs`, `checks/naming/reading.rs` and their siblings turn an inadmissible answer
into an `Unread` finding carrying the failing `Applicability`, and `Finding::Can_Fail_A_Build`
already refuses to fail a build on one. What the trail adds is the case those cannot reach:
the 26 rules whose optional policy read is silent by design, where a miss today produces no
signal at all and a `Read` answer containing a miss is the only way a person learns that a
finding was judged against a compiled-in default instead of their declaration.

### 4. On `CheckOutcome::Judged`, not a second return shape

`OD-HOST-010` named both. The field is chosen, for `OD-HOST-002`'s reason: two return shapes
make the richer one the only complete one and every caller taking the thinner one sees a
subset, which is the failure that record describes in its own opening. It is also cheap,
measured rather than assumed: exactly two non-test sites destructure `Judged` exhaustively --
`crates/host/nomos-api/src/check.rs:113` and
`crates/host/nomos-cli/src/check/report.rs:147` -- and every other reader already writes `..`.

The trail is recorded in `RuleReassessmentCache` beside the findings it produced, and reused
with them. Without that, the one caller that reuses findings is the one caller that sees
`Unrecorded` on nearly every keystroke, which would make the mechanism worst exactly where it
is wanted most.

### 5. The descriptor gains no `Requirement` field

Refused, on three grounds and the first decides it.

**A floor is not an answer.** `Requirement::minimum` is what the rule would not do without;
the question is what answered. Publishing the floor beside a finding would look like the
provenance and would not be it -- a rule asking `Sound / Unknown` tells a reader nothing about
whether `nomos-lang-rust` or `nomos-lang-go` answered, which is the whole of what differs.

**It is not well-defined per rule.** Sixteen `*_Requirement()` functions serve 40 fact-reading
rules, two names are written twice in two modules, and `Syntax_Requirement_For` varies the
floor per subject by the source's preferred provider. There is no per-rule constant to put in
a constant table.

**It would be a second unchecked declaration on a shared table.** `OD-GATE-020` measured that
table going out of step by hand twice; `OD-HOST-015` refused a `requirements` field on it one
record ago for the same reason and for `OD-SPEC-007`'s grain argument; and `requires`, the
declaration already there, has no check against reality at all. Adding a second one before the
first has a falsifier is the wrong order.

The trail supplies the honest version of what that field was reaching for -- the guarantee
that answered, observed rather than declared, the same move `OD-ANALYSIS-002` made for
invalidation edges.

### 6. A `FactKey`-reconstruction capability is refused

To rebuild a key from outside a rule, a caller must supply the capability, the bytes hashed
into `semantic_inputs`, and the `Requirement` -- and none of the sixteen requirement functions
is `pub`. It would then have to re-run `Registry::Resolve` to get the provider and the
guarantee digest, which is a second implementation of the first half of `Reader::Require` that
must agree with it and that nothing would compare. And a correct key buys only a
`MemoryFactStore` lookup, which the reduction in decision 2 performs inside the run where the
store is unambiguous and the generation is known.

It is also the wrong kind of answer. `OD-TRACE-001` excludes derivation at check time, and
`OD-HOST-015`'s third decision refused the location-based derivation for the requirement link
on the same ground. Re-deriving privately held state from outside is that shape again.

### 7. What the editor carries

`nomos_lsp::WalkOutward` gains a `supporting_facts` field: the four-shape answer above for the
finding's rule, filtered out of the trail the outcome now carries. `Diagnostics_For` gains the
trail as an input, the same way `OD-HOST-015` predicted it would gain the assessments -- both
building items add a parameter to that one `pub fn`, and if both land they should hand it a
context struct rather than a third parameter. The two decisions are otherwise disjoint;
neither field reads the other's input.

`GoverningRule::required_facts` stays exactly as it is and keeps meaning exactly what its doc
says -- what the descriptor declares. With `supporting_facts` beside it, the declared families
and the observed reads sit next to each other in one payload, which is the first time a reader
of a nomos diagnostic can see a declaration and its reality together.

### 8. What the building item owes as a falsifier

`OD-COMPLETENESS-001`'s first corollary exempts a derived universe, and the trail is derived,
so it owes nothing on its own account. What it makes possible for the first time is the check
`requires` has never had: a contract test running a real check over a fixture tree and
asserting that every capability a rule's observed trail names is one its own descriptor
declares. `tests/contract` is where `Test_Every_Composed_Rule_Should_Have_A_Descriptor` already
compares rule populations against `DESCRIPTORS` from above both crates, and it is where this
belongs. The building item runs that comparison and reports what it finds; this record does not
predict the answer, because a predicted number is the thing `OD-HOST-015` measured two records
going stale about.

Separately, and per `.claude/skills/nomos-task/SKILL.md`'s falsifier rule, the per-rule split
owes a test that distinguishes it from the shared reader: one rule's trail must not contain
another rule's reads, and a run of two fact-reading rules is the fixture that shows it.

## What this depends on from `OD-ANALYSIS-009`

**Nothing that record leaves open.** Its subject is whether a fact store should survive a
process, and it declines that, twice. This decision needs the store only for the duration of
one rule's reduction, inside `Run`, so it takes nothing from the open question and does not
disturb the decline.

It does rest on what that record's *first* amendment built --
`P14-ANALYSIS-009-STORE-WORKSPACE-REUSE-FIRST-INCREMENT`'s caller-supplied `workspace` and
`store`, whose equivalence to a clean recomputation
`Test_A_Store_And_Workspace_Reused_Across_An_Edit_Agrees_With_A_Clean_Recomputation` proves --
and on the reassessment cache built beside it. The dependency is a real constraint rather than
a courtesy: because findings are reused, a trail that lives only for the call is empty for the
findings an editor actually shows, which is decision 4's second paragraph. If that cache did
not exist, this decision would be one field and no cache change.

## What this record does not do

**It changes no code**, adds no field, and writes no test. The territory below is a
prediction, and `OD-LEDGER-039`'s `work widen` is how execution corrects it.

**It does not amend `OD-HOST-010`.** That record named this target open and named the
follow-up territory it would take -- "`nomos-check-orchestration` (widening `CheckOutcome` or
adding a second, opt-in return shape) and `nomos-rules` (deciding whether `RuleDescriptor`
should carry a rule's own `Requirement` floor) together". This is that decision, and its
follow-up paragraph reads true afterwards except in one particular: `nomos-rules` is **not**
reserved, because decision 5 refuses the field that record thought might need it.

**It does not amend `OD-ANALYSIS-009`, and does not correct its stale `server.rs` citation.**
That crate reorganized after the amendment was written; repairing it is that record's own
territory.

**It does not touch `Finding`.** The answer is walked from a finding through its rule, not
carried on it, the same direction `OD-HOST-015` chose for the requirement link and for the
same reason: a `Finding` is what a rule says about a subject, and how the rule came to say it
is a property of the run.

**It does not make `nomos check`'s text report carry provenance**, and does not widen
`nomos-api`'s response twins. Both are real, separate questions about what a surface renders,
and `OD-HOST-011` owns the second.

## Territory a building item reserves

Predicted from the acceptance predicate's dependency cone, in the manner
`.claude/skills/nomos-task/SKILL.md` requires, and validated by running the change. It splits
cleanly along the crate line into two items, the second depending on the first.

The orchestration half:

- `crates/orchestration/nomos-check-orchestration/src/run_context/judging.rs` -- the per-rule
  reader and the reduction;
- a new `crates/orchestration/nomos-check-orchestration/src/examined/fact_trail.rs` (or a
  sibling of it) for the provenance type and the four-shape answer,
  `src/examined/check_outcome.rs` for the new `Judged` field, `src/examined.rs` and
  `src/lib.rs` for the declaration and the re-export;
- `crates/orchestration/nomos-check-orchestration/src/run_context/rule_reassessment_cache.rs`
  -- the trail cached beside the findings, and `src/run_context.rs` where `Reassessment` and
  `JudgeEnvironment` are declared and `CheckOutcome::Judged` is constructed (line 342);
- `crates/orchestration/nomos-check-orchestration/src/run_context/tests.rs` and
  `src/tests/composition.rs` -- every exhaustive `Judged` destructuring inside the crate;
- `tests/contract/surface/nomos-check-orchestration.txt` -- a new `pub` type and a new field;
- `crates/host/nomos-api/src/check.rs` and `crates/host/nomos-cli/src/check/report.rs` -- the
  two non-test exhaustive destructurings, plus the `Judged` constructions in
  `crates/host/nomos-api/src/response/check_outcome_response.rs` and
  `crates/host/nomos-cli/src/gate/report/tests*` that a new field breaks. These are the
  "compiler reveals the cone" files `nomos-task` warns a static reservation will miss, named
  here because they were grepped rather than guessed;
- `tests/contract/tests/rule_descriptors.rs` -- the declared-versus-observed `requires`
  comparison of decision 8.

The editor half:

- `crates/host/nomos-lsp/src/walk_outward.rs`, a new `src/walk_outward/supporting_facts.rs`,
  `src/lib.rs`, `src/file_diagnostic.rs` and `src/nomos_diagnostic_provider.rs` -- the field,
  its module, the re-export, and the parameter `Diagnostics_For` gains;
- `tests/contract/surface/nomos-lsp.txt` -- the new type and the changed signature;
- `README.md`, whose `nomos-lsp` row ends "Three of `P42-LSP-PROJECTION`'s five walk-outward
  targets are answered; two are named undecided in `OD-HOST-010`." and reads stale the moment
  the fifth is answered. `OD-HOST-015` reserves the same sentence for the fourth; whichever
  lands second corrects what the first left.

Not reserved, deliberately: `crates/substrate/nomos-analysis` (nothing there changes -- the
measurement above is that its whole surface is already sufficient),
`crates/substrate/nomos-capability`, `crates/contracts/nomos-contracts` (`Finding` is
unchanged), and `crates/rules/nomos-rules` (decision 5 adds no descriptor field, and no rule
body moves). A building item that finds itself needing any of them has found a different
decision than this one.

The predicate is `cargo test --no-fail-fast -p nomos-check-orchestration` for the
orchestration half and `cargo test --no-fail-fast -p nomos-lsp` for the editor half, with
`public_surface` blessed for exactly the crates named above -- scoped to the obligations each
item declares, per the same skill, and deliberately not reaching the contract suite's other
obligations, which belong to whoever holds them.

## Status

Accepted. At `3df66fe8` a run builds one `Reader` and throws its trail away unread; 31 of 71
composed rules can never have a trail at all; every one of the other 40 already holds the
`Guarantee` of the offer that answered it, on the `MaterializedFact` `Require` hands back, and
not one reads it, while all 60 of the crate's finding constructions state an `EvidenceClass`
as a literal. The answer is built from the trail, at rule grain, reduced to distinct
provenance tuples and resolved against the store inside the run; the subject is dropped and a
per-finding join refused, because a finding's subject is not the subject its rule read in 12
of 56 construction sites; a descriptor-side `Requirement` field and a `FactKey`-reconstruction
capability are both refused with their reasons. `OD-HOST-010`'s two undecided targets are now
both decided, neither of them built, and `P42-LSP-PROJECTION`'s five walk-outward targets have
five answers on the board.
