---
id: OD-RULES-034
type: decision
title: A declared form is owed for the archetype forty-four of seventy-one rules share, and the detector stays Rust
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - authoring
  - packages
  - measurement
relations:
  - target: OD-RULES-007
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-022
    type: relates-to
  - target: OD-RULES-027
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-GATE-020
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
---

# A declared form is owed for the archetype forty-four of seventy-one rules share, and the detector stays Rust

## Question

Every rule this workspace ships is a Rust function compiled into `nomos-rules` and named in
one `const` table, so authoring one costs a function, a descriptor row and, when the rule
looks at a construct nothing else looks at, a detector. An external review named a rule
authoring surface as the largest gap against arbitrary rule creation, and in the same breath
said not to invent a rule intermediate representation before real rules require one.
`OD-RULES-007` already holds the second half and is open on it.

What has changed is that the population is now large enough to ask the first half from
evidence. Nobody has. This record asks it by counting, and the counting is the point: the
question "is a declarative form owed" has no answer that does not begin with what the rules
actually are.

## Method

Every count below is taken from `crates/rules/nomos-rules` at the revision this record lands
on, by a command a later reader can re-run.

- The table's size is `awk '/^pub const DESCRIPTORS/,/^\];/' src/rule_descriptor.rs | grep -c
  'Descriptor_For('`.
- The per-rule columns — subject kind, required families, judgment function — are parsed out of
  the same slice of that file, one row per `Descriptor_For(`, because the table states all three
  on one line per rule.
- The exported population is `grep -c '^pub fn nomos_rules::Check_'
  tests/contract/surface/nomos-rules.txt`, the committed surface snapshot, which is also the
  list `Test_Every_Exported_Rule_Is_Composed_Or_Accounted_For` reads.
- Rule identifiers are the distinct string values of the `pub const <NAME>: &str = "..."`
  declarations under `src/checks/`.
- Whether a required family is read per source or once for the whole workspace is not in the
  table, so it was taken from the reads themselves: a whole-workspace fact is required at
  `nomos_model::Subject_Of_Path("")` and a per-source fact at `&source.subject`, and grepping
  both partitions the fourteen families cleanly.
- Shared-engine use is the call-site count of each engine under `src/`, excluding its own
  declaration and every `tests.rs`.
- Judgment locality — whether a rule judges one subject or relates several — is not mechanical
  in general, so all seventy-one entry-point bodies were read. A detector for the mechanical
  half of it (an entry that builds an index or a tree over all `sources` before judging) agrees
  with that reading exactly and names the same four rules, which is why the reading is reported
  rather than the detector.

One measurement was attempted and discarded rather than reported: a transitive call cone per
rule, to size each judgment. Shared helper names (`Payload_Of`, `Violations_In`, `Findings_In`)
occur in several modules, so a name-keyed cone attributes one module's body to another rule and
sums to twice the crate. Nothing below depends on it.

## The census

**The real total is seventy-one, not seventy, and the exported population is eighty-seven.**
`DESCRIPTORS` names seventy-one rules. The crate exports eighty-seven `Check_*` functions under
eighty-six distinct rule identifiers — the one function with no identifier of its own is
`Check_Function_Arity_Policy`, a shared engine two identified rules call. The sixteen exported
functions the table does not name are not a gap: each carries a stated, measured reason in
`ACCOUNTED_FOR`, which `Test_Every_Exported_Rule_Is_Composed_Or_Accounted_For` checks in both
directions. Two doc comments inside the crate still say sixty-nine and seventy; they are stale,
and no test compares either against the table.

**Subject kind.** Thirty-eight rules declare `SubjectKind::SourceFacts`, thirty-one
`SubjectKind::SourceText`, two `SubjectKind::Workspace`.

**Fact arity is small and has a ceiling of three.** Thirty-one rules require no materialized
family at all, twenty-six require one, twelve require two, and two require three. No rule
requires four. `Test_A_Source_Text_Rule_Should_Require_No_Fact` and
`Test_A_Fact_Reading_Rule_Should_Require_At_Least_One_Fact` already keep the zero-and-nonzero
ends honest.

**Fourteen families are declared and the distribution is long-tailed.** `SyntaxItems` is
required by seventeen rules and `TestMaterialPolicy` by fourteen; `LimitsPolicy` by six,
`NamingPolicy` by five, `DependencyEdges` and `ArchitectureDeclaration` by three each. The
remaining eight families — `LintDiagnostics`, `DependencyPolicy`, `Reachability`, `GoalsPolicy`,
`WordsPolicy`, `ScriptingPolicy`, `ReviewFindings`, `RequirementTrace` — are each required by
exactly one rule.

**Subject material splits three ways, and the raw-text half is the largest.** Partitioning the
families by how they are read: six are per-source material (`SyntaxItems`, `DependencyEdges`,
`LintDiagnostics`, `Reachability`, `ReviewFindings`, `DependencyPolicy`) and eight are read once
for the whole workspace. Twenty-four rules read at least one per-source material fact and judge
a decoded payload. Two judge one whole-workspace payload and read no source. The other
forty-five take one file's raw text as their subject material — thirty-one requiring nothing,
fourteen requiring only a whole-workspace policy fact that parameterizes a text scan rather than
supplying its material.

**Sixty-five of seventy-one are a predicate over one subject.** Four relate several:
`COMPLETENESS_MIRROR` and `GUARANTEE_DECLARES_ITS_EXERCISER` build a workspace-wide name set
before judging any item against it, `CROSS_LANGUAGE_CORRESPONDENCE` resolves a declared
correspondence against an index of every file's structs, and `NO_ORPHAN_MODULES` walks a module
tree outward from roots. Two are the whole-workspace payload rules. The remaining sixty-five —
forty-four over raw text, twenty-one over a per-source payload — judge each file against a norm
and never against another file. That includes the three dependency rules, which read one
workspace-wide architecture declaration and then judge each member against it: a per-subject
predicate with a declared parameter, not a relation between subjects.

**Thirty of seventy-one take a parameter a repository declares.** That is the count of rules
requiring at least one of the eight policy families `OD-RULES-011` made reachable; forty-one
take none. Within the forty-five raw-text rules the split is thirty-one compiled-in against
fourteen declared. The engines that resolve the declared half are already shared and already
few: `Resolve_Limit` has eight call sites, `Resolve_Case` seven,
`Resolve_Declared_Fixture_Locations` twelve.

**The dominant archetype is already served by parameterized engines, written by extraction
rather than by design.** Counted as call sites under `src/`: `Finding_For_Line` 22,
`Code_Prefix` 21, `Resolve_Declared_Fixture_Locations` 12, `Resolve_Limit` 8,
`Unjustified_Construct_Findings_In` 7, `Resolve_Case` 7, `Finding_For_Source` 7,
`Judged_Sources` 5, `For_Each_Line_Number` 5, `Relay_Findings` 4, `Findings_For_Threshold` 4,
`Judged_Members` 3, and four more at two each. Each was extracted after the duplication was
real and reported: `finding_shape`'s own doc names ten rule modules that built the same finding
by hand, `code_prefix`'s names eleven copies of one line split, `declaration_scan`'s names two
rules driving the same brace-delimited scan, and `Unjustified_Construct_Findings_In`'s names the
six functions it replaced and states the shape in one sentence — "a construct-matching
predicate, an unless-locally-justified predicate, and one finding message."

**The gate is uniform and two-thirds compiled in.** Twenty-three entry-point bodies gate on
`SourceFile::Is_Written_In`, thirteen on a self-exemption. The self-exemption is nine
independent predicates over nine private path literals, one per module, and `closure_bounds`'s
own doc says why it is not a policy fact: a rule's detector constants spell out the syntax the
rule looks for, and "no repository declaration could or should make that judgeable."

**Contract authority, re-measured.** Fifty-nine rules cite `PORTED_STANDARD`, eleven cite a
versioned governing record, and one cites `WORKSPACE_CONVENTIONS`.
`Test_Every_Descriptor_Should_Cite_A_Real_Authority` keeps the pairing honest.

## Why OD-PACKAGE-008 is the right starting point and does not answer this

`OD-PACKAGE-008` did the field-by-field comparison this record's method imitates — it read four
real rules against `ARCH-002`'s contents list rather than generalizing from one — and its
conclusions about `identity/version`, `required canonical capabilities` and `applicability
semantics` are the reason `nomos-rule-package` has the shape it has. It does not answer this
question, and the reason is structural rather than a matter of age. Every field it measured is
*metadata about* a judgment: who decides the rule, at what version, against which capabilities,
with what evidence class, how completely. None of them is the judgment. `RulePackage` carries a
`Judgment` that says whether a linked implementation exists, and nothing anywhere in that
manifest says what one does. An authoring surface is a decision about that missing thing, so no
amount of re-reading a manifest schema produces it.

One of its numbers has moved far enough to be worth writing down where a later reader will find
it. At version 4 it read contract citation as 3-of-4 and called three independent rules
converging on the same citation shape "materially more evidence for that field's stability". At
seventy-one it is 11-of-71: the versioned citation is the exception and the ported standard is
the rule. That is an observation for a holder of `OD-PACKAGE-008`, not an amendment here.

## The decision

**A declarative authoring form is owed, and it is owed for exactly one archetype: a per-line
predicate over one file's raw text.** Forty-four rules are that shape, sixty-two per cent of the
table and the largest single group by every axis above. They share a gate, a traversal, a
finding constructor and — for seven of them already — a whole engine. Authoring one today means
writing a function whose body is the same fifteen lines the last one had, and this crate's own
history is a record of noticing that four separate times and extracting a helper each time. The
form is the fifth extraction, taken one level further: instead of a function that calls the
engine, a declaration the engine reads.

**Its inputs are read off the engines that already exist, not invented.** A declared text rule
states:

1. its **rule identifier** and its **contract citation**, the two fields the descriptor already
   carries and `Test_Every_Descriptor_Should_Cite_A_Real_Authority` already checks;
2. its **language**, naming what `SourceFile::Is_Written_In` compares, or none;
3. its **test-material sensitivity** — whether `Is_Test_Or_Example_Source` excludes a subject,
   resolved from the repository-declared locations `Resolve_Declared_Fixture_Locations` reads,
   per `OD-RULES-011`;
4. its **self-exemption**, the implementation files whose own text necessarily spells out what
   the rule looks for, stated as a field because nine rules already carry one privately and it
   cannot be derived;
5. its **detector**, named from a closed vocabulary the interpreter owns, never written in the
   declaration;
6. its **justification clause**, optional, naming the local-justification detector the same way
   — this is the half `Unjustified_Construct_Findings_In` already takes;
7. its **message**, the one sentence the finding carries;
8. its **parameter**, optional, naming a policy key and a default, which is what `Resolve_Limit`
   and `Resolve_Case` already take.

Eight fields, and every one of them is a parameter some engine in the crate takes today. That
is the test this record applies to the whole proposal: a field with no existing caller would be
a field measured against nothing.

## What the form may not express

A form that silently cannot say something is worse than no form, so each of these is a refusal
the interpreter states rather than a case it renders empty.

- **The detector itself.** This is the important one. The forty-four detectors are `fn(&str) ->
  bool` over a comment-and-literal-aware prefix of one line, and they are not one shape:
  `Has_Unsafe_Construct` is a substring test, `Credential_Match_In` returns the text it matched
  so the summary can quote it, `Bounded_Fields_In` parses a declared numeric range out of a
  documentation comment, and `Narrowest_That_Holds` computes integer widths. A declaration names
  one of them; it does not contain one. A rule needing a detector the vocabulary does not have
  is a Rust change, and the form says so rather than approximating.
- **A relation between subjects.** The four cross-subject rules are out of scope by
  construction, and a form offering a per-file loop would render a mirror rule as a rule that
  finds nothing — the exact failure this workspace already ranks `Blocking` when a mirror claim
  resolves to nothing.
- **A judgment over a materialized payload.** The twenty-one per-source payload rules are also
  out of scope, and deliberately so even though they look adjacent: their recurring families are
  already parameterized end to end — six casing rules through `Resolve_Case` and one `Case`
  vocabulary, two arity rules through `FunctionArityPolicy` — so a declared form over them would
  re-parameterize what `OD-RULES-011` already parameterized, and buy a second way to say it.
- **Block or scope context.** `NESTING_DEPTH` and `NAMED_FIELDS_OVER_POSITIONAL_VARIANT_PAYLOADS`
  carry brace state across lines. The form's traversal is one line at a time plus the
  justification look-back the engine already does, and nothing wider.
- **Anything about which rules run.** Selection is `OD-HOST-004`'s question and `OD-RULES-022`'s
  resolution step, and a form that declared its own applicability would be a second answer to it.

## Where it sits

**`RuleJudgment` is the seam, and there is no second table.** It is a newtype over a `fn`
pointer today because `DESCRIPTORS` is a `const` and a `fn` pointer is the only callable a
`const` can hold — `OD-RULES-027` decided that, and it is what lets the run derive its table
instead of writing a second copy. A declared form carries data, which a bare `fn` pointer
cannot, so the temptation is a separate list of declared rules beside the table. That is exactly
the accretion `OD-GATE-020` measured going silently out of step twice, and this record refuses
it.

`RuleJudgment` becomes two arms instead of one — a linked `fn` pointer, or a `&'static`
reference to a declared form — and both are `const`-constructible, so the table stays a `const`
table and `DESCRIPTORS` stays the whole declaration of a rule. `RuleJudgment::Judges` keeps its
signature, which matters more than it looks: it is called from exactly one place in the whole
workspace, `run_context/judging.rs`, so the run needs no change at all.

Nothing in `nomos-rule-package` changes either, and that is a conclusion rather than an
omission. `Judgment::Mechanical` means a linked implementation decides the rule and resolution
refuses the package if none is registered; a declared form interpreted by a linked interpreter
is precisely that, so a declared rule is mechanical and `declared_rules.rs`, which already
derives every `RulePackage` from `DESCRIPTORS`, keeps deriving them correctly with no new arm.

**The form decided here is compile-time, and the manifest-loaded one is not decided.** A
declaration a repository ships and this build loads at run time cannot be `&'static`, so it
would make the composed rule set a run-time construction rather than a `const` — a real change
to `OD-RULES-027`'s decision, and one nothing in this population demands, because every rule
measured above is one this workspace itself ships. The observation that would decide it is a
repository outside this workspace needing a rule this workspace does not ship, which is a fact
about a consumer and not about the table.

## The three things this record was told not to propose

**A visual editor is premature, and it is not on this path at all.** The surface decided above
has eight fields, every one of them a scalar or a name from a closed vocabulary, and there is
one author. A form-filling interface over eight fields is worth less than the literal it would
produce, and it would be a second authoring path beside the one this record names — the thing
the seam above exists to prevent. The observation that would change it is not a rule count: it
is somebody authoring rules who cannot read the repository's own declarations.

**A general expression language is premature, and the census says precisely why.** It would be
the answer to the detector gap, and the detector gap is real: naming a detector from a closed
vocabulary means a rule needing a new one still costs Rust. But the four detectors quoted above
are a substring test, a matcher that returns its match, a documentation-comment parser and an
integer-width computation, and an expression language general enough to hold the last two is a
programming language. One that held only the first would let a declaration be written for the
other three and silently fail to mean them, which is the failure mode this record forbids by
name. A closed vocabulary that refuses is the honest version of the same offer. The observation
that would change it is two or more new rules whose detectors differ only in a literal the
vocabulary cannot parameterize.

**A rule intermediate representation stays premature, and `OD-RULES-007` stays open.** That
record named six triggers at a population of four and asked to be re-measured rather than
reaffirmed by count. Re-measured at seventy-one: five do not fire. The sixth — "a reusable
subworkflow or procedure two or more rules genuinely share, the way `nomos-proto`'s sixteen
duplicated driver loops did before extraction" — fires in its observation and not in its
conclusion, and the distinction is the whole of this record's argument. Seven rules share
`Unjustified_Construct_Findings_In`; four share `Findings_For_Threshold`; ten rule modules built
`finding_shape` by hand before it existed; eleven copies of one line split preceded
`code_prefix`. Every one of those extractions landed as an ordinary Rust helper inside one
crate, and no planner needs to see any of them: the redundancy each removed was in *authoring*,
not in fact acquisition or in judgment scheduling. A declared form removes authoring redundancy
without making judgment structure visible to anything that schedules; an intermediate
representation is defined by doing the second. So the trigger that fired argues for this
record's decision and not for that record's, and `OD-RULES-007`'s own six conditions are left
where they are.

## What this record does not do

It does not build the form, the interpreter, the detector vocabulary or the second
`RuleJudgment` arm. It does not convert any existing rule.

It does not amend `OD-RULES-007` or `OD-PACKAGE-008`. Both are re-measured against the
population above and neither is edited, because amending a record is the work of an item that
holds it.

It does not decide the detector vocabulary's membership. Which detectors a declaration may name
is a list to be read off the forty-four rules when one is built, not a list to guess at here.

It does not repair the two stale rule counts in `nomos-rules`' own doc comments, and does not
propose a test that would compare them against the table.

## The item that would build it

`P129-A-DECLARED-TEXT-RULE-FORM-IS-A-SECOND-JUDGMENT-ARM-IN-THE-ONE-DESCRIPTOR-TABLE`.

Its `done_when` is that one existing rule already on `Unjustified_Construct_Findings_In` is
re-declared through the form and produces byte-identical findings against this workspace's own
tree, that the declaration names a detector from the vocabulary and cannot express one inline,
and that `DESCRIPTORS` remains a `const` table with no second list beside it. The identical-
findings comparison is the falsifier: a form that changed what a rule reports would be a new
rule wearing an old identifier.

Territory, by path:

- `crates/rules/nomos-rules/src/rule_descriptor/rule_judgment.rs`
- `crates/rules/nomos-rules/src/rule_descriptor/declared_form.rs`
- `crates/rules/nomos-rules/src/rule_descriptor.rs`
- `crates/rules/nomos-rules/src/checks/rust_text.rs`
- `crates/rules/nomos-rules/src/checks.rs`
- `crates/rules/nomos-rules/src/lib.rs`
- `tests/contract/surface/nomos-rules.txt`

The surface snapshot is territory because `RuleJudgment` is a public type and a second arm moves
it. `crates/orchestration/nomos-check-orchestration` is deliberately not territory: the run
calls a rule through `RuleJudgment::Judges` at one site, and an item that needed to widen into
that crate would be evidence this seam was chosen wrongly.

## Status

Accepted.

Revisit if the first declared rule cannot state something its function stated — which would mean
the eight fields were read off the engines and still generalized past what forty-four rules
show, rather than a reason to add a ninth.
