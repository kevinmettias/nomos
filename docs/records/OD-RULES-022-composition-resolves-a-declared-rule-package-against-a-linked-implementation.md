---
id: OD-RULES-022
type: decision
title: Composition resolves a declared rule package against a linked implementation, and OD-HOST-004's trigger has already fired
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - packages
  - orchestration
  - architecture
relations:
  - target: OD-HOST-004
    type: relates-to
  - target: OD-RULES-004
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-GATE-017
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
---

# Composition resolves a declared rule package against a linked implementation, and OD-HOST-004's trigger has already fired

## Question

`OD-HOST-004` decided that `Run()` stays hand-written "for as long as every rule it calls runs
unconditionally, on every invocation", and did not leave that open-ended. It named the exact
event that ends it: "`Run` flips the first time a rule (or a group of rules) is meant to run for
*some* check invocations and not others — selected by something the request states, rather than
by whether the crate happens to know about it." It went further and said what must not happen at
that moment: "adding the choice as a hand-written `if`/`match` naming rules by branch is exactly
the accretion this record exists to head off."

`OD-RULES-004` then split the question in two, and its split is the one this record works
inside. Registration — how a rule becomes a nameable thing at all — was answered with a
`RuleId`-carrying offer type and a registry. Selection — which rules run on this invocation —
was left to `OD-HOST-004`, with one sentence pointing forward: "The moment a registered rule's
participation is meant to vary by request — the criterion `OD-HOST-004` already names — `Run()`
still needs the selection mechanism that record describes, and this record does not build it."

That moment has passed, unremarked. The question is what composition should be now that it has,
and whether the manifest format this workspace already built for exactly this content is the
answer or a coincidence.

## What Was Measured

Every count below was taken from the tree at `e9b9363f` on 2026-09-05.

**The trigger fired, and the answer given at the time was a filter over an array.**
`nomos_check_orchestration::Run` takes `selected: &[RuleId]`. `OD-GATE-017` made a non-empty
selection narrow what `Run` computes at all rather than merely filtering its output, and
`nomos gate run --rule <id>` reaches it from a command line. A rule's participation now varies by
request, which is `OD-HOST-004`'s stated criterion word for word. What it did not get is the
declared selection mechanism that record asked for. `run_context.rs` carries
`const RULE_COUNT: usize = 57` and builds a fixed `[ComposedRule; RULE_COUNT]` array, and its own
doc describes the structure beside it as "a fixed, hand-written mapping from `RuleId` to the
fact(s) it needs". A declaration doing registry work inside a function body is the shape
`OD-HOST-004` refused; it arrived as a table rather than as a `match`, which is why nothing
caught it.

**Composing one rule costs five registration sites in three crates, and three of them are
hand-maintained tables that must agree.** Measured against `no-orphan-modules`, the most recent
rule to land: `nomos-rules/src/checks.rs` declares the module, `nomos-rules/src/lib.rs`
re-exports the function and its `RuleId` constant, `nomos-rules/src/rule_descriptor.rs` adds a
`DESCRIPTORS` row, `nomos-check-orchestration/src/run_context.rs` raises `RULE_COUNT` and adds
both a `ComposedRule` entry and a row to the capability mapping, and
`nomos-gate-orchestration/src/composition.rs` adds an `OFFERINGS` row carrying the contract
citation. The first two are ordinary Rust plumbing. The last three are declarations of the same
rule, written three times, in three crates, that nothing but a test holds together.

**The agreement is enforced by tests, and it broke while this record was being written.** At
`e9b9363f` both `Test_Registered_Should_Offer_Every_Composed_Rule` and
`Test_Registered_Should_Compose_Every_Rule_A_Check_Run_Composes` failed: `no-orphan-modules` was
added to the composed array without an `OFFERINGS` row, so `Run` judged by a rule the registry
did not offer and `nomos gate plan` described a smaller gate than `nomos gate run` performed. It
was fixed one commit later by `8be7904a`, whose own message names the cause exactly: the item's
verification predicate covered four of the five sites, so `work finish` exited zero over a tree
where the fifth was never built.

That sequence is the argument for this record, not an anecdote beside it. The tests did their
job. What a test cannot do is stop the drift, because three declarations of one rule that must
be edited together will be edited apart, and a predicate is scoped to the crates an author
remembers. This is not evidence of carelessness — it is what a duplicated declaration costs at
57 entries, found within hours of reaching them.

**The gap between exported and composed is invisible for the same reason.** 75 exported against
57 composed leaves 18 rules that are built, tested, documented and exported, and judge nothing.
`P46-UNCOMPOSED-RULES-ARE-COUNTED` had to establish that number by hand-diffing two lists,
because there is no declaration for the array to be compared against.

**The format that carries all of this already exists and has no consumer.**
`nomos-rule-package` parses a manifest whose fields are `rule_id`, `contract`,
`required_capabilities`, `applicability`, `evidence_schema`, `title`, plus the correction,
diagnostic-mapping, fixture and agent-guidance fields `ARCH-002` names. Those are, field for
field, what the three tables above hold between them. `OD-PACKAGE-008` measured the format
against four real rules rather than inventing it from corpus text. Grepped at `e9b9363f`, no
crate in this workspace depends on it: its only appearances outside its own directory are two
band-table rows and one doc comment in `nomos-contracts`.

**The population this is aimed at is not 57.** The Go predecessor
(`github.com/kevinmettias/nomos-proto`, checked out as `code-standards`) declares 1,143 rules —
1,127 in `kernel/rules/corpus` and 16 in `metacorpus`. Its own `rulespec/rule.go` states that
728 of them are decided by a model rather than by a parser. A hand-written array with a parallel
hand-written contract table is not a structure that holds either number.

## The Decision

**Rule composition resolves a declared rule package against a linked implementation.** The
declaration and the implementation are different artifacts owned by different layers, and
composition's job is to match them and to refuse when it cannot.

A **declaration** is a `RulePackage`: identity, contract citation, required capabilities,
applicability semantics, evidence class, presentation metadata, and the judgment clause below.
It is data. It is what `nomos gate plan` reports, what a registry offers, and what a future
projection renders into documentation — the one canonical definition the migration plan calls for.

An **implementation** is linked Rust code: a function of the shape `nomos-rules` already
establishes. A manifest cannot conjure a function, and this record does not pretend otherwise.
This is the same division `KnownProviders` already draws for providers — a package names a
`ProviderId`, the provider crate exports its own `PROVIDER` constant, and the allowlist is
checked rather than trusted — and it is why that shape is the precedent here rather than an
analogy.

**A declaration states its own judgment, and that is what makes the resolution total.** A
declaration is either *mechanical*, in which case an implementation must exist and resolution
fails if it does not, or *model-judged*, in which case there is no implementation to resolve and
the rule contributes nothing to a deterministic run. Without this clause the resolution rule
would have to refuse every one of the predecessor's 728 model-judged rules or admit them as
implementations that silently judge clean, and `OD-GATE-001` already names the second of those as
the defect the gate exists to prevent. A model-judged declaration is a rule this system states
and does not yet enforce, which is a truthful thing to be and a different thing from a rule that
ran and found nothing.

**Resolution is refused in both directions.** A declaration naming no implementation, when it
claims to be mechanical, is a rule the gate would report and cannot perform. An implementation
with no declaration is the 18 exported-but-uncomposed rules, which read to any reader of the
crate as though they are in force. Neither is a state composition may enter silently, and a
refusal names which of the two it found.

**This does not weaken `OD-RULES-004`'s split; it completes it.** Registration answered how a
rule becomes nameable and built the registry to hold it. This answers what the registry's
entries are made of and where they come from, which that record explicitly left to a follow-on.
Selection stays exactly what `OD-HOST-004` described — a request-carried property resolved
against a declaration — and gains the declaration it was missing.

## What This Record Does Not Do

It does not build any of it. The manifest crate, the resolution step, the refusal type, the
migration of the 57 composed rules from array rows to declarations, and the judgment field
`nomos-rule-package` does not yet carry are each their own item, and each has to keep the gate
green while it lands.

It does not group rules into packages. The plan names aggregates like `nomos.rules.core-naming`
and `nomos.rules.standard-engineering`, and `nomos-rule-package`'s shipped shape is one rule per
package (`rule_id`, singular). Which of those is right is a question with a real answer that this
record does not have, because nothing has yet needed a group.

It does not amend `OD-HOST-004`. That record still reads as though its trigger is ahead of it,
and correcting a record whose stated condition has passed is the work of an item that holds it —
the same shape `P43-CAPABILITY-008-TRIGGERS-FIRED` and `P44-HOST-007-TRANSPORT-NOW-EXISTS` take
for their own records.

It does not decide how many of the predecessor's 1,143 rules come across, or in what order. It
decides the shape they land in when they do.

It does not fix the drift it measured, and did not need to: the session that composed
`no-orphan-modules` landed the missing row itself as `P47-ORPHAN-MODULES-HAS-NO-OFFERINGS-ROW`
while this record was being written.

## Status

Accepted.

Revisit if the resolution step, once built, finds a rule whose declaration cannot state
something the array said — which would be evidence the manifest was measured against four rules
and generalized past what they showed, rather than a reason to keep a second table beside it.
