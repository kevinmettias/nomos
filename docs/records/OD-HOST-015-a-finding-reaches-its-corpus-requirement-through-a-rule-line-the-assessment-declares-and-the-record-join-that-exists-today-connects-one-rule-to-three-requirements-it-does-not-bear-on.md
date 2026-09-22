---
id: OD-HOST-015
type: decision
title: A finding reaches its corpus requirement through a rule line the assessment declares, and the record join that exists today connects one rule to three requirements it does not bear on
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - editor
  - diagnostics
  - traceability
  - requirements
relations:
  - target: OD-HOST-010
    type: relates-to
  - target: OD-HOST-003
    type: relates-to
  - target: OD-TRACE-001
    type: relates-to
  - target: OD-TRACE-002
    type: relates-to
  - target: OD-TRACE-005
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-GATE-020
    type: relates-to
  - target: OD-SPEC-007
    type: relates-to
---

# A finding reaches its corpus requirement through a rule line the assessment declares, and the record join that exists today connects one rule to three requirements it does not bear on

## Question

`OD-HOST-010` built three of a diagnostic's five walk-outward targets and named the fourth
-- which corpus requirement (`AGT-007`, `CHK-003`, ...) a finding bears on -- as undecided
rather than declined. It named two shapes the answer could take, "a `RuleId`-to-requirement-id
table committed the same way an assessment file is" and a `rule:` field on the assessment
itself, and left the choice to "a future item should claim once someone decides the shape".
`crates/host/nomos-lsp/src/lib.rs`'s own module doc says the same, and nobody has decided.

Three facts have changed since that record. First, the join it said did not exist does exist,
in the weak form of a shared identifier: `OD-GATE-020` put a governing-record citation on
every `RuleDescriptor`, and `OD-TRACE-001`'s assessment format lets an entry name a governing
record too, so a `RuleId` can be walked to a record and a record to every assessment naming
it. Second, `nomos-cap-requirement-trace` exists, and it is the one crate that reads an
assessment as data rather than as a test fixture -- so a field added to the assessment has a
reader a host can call. Third, `Finding` still carries no requirement, and `OD-HOST-003`'s
"carries no direct pointer to a record id yet" is still true for the requirement half.

This record measures the join that exists, decides the mechanism, says which side owns the
declaration, says what a dangling or drifted link reports, and names the territory a building
item reserves. It changes no code and edits no assessment.

## What was measured

Measured at `8338c6ec6bbb095cb897675643c82e4bb62d379f`, by a script over
`crates/rules/nomos-rules/src/rule_descriptor.rs` and `tests/contract/requirements/*.assessment`
and then re-read by hand.

**The descriptor side.** `nomos_rules::DESCRIPTORS` holds 71 descriptors. Fifty-nine cite
`PORTED_STANDARD` (`"code-standards"`) and one, `NAMING_CONVENTION`, cites
`WORKSPACE_CONVENTIONS` (`"README.md"`), both at `NO_VERSIONED_RECORD`. Eleven cite a
versioned governing record, through nine constants, naming eight distinct records:

| record | version cited | descriptors citing it |
|---|---|---|
| `D-134` | 2 | `COMPLETENESS_MIRROR` |
| `OD-RULES-003` | 1 | `DEPENDENCY_DIRECTION`, `DEPENDENCY_COMPLETENESS` |
| `OD-RULES-023` | 1 | `WRITE_AUTHORITY` |
| `OD-RULES-010` | 2 | `LINT_DIAGNOSTICS`, `DEPENDENCY_POLICY`, `REVIEW_FINDING` |
| `OD-CAPABILITY-016` | 1 | `GUARANTEE_DECLARES_ITS_EXERCISER` |
| `OD-RULES-008` | 2 | `UNREAD_REACHES_FINDING` |
| `OD-CAPABILITY-010` | 1 | `CROSS_LANGUAGE_CORRESPONDENCE` |
| `OD-TRACE-001` | 1 | `REQUIREMENT_TRACE_STALENESS` |

Every one of the eleven cited versions equals the `version:` its record's own front matter
declares today, and `Test_Every_Cited_Version_Should_Match_The_Records_Own_Front_Matter` in
`tests/contract/tests/rule_contract_citation.rs` holds that on every run;
`Test_Every_Cited_Record_Should_Still_Exist` beside it holds that the record is still on disk.
That side of the join is drift-guarded.

Two prose counts of the same population have drifted while the mechanical ones held.
`rule_descriptor.rs`'s own field doc says the versioned citation is "for the eight rules whose
contract a record decides", and `Descriptor_For`'s doc says the ported standard "is what all
but eight rules in the table cite"; `OD-GATE-020`'s second amendment reports "seven rules
citing a versioned governing record, one citing `README.md`, every other rule citing the
ported standard, and no third shape found". The table holds eleven and one. Neither count is
wrong about the shape; both are wrong about the number, and nothing reddens. That is the
reason this record puts nothing about the link in prose that a test does not compare.

**The assessment side.** `tests/contract/requirements/` holds 39 assessments: 7 `Met`, 29
`Partial`, 3 `Diverges`, none `NotBinding`; 106 `site` lines and 33 `gap` lines. Eight carry
a `record` line, naming five distinct records:

| record | assessments naming it |
|---|---|
| `OD-PLATFORM-003` | `AGT-006` (Diverges) |
| `OD-TRACE-001` | `CAP-002` (Met), `CAP-003` (Met), `EVID-001` (Met) |
| `OD-CONTRACTS-002` | `CHK-003` (Met) |
| `OD-PACKAGE-010` | `MODEL-ROUTE-037` (Partial) |
| `OD-LEDGER-017` | `WORK-LEDGER-001` (Diverges), `WORK-LEDGER-005` (Diverges) |

All eight resolve to a registered record. `Test_Every_Named_Record_Should_Exist_And_Be_Registered`
in `tests/contract/tests/requirement_trace/committed.rs` holds that, and `Unresolved_Records`
in `crates/capabilities/nomos-cap-requirement-trace/src/predicates/unresolved.rs` reports the
same failure as a `ProblemKind::UnresolvedRecord` inside a real `nomos check`. That side is
drift-guarded too.

**The join.** `descriptor.contract_record == assessment.record` holds for exactly one record,
`OD-TRACE-001`, and produces exactly three pairs: `REQUIREMENT_TRACE_STALENESS` to `CAP-002`,
to `CAP-003` and to `EVID-001`. One rule of 71 and three assessments of 39. Seven of the eight
records rules cite are cited by no assessment; four of the five records assessments cite are
cited by no rule.

And the three pairs it does produce are wrong. `CAP-002` is about `FactVariant` carrying five
variants, `CAP-003` about provider selection comparing guarantees, `EVID-001` about
`EvidenceClass` carrying eight classes. `requirement-trace-staleness` reports a committed
assessment whose site, gap or record no longer resolves. It bears on none of the three. The
identifier is shared because the two `record` fields are two different relations: on the
descriptor it is "The authority this rule's implementation cites." (`rule_descriptor.rs`, the
`contract_record` field), and on the assessment it is "The governing record carrying the
reasoning, when one is named." (`crates/capabilities/nomos-cap-requirement-trace/src/assessment.rs`,
the `record` field) -- for those three entries, the record whose hand audit produced the
verdict. `OD-TRACE-001` decided the staleness rule's contract and it performed the audit
behind three verdicts, so it sits on both sides once, and the join reads that coincidence as a
bearing. A join that is empty over seventy rules and false over the remaining one is not a
small answer waiting to grow. It is the wrong relation, and no growth in either population
changes what it means.

**What already walks a finding to a record, correctly.** `nomos gate explain` stamps
`contract_record` and `contract_record_version` onto an explained finding (`GateExplainResponse`
in `crates/host/nomos-api/src/response/gate_explain_response.rs`, through `Contract_Of` in
`crates/orchestration/nomos-gate-orchestration/src/finding_query.rs`), and `nomos-lsp`'s
`GoverningRule` carries the same pair. That is the descriptor half of the join, shipped, and it
is right as what it is: the record that decided the rule. `AGT-008`'s own assessment already
counts it as the closed "rule version" clause and cites `OD-HOST-003`'s pointer sentence as
its remaining gap. Nothing here withdraws it; what is refused is extending it one hop further.

**What an assessment already names inside a rule.** One assessment of 39 names a site inside
a rule's own module: `AGT-003` names
`crates/rules/nomos-rules/src/checks/dependency/violations.rs#Violations_In`, the function
`Check_Dependency_Direction` judges through, as "the nearest real site for 'undeclared
dependency changes'". That is what a rule line is for, already stated in a form nothing can
read back: a `site` is a `path#symbol` and a finding names a `RuleId`, so a walk from a finding
to that site would have to map a rule to the file its judgment lives in, which no descriptor
declares (`RuleDescriptor::check` is a `fn` pointer, not a path). `MODEL-ROUTE-001` names
`rule_descriptor.rs#RuleDescriptor` as a gap for an unrelated reason -- a descriptor has no
field for a model profile -- and is not a link.

**What each crate may see.** `nomos-architecture.json`'s `permits` grants `Capability
Contract` only `Protocol` and `Substrate`, so `nomos-cap-requirement-trace` cannot name
`nomos-rules` and cannot compare a declared rule identifier against `DESCRIPTORS`. `Rules` may
name `Capability Contract`, which is how `Check_Requirement_Trace_Staleness` reads the trace
payload. `Host` may name both, which is how `nomos-lsp` already reads `DESCRIPTORS` (Rules)
and `nomos_repo_policy::architecture::Discover_Workspace` (Provider) in one crate.
`Verification` may name everything, which is why `tests/contract/tests/rule_descriptors.rs` is
already where two rule populations are compared.

## The decision

**A finding reaches its corpus requirement through `rule` lines the assessment declares. The
assessment owns the declaration. The descriptor gains no `requirements` field, and the record
join is refused as a mechanism.**

### 1. The line

An assessment may carry `rule: <identifier>` lines, repeated, one identifier each -- the same
grammar `site` and `gap` already use -- where the identifier is the string `Finding::rule`
carries and `RuleDescriptor::id` declares (`dependency-direction`, `requirement-trace-staleness`).
The line means: a finding from this rule bears on this requirement, because this rule is where
this build enforces the part of the requirement the entry's own sites satisfy. It is evidence
about the verdict at a site, never the verdict.

The line is optional per entry, and absence means what absence means everywhere in this
registry: nobody declared one. It does not mean no rule bears. `OD-TRACE-001` chose that
reading for the whole registry -- "Absence of an entry means nobody looked" -- and `OD-TRACE-002`
refused to let `Unassessed` be written for the same reason; a `rule: none` would record that
somebody looked and found nothing, which is a claim this record has no evidence anyone will
have checked. Fifty-nine of 71 rules are ported code-standards rules, and most bear on no v14
requirement at all; a mechanism that demanded a line for each would be answering a question
nobody asked.

The reader is the one that exists. `Read_One` in
`crates/capabilities/nomos-cap-requirement-trace/src/registry.rs` refuses an unknown key
today, which is exactly where `rule` is added; `Assessment` gains
`rules: Vec<nomos_contracts::RuleId>` (`RuleId::New` takes `impl Into<String>`, so a read line
becomes the kernel's own type rather than a second string spelling of it); an empty identifier
and a repeated one are refused the way an empty or second record line is.
`Test_The_Reader_Should_Refuse_Every_Malformed_Entry` in
`tests/contract/tests/requirement_trace/reader.rs` gains those cases. The contract suite is a
thin wrapper over that crate's reader, by its own module docs, so there is one grammar to grow.

### 2. Why the assessment owns it

Three reasons, and the first decides it.

**A requirement link is an assessment claim.** `OD-TRACE-001`'s whole mechanism is "a declared
entry committed to this repository, compared against the workspace by this suite, and never
derived from the corpus at check time" (the `requirement_trace` suite's own module doc).
Which rule enforces a requirement is a claim about *this build's* relationship to *this
corpus*, with a verdict beside it -- the same kind of claim a `site` is, and it belongs beside
the verdict it is evidence for. `DESCRIPTORS` describes a rule for every repository nomos
judges; `Discover_Workspace`'s own doc says "this predicate corpus is nomos's own, not a
convention every judged repository is expected to have adopted". A `requirements` field on the
descriptor would compile nomos's assessment of itself into the product's rule table and ship
it to every repository, and it would carry no verdict, no site and no hash -- the three things
`OD-TRACE-001`, `OD-TRACE-002` and `OD-TRACE-005` decided a requirement claim owes. It would be
a second, weaker place to say what an assessment says.

**Grain.** `OD-TRACE-002` made an assessment one file per requirement so two assessors never
collide, the shape `OD-SPEC-007` chose for registrations. A `rule` line touches one
assessment. A `requirements` field lives on the one table every rule shares, and `OD-GATE-020`
measured that table going out of step by hand twice before `OD-RULES-027` folded the judgment
into it; a per-rule corpus declaration on it would reintroduce the shared edit `OD-SPEC-007`
spent two items dissolving, on a schedule set by whoever assesses next.

**A declared universe must be checkable where it is declared.** `OD-COMPLETENESS-001`: "A
completeness guard must say what universe it quantifies over, and a declared universe must
have a check comparing it against the reality it claims to enumerate." A descriptor-side
`requirements` list would be a declared universe whose reality is the assessment set, and the
only crate below `tests/contract` that could compare the two is the rule itself -- which reads
a payload of already-judged problems, not assessments, and has no `FileSystem` port
(`checks/requirement_trace.rs`'s own doc says so). The declaration would sit where nothing
near it can check it. An assessment-side `rule` line is a declared universe whose reality is
`DESCRIPTORS`, and `tests/contract` already compares rule populations against `DESCRIPTORS`
from above both crates (`Test_Every_Composed_Rule_Should_Have_A_Descriptor`). The check goes
where the comparison is already possible.

`OD-GATE-020` put `contract_record` on the descriptor because a rule's contract authority is a
property of the rule, and that decision stands. A requirement is not a property of the rule.
The field is not extended.

### 3. Why not derive it

The tempting derivation -- a finding whose `locations` fall inside an assessment's `site`
path bears on that requirement -- is refused twice. It is the derivation at check time
`OD-TRACE-001` excludes by name, applied to a link instead of a verdict. And it is wrong on its
own terms: a `site` is where a requirement is *satisfied*, a finding's location is where a
rule *fired*, and the two coincide only by accident. `AGT-003`'s site is inside
`violations.rs`; a `dependency-direction` finding is located at the package whose edge runs
the wrong way (`OD-HOST-010` measured `violations.rs`'s own `subject_name` as a package name),
which is never `violations.rs`. Nothing about a location says which requirement a rule
enforces, and a mechanism that read one as the other would produce links as confidently wrong
as the three the record join produces.

### 4. What a dangling link reports

A `rule` line naming an identifier no descriptor in this build declares -- a typo, a rule
renamed, a rule deleted -- is red in `tests/contract`: a committed-set assertion in
`tests/contract/tests/requirement_trace/committed.rs`, beside
`Test_Every_Named_Record_Should_Exist_And_Be_Registered`, compares every declared rule
identifier against `nomos_rules::DESCRIPTORS` and names the assessment and the identifier when
one does not resolve, with a control that runs the same predicate over a constructed entry,
the shape `Test_A_Record_That_Does_Not_Resolve_Should_Be_Reported` already has. `tests/contract`
is above both crates and is where `OD-TRACE-001` put the guard.

It is deliberately **not** a sixth `ProblemKind` and not a finding from
`requirement-trace-staleness`, and the reason is the lattice, not a preference. The provider
cannot see `DESCRIPTORS` (`Capability Contract` reaches `Protocol` and `Substrate` only). The
rule could, but the payload it reads carries problems the provider already judged, and
widening `nomos.requirement.trace.v1` to carry raw declarations so the rule can judge them a
second time is the shape that crate's own `lib.rs` refuses -- "the fact this capability
answers is the *already-judged* comparison". So a dangling rule line reddens
`cargo test -p nomos-contract-tests --test requirement_trace` and does not redden
`nomos check`. That is a narrower promise than the five existing problem kinds make, stated
rather than hidden. It moves the day the payload gains a declarations section under a second
schema version, which is a decision about that capability's contract and not this record's.

A renamed rule reads as dangling, and the repair is the assessment, never the rule -- the same
direction a vanished `site` already takes. Rule identifiers are `pub const` strings in
`nomos-rules`, so this is rare.

### 5. What a drifted link reports

Nothing, and this is `OD-TRACE-001`'s limit unchanged rather than a new one. A rule that still
exists, whose finding no longer means what the entry's prose says it means, is what that
record already excluded: "Semantic drift is meaning, and this workspace has no type for it."
`Test_A_Historical_Assessment_Whose_Citations_All_Resolve_Should_Not_Be_Reported` in
`crates/capabilities/nomos-cap-requirement-trace/src/provider.rs` already proves the guard
cannot see a stale verdict behind citations that resolve, and a rule line is one more citation
of that kind.

A `rule` line carries no version, and the reason is the measurement above: 60 of 71
descriptors cite no versioned record, so a version on the line would be `NO_VERSIONED_RECORD`
for the majority, and the `Unhashed` ambiguity `OD-TRACE-005` took care to name would be the
common case rather than the transitional one. The eleven rules whose contract is versioned are
already guarded on the descriptor side by
`Test_Every_Cited_Version_Should_Match_The_Records_Own_Front_Matter`; when one of those
records is amended, re-auditing an assessment that names the rule is a person's act, the
refusal `OD-TRACE-005`'s fifth decision already makes for a drifted hash, applied once more.

The reverse direction -- a descriptor no assessment names -- is not asserted, deliberately.
The declared universe is the set of `rule` lines and its reality is `DESCRIPTORS`; that
direction is decision 4. `DESCRIPTORS` is itself mirrored against the composed run by
`Test_Every_Composed_Rule_Should_Have_A_Descriptor`. Requiring every rule to be named by some
assessment would demand a corpus claim for fifty-nine ported rules that have none, the
count-not-floor shape both `OD-SPEC-007` and `OD-TRACE-002` refused.

### 6. What the editor carries

`nomos-lsp`'s `WalkOutward` gains a sixth field, `requirements: Vec<RequirementLink>`, each
carrying the requirement identifier and `Verdict::Label`'s own word, built by filtering the
assessments of the repository under check for the finding's `rule`. The assessments are read
once per `Diagnose` batch through `nomos_cap_requirement_trace::Assessments_In` (already `pub`,
already taking a `FileSystem`) over `REGISTRY`, the same declaration-reading shape
`ArchitecturalComponent` already takes from `nomos_repo_policy::architecture::Discover_Workspace`;
a repository with no `tests/contract/requirements/` -- every repository but this one --
carries an empty list, honestly, exactly as `Discover_Workspace` reports zero problems for it.
Nothing is derived from the finding's location, per decision 3, and nothing is cached past the
batch (`OD-HOST-002`).

`OD-HOST-010` predicted "a sixth field there is an addition to that one function", meaning
`Diagnostics_For`. The field is. The declaration it reads is not already in hand the way the
architecture is, so `Diagnostics_For` gains a parameter, or a context struct carrying both
declarations, and that is a change to a `pub fn` signature the surface snapshot will see.

## What this record does not do

**It edits no assessment and writes no `rule` line.** The one entry whose site already sits
inside a rule module, `AGT-003`, is the first candidate for a line naming
`dependency-direction`, and authoring it is the building item's act against the assessment's
own prose, not this record's.

**It changes no code.** No reader, no test, no `WalkOutward` field, no dependency edge. The
territory below is a prediction, and `OD-LEDGER-039`'s `work widen` is how execution corrects
it.

**It does not reopen `OD-HOST-010`.** That record named this decision as owed and named the
territory it would take; this is the decision, and its follow-up paragraph reads true
afterwards. It does not amend `OD-GATE-020`: the descriptor's citation is unchanged and its
meaning is unchanged. It does not touch the supporting-facts gap, `OD-HOST-010`'s other open
target, which needs a per-finding read trail from `CheckOutcome` and is a different decision.

**It does not make `nomos check` see a dangling rule line.** Decision 4 says why, and names the
condition under which that changes.

## Territory a building item reserves

Predicted from the acceptance predicate's dependency cone, in the manner
`.claude/skills/nomos-task/SKILL.md` requires, and validated by running the change:

- `crates/capabilities/nomos-cap-requirement-trace/src/assessment.rs` and `src/registry.rs`
  -- the field and the `rule` key;
- `tests/contract/surface/nomos-cap-requirement-trace.txt` -- a `pub` field is a surface
  change;
- `tests/contract/tests/requirement_trace/reader.rs`, `committed.rs` and `controls.rs` -- the
  malformed-entry cases, the dangling-rule assertion, and its control;
- `crates/host/nomos-lsp/src/walk_outward.rs`, a new `src/walk_outward/requirement_link.rs`,
  `src/lib.rs`, `src/file_diagnostic.rs` and `src/nomos_diagnostic_provider.rs` -- the sixth
  field, its module, its re-export, the parameter that hands the assessments in, and the read
  once per batch;
- `crates/host/nomos-lsp/Cargo.toml` and `Cargo.lock` -- the new dependency on
  `nomos-cap-requirement-trace`, within `Host`'s own `permits`;
- `tests/contract/surface/nomos-lsp.txt` -- the new type and the changed signature;
- each assessment that gains a line, named individually --
  `tests/contract/requirements/AGT-003.assessment` first;
- `README.md`, whose `nomos-lsp` row says "Three of `P42-LSP-PROJECTION`'s five walk-outward
  targets are answered; two are named undecided in `OD-HOST-010`." and reads stale the moment
  the fourth is answered -- the kind of file `nomos-task` names as invisible to a diff until
  somebody asks what the change made untrue.

Not reserved, deliberately: `crates/rules/nomos-rules` (no descriptor field, no rule change),
`crates/contracts/nomos-contracts` (`Finding` is unchanged; the link is walked from a finding,
not carried on it), `crates/orchestration/nomos-check-orchestration`, and the trace payload's
schema. A building item that finds itself needing any of them has found a different decision
than this one.

The predicate is `cargo test --no-fail-fast -p nomos-contract-tests --test requirement_trace`
for the assessment half and `cargo test --no-fail-fast -p nomos-lsp` for the editor half, with
`public_surface` blessed for exactly the two crates above -- scoped to the obligations the item
claims, per the same skill. If the assessment half and the editor half are worked by two
sessions, they are two items: the second depends on the first, and the territories above
split cleanly along the crate line.

## Status

Accepted. At `8338c6ec` the record join connects one rule to three assessments through
`OD-TRACE-001`, and every pair is a coincidence of citation; it is refused as a mechanism. A
finding reaches its corpus requirement through `rule` lines an assessment declares, read by the
reader `nomos-cap-requirement-trace` already has, checked against `nomos_rules::DESCRIPTORS`
from `tests/contract`, and carried by `nomos-lsp` as a sixth walk-outward field. Nothing is
built here.
