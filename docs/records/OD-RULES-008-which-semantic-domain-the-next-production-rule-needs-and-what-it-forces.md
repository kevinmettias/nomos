---
id: OD-RULES-008
type: decision
title: Which semantic domain the next production rule needs, and which deferred architecture piece it actually forces
status: open
version: 1
authority: canonical-normative-record
tags:
  - rules
  - capability
  - analysis
  - architecture
relations:
  - target: OD-RULES-007
    type: relates-to
  - target: OD-ANALYSIS-007
    type: relates-to
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: OD-CAPABILITY-008
    type: relates-to
  - target: OD-RULES-005
    type: relates-to
  - target: OD-GATE-014
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: D-135
    type: relates-to
---

# Which semantic domain the next production rule needs, and which deferred architecture piece it actually forces

## Question

The board went quiescent after `P13-LEDGER-GAPS-3`: every item is `Done` or `Declined`,
and `nomos work list` reports `next: nothing is eligible`. The three shipped rules --
`Check_Naming_Convention`, `Check_Completeness_Mirrors`, `Check_Dependency_Direction` --
occupy three narrow domains: syntax-local per-item judgment, cross-artifact consistency
over already-decoded fact text, and package-dependency graph reachability. None reads a
program's control flow, its call structure, or a resolved name or type. `OD-RULES-005`,
`OD-RULES-006`, `OD-RULES-007`, `OD-GATE-014` and `OD-CAPABILITY-008` each left a deferred
abstraction open rather than built, and each named its own concrete trigger -- a second
rule, a third provider, two rules exhibiting a shared shape, a caller needing to select
less than everything. Re-litigating any of those from a taxonomy would repeat the mistake
`D-135` already names: inferring genericity from a wish rather than a demonstrated need.

The board's next honest move is not another pass over those open questions from the
outside. It is a concrete next rule, real enough that its own requirements either trip one
of those triggers or plainly do not -- decided from what the rule needs, not from what
would make a tidier architecture. This record names one candidate, drawn from a defect
this workspace's own three shipped rules already exhibit, independently, three times, and
follows it through every deferred question it touches.

## The candidate: `Check_Unread_Reaches_A_Finding`

**Informal statement.** Every control-flow path that begins at a fact-read failure --
`FactReader::Require` returning `Err(applicability)` -- must reach a `Finding`
construction, directly or by propagation, before the enclosing function returns. A path
that instead falls through, returns an empty collection, or otherwise continues as though
the read had succeeded is the exact defect `Applicability`'s own module doc names as this
product's first principle: "unknown is not pass."

**Why this one, and not an invented example.** The convention this rule would mechanize is
not hypothetical -- it is already hand-written, independently, three times, with nothing
checking that any of the three actually holds it:

- `crates/rules/nomos-rules/src/naming.rs`, `Payload_Of` (line 78): matches
  `facts.Require(...)`'s `Err(applicability)` arm and returns `Err(Unread(...))`.
- `crates/rules/nomos-rules/src/mirror/index.rs` (line 74 on): the identical shape, its
  own private `Unread_Of`.
- `crates/rules/nomos-rules/src/dependency.rs`, `Payload_Of` (line 168): the identical
  shape again, its own private `Unread`.

Three rule authors, working on three different capabilities, each reached for the same
pattern and each wrote their own copy of the function that closes it, because nothing
states the obligation once and checks it. That is the same shape `nomos-cap-syntax`'s own
module doc describes for why a contract needed a home of its own: "nothing would have
noticed the day one of them was retyped differently." Here nothing would notice the day
one of them forgot the `Err` arm entirely -- `Violations_In`, `Check_Names_In` and every
sibling judgment function stay exactly as green if a future edit to any of these three
`Payload_Of`s drops its error path, because nothing but a human re-reading the diff holds
the convention.

**Canonical subject.** Not a file and not a package, the grain every rule to date reads.
The subject is a control-flow edge inside one function body: the continuation of an `Err`
arm (or an equivalent early-exit branch) reached from a fact-read call. This is a subject
grain finer than anything a `SubjectId` names in this workspace today.
`IncrementalGranularity::Region` -- "a sub-symbol region refreshes independently" -- is
already declared in `crates/contracts/nomos-contracts/src/guarantee/incremental_granularity.rs`
and has never been used by a real provider; this candidate's subject is the first real
shape that grain was named for.

**Semantic domain.** Intraprocedural control flow / error-flow reachability: whether a
`Finding` is constructed on every path forward from a given point, not what the source
declares or what package depends on what.

**Logical form.** A per-path reachability obligation -- "on every path forward from S,
some node before the function's exit constructs a `Finding`" -- not the flat
per-subject decode-and-compare predicate all three shipped rules share.
`Check_Dependency_Direction` is, in `OD-RULES-007`'s own words, "a plain per-source,
per-edge loop over an already-decoded fact." This candidate cannot be phrased that way: it
is a graph traversal over a function body's own branches, not a loop over a list a
provider already flattened.

**The exact facts it would consume, checked against what exists today, not assumed.**
`crates/capabilities/nomos-cap-syntax` is checked directly: its payload is `header
item*`, one record per top-level declaration (`crates/capabilities/nomos-cap-syntax/src/payload.rs`'s
grammar), carrying `ordinal`, `kind`, `visibility`, `qualified_name`, `documentation` and
`shape` -- and `shape` itself, for a function, is only `fn/<arity>`, the parameter count.
There is no statement, expression, branch or call-site record anywhere in this schema.
`nomos.cap.dependency.edges` carries package-level edges, not function bodies. Neither
existing capability can answer this candidate's question at any guarantee level; the fact
it needs -- an intraprocedural control-flow graph, with fact-read calls and
`Finding`-construction sites identified within it -- does not exist under any capability
this workspace ships.

**Examined / unsupported / insufficient evidence / not applicable**, in this workspace's
own vocabulary (`Applicability`, `crates/contracts/nomos-contracts/src/finding/applicability.rs`):

- `NotApplicable` -- the function contains no fact-read call (no `Require` call whose
  result is matched or propagated) at all.
- `Supported` -- a control-flow provider resolved every path from every `Err` arm found
  and confirmed a `Finding` reaches the function's exit on each.
- `PartiallySupported` -- some paths in the function were resolved and some were not (a
  macro-expanded arm, for instance -- `nomos-cap-syntax`'s own header record already
  carries "a lower bound on unexpanded tokens" for exactly this reason).
- `AgentRequired` -- the path forwards into a call the provider cannot resolve
  statically: a call through `dyn FactReader` itself (the very trait this rule's own
  sources call through), a stored closure invoked elsewhere, or any other dynamic
  dispatch. This is not `MissingCapability` or `ProviderUnavailable`; a provider is
  present and ran, and the honest answer is that no mechanical method decides the
  question for this specific path.
- `MissingCapability` / `ProviderUnavailable` -- no control-flow provider is installed,
  or one is installed and could not run, the same two-state split
  `Materialize_Dependencies` already draws for a `cargo metadata` failure.

**Whether deterministic enforcement is possible.** In two tiers, not one:

1. A heuristic, `Syntactic`-level version can flag the syntactically obvious cases --
   an `Err` arm with an empty body, or one whose tail expression is plainly `Ok(...)`,
   `continue`, or a bare `return` with no `Finding`-shaped value in sight -- by pattern
   matching the AST alone, no name or type resolution required. This would already catch
   the shape of defect a careless edit to any of the three existing `Payload_Of`s could
   introduce.
2. A sound version -- one that does not merely fail to flag a defect it happened not to
   recognise -- requires resolving every call the `Err` arm reaches, including into
   helper functions elsewhere in the crate, to confirm each actually constructs or
   propagates a `Finding` rather than merely being named as though it does (`Unread`,
   `Unread_Of` and `Unreadable` are three different functions with the same implied
   contract and no shared type enforcing it). That resolution is
   `FactVariant::SemanticallyResolved`'s own obligation as `OD-ANALYSIS-004` fixed it:
   "resolved every name occurrence in its subject to the declaration it actually binds."
   Tier 1 is a lint; tier 2 is the actual claim this rule states, and only tier 2 is
   honestly reportable as `Applicability::Supported` rather than
   `Applicability::SupportedWithFallback`.

Neither tier exists in this workspace today. Building either is not this record's
territory -- this record specifies the rule and traces what building it would force, and
stops there.

**Whether correction is meaningful.** Mostly not. `nomos-corrections`' own crate doc states
its lifecycle runs "no agent and no model backend anywhere," every step a pure function of
an already-decided plan. The fix for a missing `Finding` on an `Err` path is "write a
`Finding` that correctly describes this specific failure" -- a judgment about wording and
severity, not a deterministic edit `nomos-corrections` could stage. The one exception is
also the weakest claim: a syntactically empty arm (`Err(_) => {}`) is deterministically
*wrong*, but the correction available for it is still only "flag it," a `Finding`, not a
supplied fix.

## What this candidate forces, checked against each open deferred question in turn

**`OD-ANALYSIS-007` -- the first program-semantics capability.** This is the one question
this candidate actually moves. `OD-ANALYSIS-007`'s own "What Would Decide It" section
names its trigger exactly: "a rule ... that needs a program-semantics fact to reach a
verdict it cannot reach at `Syntactic` or `Approximate` today. That rule's own subject
would name which of `OD-ANALYSIS-004`'s five shapes ... is the real one to build." This
candidate's sound tier is precisely that: a claim that a `Finding`-construction effect
does not occur on a path this repository's own convention (`Applicability`'s "unknown is
not pass") forbids the absence of -- `OD-ANALYSIS-004`'s "Effects" shape by name ("a claim
that an effect occurs from code this repository's own ... policy restricts it from
performing," read in the negative). `OD-ANALYSIS-007` also states, as of its own writing,
"no rule in this workspace has ever asked for `SemanticallyResolved` ... evidence" --
true when it was written, against a population of two rules, and no longer precisely true
today: `Check_Dependency_Direction`'s own `Dependency_Requirement` (`dependency.rs`, line
124) already asks for `FactVariant::SemanticallyResolved`, at `IncrementalGranularity::Project`.
That existing instance is resolution of Cargo's own package graph, not of program code --
a materially different sense of "semantically resolved" than `OD-ANALYSIS-004`'s five
illustrative shapes name, all of which are about a *program's* resolved names, types and
control flow. So `OD-ANALYSIS-007`'s premise needs a narrower restatement -- a real rule
has asked for `SemanticallyResolved` build-graph evidence, but none has yet asked for
`SemanticallyResolved` program-code evidence in `OD-ANALYSIS-004`'s sense -- and this
candidate is the first that would. Correcting `OD-ANALYSIS-007`'s own text is outside this
record's territory; noting the distinction here is not.

`OD-ANALYSIS-007` is not therefore closed by this record. It is narrowed the same way
`OD-CAPABILITY-008` narrowed its own question on its third provider landing: this
candidate is a real, specified subject naming a real, undecided shape (an
intraprocedural control-flow / effects capability), which is what that record's own
trigger asked for. Building it is separate work reserving its own territory, per
`OD-ANALYSIS-004`'s "What This Record Does Not Do."

**`OD-RULES-007` -- a Rule Program/IR.** Not forced. The candidate's own judgment
function stays the same flat shape every shipped rule already has: a loop over subjects,
reading one already-materialized fact per subject and comparing it. The graph traversal
this candidate needs happens once, inside the new capability's own provider, building the
control-flow fact -- ordinary capability work, the same place `Check_Dependency_Direction`'s
own graph-shaped fact (`DependencyPayload`'s edges) is built, not inside rule execution.
`OD-RULES-007`'s own three-way split already places "new capability families" in the tier
that is allowed to emerge from a real rule, separately from a Rule Program/IR; this
candidate is exactly that tier and nothing else.

**`OD-CAPABILITY-008` -- a declared provider trait.** Not forced, and not moved either
way. A control-flow provider would be the first (not a third or fourth) implementation of
a wholly new capability, not another instance of `nomos.cap.syntax.items` or
`nomos.cap.dependency.edges`. It adds no population to the question `OD-CAPABILITY-008`
is actually asking, which is whether *multiple providers of the same capability* need a
shared trait.

**`OD-RULES-005` -- a measured `Inputs` classification.** Not forced. This candidate reads
one function body's already-walked source, the same "scoped files" shape
`Check_Naming_Convention` and `Check_Completeness_Mirrors` already have -- it names no
whole-corpus or external-tool dependency `OD-RULES-005`'s open question is about.

**`ARC-ROADMAP-001`'s shared demand planner.** Not forced, on the workspace's own already-
stated reasoning, not a fresh judgment call. `Materialize_Dependencies`'s own doc comment
(`crates/orchestration/nomos-check-orchestration/src/facts.rs`, line 119) already answers
this for a second hardcoded materialization step: "composing this as one more hardcoded
step is `OD-HOST-004`'s 'composition, not choice' again, not a case for the shared demand
planner `ARC-ROADMAP-001` still leaves for later." A third hardcoded step for this
candidate's control-flow fact would be the identical shape a third time: `Run` (`crates/orchestration/nomos-check-orchestration/src/run.rs`)
still has no selector of any kind (`OD-GATE-014`, still open), so nothing is asking to
materialize *less* than everything -- the planner's trigger is selection creating unread
demand, not the count of hardcoded steps rising. Three unconditional steps cost more than
two the same way two cost more than one, and that growing cost is worth naming, but it is
not, by itself, the trigger `OD-GATE-014` and `ARC-ROADMAP-001` already require.

**Architecture-core facts, feature topology, richer applicability beyond what
`Applicability` already states.** Not forced. This candidate's subject is one function
body; it makes no claim about cross-module architecture, feature ownership, or a coverage
model `Applicability`'s existing ten variants cannot already state -- `AgentRequired`
already exists for exactly the tail this candidate would be the first rule ever to use.
Checked directly: no rule under `crates/rules` or `crates/orchestration` constructs
`Applicability::AgentRequired` today.

## Status

Open, narrowed rather than resolved, the same standing `OD-CAPABILITY-008` records for
its own question. `OD-ANALYSIS-007`'s trigger -- a rule naming the program-semantics fact
it needs -- is the one this record answers in the affirmative: `Check_Unread_Reaches_A_Finding`
is a real, specified subject whose sound form needs an intraprocedural control-flow /
effects capability at `FactVariant::SemanticallyResolved`, a shape `OD-ANALYSIS-004`
already names and a producer for which does not exist. The concrete next capability item,
should one be claimed, is that provider -- `nomos.cap.controlflow.reachability` or
equivalent, scoped to intraprocedural reachability from a fact-read failure to a
`Finding`-construction site -- with `Check_Unread_Reaches_A_Finding` as its first and, at
first, only consumer. Every other deferred question this record checked -- a Rule
Program/IR, a declared provider trait, a measured `Inputs` classification, the shared
demand planner, architecture-core facts and feature topology -- stays exactly as
undecided as its own record already left it; this candidate's real requirements do not
reach any of them. Revisit `OD-ANALYSIS-007` when a session claims the capability this
record names, or names a different one this record did not anticipate.
