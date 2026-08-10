---
id: OD-RULES-001
type: decision
title: A rule states what it needs and is refused an answer beneath it
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - rules
  - capability
  - analysis
  - completeness
  - enforcement
relations:
  - target: D-134
    type: affects
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-CAPABILITY-003
    type: relates-to
  - target: OD-GATE-001
    type: relates-to
---

# A rule states what it needs and is refused an answer beneath it

## Question

`P10-FACT-BYPASS` opened against a measurement and a charge.

The measurement: `nomos-rules` was the only thing in this tree that judges code, and its
dependencies were `nomos-contracts`, `nomos-model`, `syn` and `proc-macro2`. Not
`nomos-analysis`. `FactReader`, `FactStore` and `FactIdentity` had exactly one consumer
outside `nomos-analysis` itself, and it was `tests/integration`. Six closed items of
substrate — `P7-ANALYSIS`, `P7-RUST`, `P7-SLICE`, `P8-PIN` and the invalidation and reuse
work behind them — reached nothing that judges. And `syn` appeared in two manifests,
`crates/languages/nomos-lang-rust` and `crates/rules/nomos-rules`: one workspace, one
language, two independent front ends.

The charge is the part that matters, because it is about a recorded argument rather than
about a shape. `D-134` argues that none of the three instances `OD-COMPLETENESS-001`
analyses can be replayed from git, each having been repaired at the site, and concludes that
the rule must therefore take source text as an argument. The premise defeats a rule that
walks the filesystem. It does not defeat a rule that reads facts. A `FactReader` handed in
as a parameter is as much an argument as a `&[SourceFile]` is; a test can hand one to the
rule over three files it wrote by hand and get the same replay. Whatever ruled facts out,
that was not it.

`OD-GATE-001` is about checks that imply more than they hold. A record whose argument proves
something narrower than its conclusion is the same defect one level in, and it is worse than
a check, because the argument is the part later work cites.

## The Charge Is Correct

It is upheld without qualification.

`D-134`'s replayability premise establishes exactly one thing: **a rule takes its subject as
an argument, and opens no file.** That proposition is true, load-bearing, and worth the
record it was written in. Everything the record does with it afterwards — that the argument
must be text, that facts are therefore out — does not follow, and no other passage of the
record supplies the missing step. The narrower conclusion is restated as `D-134`'s and the
wider one is withdrawn. `D-134` is amended in place at version 2; it is not superseded,
because six of its seven decisions are untouched and are cited by working code.

Two further things `D-134` might have argued and did not are considered below, because an
honest disposition has to say whether the conclusion survives a better argument than the one
it was given. One of them is real. Neither of them is replayability.

## What The Payload Can Actually Carry

The decisive question is not philosophical. `Check_Completeness_Mirrors` needs four things
from a parsed tree, and either the agreed payload of `nomos.cap.syntax.items` carries them
or it does not.

`SyntaxItem` (`crates/languages/nomos-lang-rust/src/syntax.rs`) has five fields: `ordinal`,
`kind`, `scope`, `name`, `visibility`. `Encode_Payload`
(`crates/languages/nomos-lang-rust/src/provider.rs`) writes one `unexpanded` line and then,
per item, exactly `item\t<ordinal>\t<kind>\t<visibility>\t<qualified name>\n`. That is the
whole schema behind `nomos.syntax.items.v1`.

Measured against what the rule needs:

| What the rule needs | In the payload |
|---|---|
| the names of `fn Test_*` definitions | **yes** — `Function` items, by name |
| an `All()` attributed to its type | **yes** — `Table::All`, by qualified name |
| that a `pub const` is of *slice* type | **no** — there is no type field; `pub const LIMIT: usize` and `pub const TABLES: &[&str]` encode identically |
| the doc comment at the declaration site | **no** — there is no doc field, in the struct or in the encoding |

So the rule's two halves fall on opposite sides of the line. Resolving a claimed mirror
against the checks that exist is fully served by the payload as it stands. Discovering which
declarations are universes, and what mirror each one claims, is not served at all, and
cannot be without a second version of a schema that two providers are bound by.

That is the fact of the matter, and it is what this record is built on rather than a
preference about layering.

## The Decision

**The rule consumes facts for the half the fact layer can serve, and the requirement it
states is the rule's own.**

`Check_Completeness_Mirrors` takes a `&mut dyn FactReader` alongside its sources. Universe
discovery continues to read the text it is handed. Check-name resolution — the half that
decides whether a finding is `Blocking` or does not exist — is read from
`nomos.cap.syntax.items` facts, one `FactReader::Require` per subject, under a `Requirement`
the rule itself declares.

There is one signature, not two. A text-only entry point kept beside a fact-consuming one
would be a second place for one rule to be spelled, which is the objection `D-134` itself
raises against writing a judgment beside `EnforcementReach`. The consequence is that the
composition root must supply real facts or the binary does not build, which is the only
construction under which "a judgment reads a fact" is a property of the product rather than
of a test.

**The floor is the rule's and not the caller's.** `Syntax_Requirement()` lives in
`nomos-rules` and asks for `FactVariant::Syntactic` with `Assurance::Sound` soundness. A
composition root cannot lower it. This is the part that makes the fact layer load-bearing
rather than decorative: `nomos-lang-rust-scan` offers the same capability at
`FactVariant::Approximate` and `Assurance::Unsound`, `Guarantee::Satisfies` refuses it
against this floor, and `Registry::Resolve` returns `Unmet::BelowRequirement`.

The refusal buys something specific and demonstrable. The scanner reports a declaration that
a parser does not: its own `Test_The_Declared_Unsoundness_Should_Be_Demonstrable` asserts
that `pub fn Commented()` inside a block comment is reported, and the same keyword table
reports a bare `fn Test_Anything()` there too. Fed to this rule, a `fn Test_Renamed_Away`
sitting inside a block comment or a multi-line string literal would resolve a mirror claim
that nothing checks — the exact defect `D-134` moved from a line scanner to a parser to
eliminate, arriving instead through the resolver. The floor is what stops it, and
`Test_A_Check_Named_Only_Inside_A_Fixture_Should_Not_Resolve` keeps its meaning under the
new source of names.

**Completeness is `Assurance::Unknown` and not `Sound`, deliberately.** A parser cannot bound
what a macro hid, so no provider of this capability can honestly claim complete — and a
floor no provider can meet is not caution, it is a declared need with nothing behind it. What
follows from an unknown-complete index is handled where it costs something, in the paragraph
below.

**Absence is not a clean tree, and it is not a phantom either.** A subject whose syntax fact
cannot be read produces a finding carrying the `Applicability` the reader returned —
`DependencyUnavailable` when the store has nothing, `MissingCapability` when nothing
satisfies the floor, `Unparseable` when the payload itself could not be decoded. And where
the check index is short of something that could have resolved a claimed mirror, that mirror
is **not** reported as a phantom and **cannot** fail a build, because the rule cannot tell a
false claim of coverage from a name it did not get to look for. A mirror that no missing
subject could have carried is one the rule *did* resolve, negatively, and it blocks.
`D-134`'s asymmetry survives — a false claim outranks an admitted gap — with a third state
above both: the rule saying it could not answer. An *admitted* gap does not inherit that
doubt and stays `Supported`, because nothing was resolved for it and so nothing could have
been missed.

*(Amended at version 2. This paragraph originally said that "the check index is marked
incomplete for the whole run: while it is incomplete, a claimed mirror that fails to resolve
is **not** reported as a phantom and **cannot** fail a build". Both halves are superseded.
`OD-RULES-002` replaced `CheckIndex::Incompleteness()` with `Shortfall_For(claimed)`, so
incompleteness is a property of the claim and not of the run; and a claim whose name no
missing subject could have carried now blocks and fails the build. What the rule must not
do — manufacture a phantom out of its own inability to look — is unchanged, and is what
decides which claims the doubt reaches. See the amendment note below.)*

**The second `syn` front end remains, in one place, for a stated and bounded reason.**
`mirror.rs` stops parsing entirely; `universe.rs` keeps `syn` because doc comments and
slice-typed constants are not in the payload and no version of that payload exists which
would carry them. That is a measurement, not a preference, and it names its own end
condition: when `nomos.syntax.items.v1` is superseded by a schema carrying an item's
documentation and its declared type shape, the residual front end goes and `nomos-rules`
depends on no parser at all.

The version is deliberately not cut here, and not for time. `nomos-lang-rust-scan` cannot
read doc comments at all — it skips comment lines and associates nothing with the item below
them. A schema in which "this item has no doc comment" and "this provider does not read doc
comments" are the same bytes would turn every universe under the scanner into one declaring
no mirror: a phantom silently downgraded to an admitted gap, which is absence becoming
success in the one field this rule's whole severity ordering turns on. Getting that right is
schema design — a per-field statement of what was observed, bounded by each provider's
guarantee — and it is `P10-SYNTAX-V2`'s subject, not a step in this one.

## Amendment, Version 2

`P10-PHANTOM-FLOOR` found that this record attached the doubt to the wrong object, and the
finding is upheld. `OD-RULES-002` holds the argument, the vocabulary and the measurement;
this note records what moved here and what did not.

The downgrade is not disturbed. A claimed mirror that fails to resolve against an index
missing a subject's names is not thereby established false — the name may be in the file that
was not read — and reporting it as a phantom would be the rule manufacturing the one finding
it is entitled to stop a build over out of its own inability to look. Absence becomes neither
success nor failure, and the third state above `D-134`'s two stands exactly as written.
`OD-RULES-002` upholds that reasoning without qualification and *sharpens* it: the doubt now
reaches the claims it actually bears on and no others, which is a stronger statement of the
same principle rather than a retreat from it.

What was wrong is the scope. Incompleteness was one flag over the whole run, so one
unreadable subject anywhere silenced a phantom claimed in a file whose facts were read
perfectly well, and those two facts have nothing to do with each other. Over this workspace
the consequence was total rather than occasional: `tests/corpus/analysis/gamma/broken.rs` is
a fixture `nomos-lang-rust` is *supposed* to refuse, so `CheckIndex::Incompleteness()`
returned `Some` on every run, so the blocking arm of this rule was unreachable by
construction and every test was green. A severity table whose top row no input can reach is
the defect `OD-GATE-001` names, and this record published one.

`CheckIndex::Incompleteness()` is replaced by `CheckIndex::Shortfall_For(claimed)`, which
answers about one name rather than about the run. A claim is downgraded only when something
missing from the index could have carried *that* name, decided by whether an unread subject's
own text spells it: an identifier a provider reads out of a token stream is spelled in the
bytes the stream was lexed from, so a subject whose text does not contain the name cannot
have declared it. That the same signal is inadmissible as a *source of names* — which this
record's floor section establishes and which is unchanged — and admissible as a *bound on
what an unread file could have held* is argued in `OD-RULES-002`: the unsoundness is confined
to the arm that adds doubt, where it can withhold a block and can never manufacture one.

So the last clause of the superseded sentence was false and not merely imprecise. Measured
with the shipped binary at `52dbf1a`, over this workspace and with only the rule changed: 14
findings, 0 that can fail a build, exit `0` on the unplanted tree, those fourteen lines
byte-identical to the ones this record's own binary printed; and under a phantom planted as
one `pub const` claiming a mirror that exists nowhere, 15 findings, 1 that can fail a build,
exit `1`. A claimed mirror that fails to resolve now can fail a build, and does.

Two things this record states are inherited rather than revised. `MissingCapability` remains
a whole-run answer, and not as a residue of the framing above: `Resolution::Applicability`
reads the registry and the requirement, neither of which varies by subject, so when it holds
there is no index at all and a name cannot be shown absent from an index that was never
built. And the `Assurance::Unknown` paragraph is untouched — the scoping inherits this rule's
existing completeness bound rather than introducing a second one, so a macro-generated
`Test_X` in a readable file is reported as a phantom before this change and after it.

One sentence of this record moved, and it is the one marked above. Two passages that turn on
the same behaviour are left as written, for reasons worth stating. **What The Fact Layer Is
For** says that with the parser admitted the rule blocks on a phantom mirror; that was not
reachable from the binary this record landed, and it is reachable now, so the amendment makes
the sentence true rather than requiring it to change. **Controls** names
`CheckIndex::Incompleteness` in its fourth weakening; that is a measurement of the code as it
stood here and is kept as it was run, and the same control against a later tree means
weakening `Shortfall_For` instead — `OD-RULES-002`'s first control does exactly that, and
three tests fail across two crates.

## What The Fact Layer Is For

Stated here because the item asks and because the answer was previously only implied: the
fact layer is what lets a judgment say what it needs and be refused an answer beneath it.

Not caching, and not incrementality — those are real and this rule spends neither today.
What it spends is `Guarantee::Satisfies` and `Unmet::BelowRequirement`. Before this record
that mechanism was exercised only by the providers' own tests and by `tests/integration`,
which is to say only by things whose purpose is to exercise it. Now a judgment that can fail
a build is on the other end of it, and the difference between the two providers has a verdict
attached: with the parser admitted the rule blocks on a phantom mirror, and with only the
scanner admitted the rule reports that it could not run rather than reporting clean.
`Test_The_Scanners_Guarantee_Should_Not_Satisfy_This_Rules_Floor` is that sentence as an
assertion.

## Reconciling D-134 With Itself

`D-134`'s Consequences section states as a virtue that "the single scanner is shared rather
than duplicated": `tests/contract` keeps the workspace walk and depends on `nomos-rules` for
recognising a universe in a file. Its decision that discovery parses created the workspace's
second `syn` front end. The record states the first as though it discharged the second, and
it does not, because they are claims about different objects.

A *recogniser* shared across bands is one implementation with two callers, and band 100
observing band 30 is the safe direction — that is true and unchanged. A *parser front end*
vendored beside a provider that already parses the same language is two independent readings
of Rust in one workspace, and no amount of sharing the recogniser makes them one.
`tests/contract/tests/boundaries.rs` had already anticipated the distinction and said so at
the time: `nomos-rules` was placed at band 30 rather than below the providers precisely so
that "the moment one needs a parsed tree it must be able to reach a provider rather than
vendor a second parser". The band was arranged for this edge and the edge was not taken.

After this record the two claims are separated and both are true. Recognition stays shared.
Parsing is duplicated in exactly one module, by a stated exception with a measured cause and
a named condition for its removal, rather than by silence.

## What Was Measured

The claim that the *judgment* is unchanged and only the source of the names moved is not
assumed here. The binary at `fbd7adb` and the binary this record lands with were both run
over this workspace, over the same 190 files, and the outputs diffed.

| | before | after |
|---|---|---|
| files examined | 190 | 190 |
| files with a syntax fact | — | 189 |
| findings | 14 | 15 |
| findings that can fail a build | 0 | 0 |

Every one of the fourteen lines the old binary printed is printed by the new one, byte for
byte: the same thirteen unmirrored universes, and the same `Applicability::Unparseable`
finding for `tests/corpus/analysis/gamma/broken.rs`. The judgment on universes is identical.

The fifteenth line is new and is the fact layer speaking. `broken.rs` is the one file of the
190 that `nomos-lang-rust` refuses, so no fact is materialized for it, so the rule reports
that its check names are missing from the index. Two independent readings of one file both
refused it, and the run now says so twice rather than once — which is the honest rendering
while `nomos-rules` still has a front end of its own, and is the number that would move first
if the two ever disagreed about a file.

So the record does not claim the output is identical. It claims the *verdict* is, and reports
the one line that is not, together with the reason it is there.

## What This Costs

`nomos-rules` gains three dependencies — `nomos-analysis`, `nomos-capability` and
`nomos-cap-syntax` — and names no provider. Its public surface changes: `SourceFile` carries
the `SubjectId` the root filed the fact under, so that rule and root cannot disagree about
addressing by construction; `Check_Completeness_Mirrors` takes a reader; and
`Syntax_Requirement` is new.

`nomos check` becomes a composition root in earnest. It ingests its walk through
`nomos-workspace` — that crate's second dependent — declares the syntax contract, registers
the parser, materialises a fact per file and runs the rule over the store. It also gains a
`build.rs` capturing the target, profile, toolchain and features of the binary being built,
because a `BuildVariantId` written as a constant is an identity component nobody derived from
anything, and one of those cannot be observed to be wrong.

That is more machinery than reading files into a `Vec`, and the machinery is what the exit
code now depends on. A walk that finds source and materialises no fact for any of it exits
`ExitCode::Vacuous` — the code `spec` already gave the meaning "the answer is empty because
something expected was not there". No seventh code is invented: an exit code means one thing
per binary, and a run that judged nothing is that thing whichever way it got there. Where the
vacuity guard *belongs* is `P10-VACUITY-HOME`'s question, and a rule that can now say "I could
not run" is evidence for that item rather than an answer to it.

`nomos-rules` writes a reader for `nomos.syntax.items.v1`. It is the third one in this
workspace — `nomos-lang-rust` and `nomos-lang-rust-scan` each author the bytes, and
`tests/integration/src/surface.rs` reads them back — and there is no canonical grammar
anywhere for a schema two providers are bound by. Adding a third is a real cost and it is
taken knowingly rather than absorbed: consolidating them is `P10-SYNTAX-SCHEMA`. Promoting a
canonical reader into `nomos-cap-syntax` here was considered and rejected. A contract earns a
home below its parties once *more than one party names it*, and there is no second consumer
of a reader today; taking it now would pull that crate and its surface snapshot into this
item for a consumer that does not exist.

`nomos check` also writes a third copy of subject-path normalization.
`nomos_ledger::Subject_Of` computes one for work territory and additionally folds a record
filename onto its identifier; `tests/integration/src/corpus.rs` computes one for a fact and
says in its own doc comment that the duplication "is worth converging behind one home the
moment a third caller appears". This is that third caller. Converging is `P10-SUBJECT-HOME`'s
and not this item's, because it would mean editing a crate this item does not hold — and
importing the ledger's would couple what a fact is about to what a claim is about, which is
the coupling `corpus.rs` refused for the same reason.

The check index derived from facts is not identical to the one derived from text. The payload
records trait-method signatures as `Function` items, which the old text walk never collected,
so a `fn Test_X(&self);` declared in a trait would resolve a claim that nothing runs. Those
are excluded by `Visibility::NotApplicable`, which is the only mark the payload puts on them.
That is a shape convention and not a proof — it works because `visit_trait_item_fn` records
`NotApplicable` and every other function form records the visibility it declares — and it is
written down here so that nobody later reads it as something the schema states. Naming the
distinction properly belongs to `P10-SYNTAX-SCHEMA`.

**What this does not do**, stated plainly:

- It does not remove `syn` from `nomos-rules`. It removes it from one of two modules and
  attaches a condition to the other.
- It does not make findings invalidatable. `Reader::Into_Dependencies` records what the rule
  read, and nothing stores a finding as a fact today, so the edges are collected and unspent.
  Building a rule that offers a capability would be an abstraction with no consumer, and this
  record does not build one.
- It does not close the thirteen unmirrored universes, and it does not widen the rule to
  private lists. Both were named as somebody's next item in `D-134` and remain so.
- It does not wire `nomos check` into the gate. `D-134` deferred that and it stays deferred;
  now that the command composes a registry and a store, the question of what the workspace
  blocks on is sharper rather than softer, and it is `P10-CHECK-GATE`'s.
- It does not settle where the reader for `nomos.syntax.items.v1` belongs. It records that
  three of them exist and that this is a defect.

## Controls

Five weakenings were applied one at a time, the failures observed, and the code restored.
A test that is not red under a plausible weakening is not evidence of anything.

**A subject whose fact could not be read is silently skipped** — the `Err` arm of the
per-subject read drops the subject instead of recording it. Five tests fail. The one worth
quoting is `Test_An_Empty_Store_Should_Not_Report_A_Clean_Tree`, which fails reporting a
single finding over a store holding nothing: subject `T`, applicability `Supported`, gate
`Blocking`, summary *"Test_Somewhere resolves to no check, so this rule is declared enforced
and never runs"*. That is the rule manufacturing the one finding it can stop a build over,
out of an index it never built — the defect the incompleteness rule exists to prevent,
produced on demand.

**The resolver falls back to scanning the text when a fact is missing** — the `Err` arm
harvests every `Test_…` word from the file's text into the index. Four tests fail, including
`Test_A_Mirror_Should_Resolve_Only_Through_A_Fact` on the assertion *"the claim must not
resolve out of a text the rule can still see"*. This is the control that a fact path no
judgment consults would pass and this rule does not: with the fact withheld the text is still
in the rule's hand, and the verdict changes anyway.

**Every decoder refusal becomes an empty set** — `NotUtf8`, `NoHeader`, `UnknownRecord` and
`WrongFieldCount` all return `Ok(BTreeSet::new())`.
`Test_A_Payload_This_Build_Cannot_Read_Should_Not_Decode_To_No_Items` and
`Test_A_Refusal_Should_Say_What_It_Refused` fail: four shapes of unreadable payload each
report a file that declares nothing.

**The index always reports itself incomplete** — `CheckIndex::Incompleteness` returns
`Some(DependencyUnavailable)` unconditionally, so nothing can ever block. Five tests fail,
`Test_A_Phantom_Should_Still_Block_When_Every_Subject_Was_Read` among them. This is the
control on the three above: without it they are all satisfied by a rule that never blocks.

**The composition root stops refusing a run that materialized nothing** — the `facts == 0`
guard is removed. `Test_A_Run_That_Materialized_No_Facts_Should_Not_Report_Clean` fails: a
real directory holding one file the parser refuses exits `Ok` instead of `Vacuous`.

## Status

Accepted, landed by `P10-FACT-BYPASS`.
