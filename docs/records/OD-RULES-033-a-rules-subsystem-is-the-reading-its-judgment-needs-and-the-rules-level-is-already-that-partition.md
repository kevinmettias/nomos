---
id: OD-RULES-033
type: decision
title: A rule's subsystem is the reading its judgment needs, and the rules level is already that partition
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
  - measurement
  - authoring
relations:
  - target: OD-RULES-034
    type: relates-to
  - target: OD-RULES-027
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-015
    type: relates-to
  - target: OD-RULES-016
    type: relates-to
  - target: OD-RULES-029
    type: relates-to
  - target: OD-GATE-020
    type: relates-to
---

# A rule's subsystem is the reading its judgment needs, and the rules level is already that partition

## Question

`check-folder-organization` reports one finding over `crates/rules/nomos-rules/src/checks:0`:
the level holds more facade-and-directory module pairs than its signal allows, and the remedy
the message names is not a move of one misplaced file but a regroup of the whole registry into
at most eight subsystems, each at most ten modules wide.

Nothing in this repository says what those subsystems are. No record, no document and no
declaration states whether the flat level is the answer for this crate, or which of its module
roots a ceiling would land on. So a worker holding that finding cannot apply a taxonomy without
first inventing one, and inventing one to move a counter is the gaming this campaign forbids by
name.

This record decides the taxonomy. It deliberately does not decide it by drawing a tree over
today's file list. A partition read off a file tree answers for the files that happen to be
there and for no rule anybody writes next, and this crate has already moved under exactly that
question: the item that boarded this decision censused 38 source files and 17 loose modules on
2026-09-16, and six days later the level holds 40 and 19, two modules having arrived at
`7edd095f` when a compiler-backed capability's rules moved in from another crate. A taxonomy
that a week of ordinary work invalidates is not a taxonomy. What is wanted is a membership
rule: something that says where the *next* rule goes, without being re-decided each time one
arrives.

## Method

Every count below is taken from `crates/rules/nomos-rules` at the revision this record lands
on, by a command a later reader can re-run from that directory.

- The level's width is `ls src/checks/*.rs | wc -l` and `ls -d src/checks/*/ | wc -l`. A
  facade-and-directory pair is a subdirectory with a module file of the same name beside it,
  which is what the check counts.
- The composed population is the identifiers named in `DESCRIPTORS`: the
  `Descriptor_For(crate::<ID>` arguments plus the `Descriptor_For_Declaration(&crate::checks::<ID>`
  ones, which is the whole table since `OD-RULES-034`'s second arm landed.
- A rule's module is resolved by finding the file that declares its identifier constant, under
  `src/checks/`, and taking the top-level module that file belongs to. Every one of the
  seventy-three resolved; none was attributed by hand.
- Subject kinds and required families are read off the same slice of `src/rule_descriptor.rs`,
  which states both on one line per linked rule. A declared rule states neither: `DeclaredTextRule`
  derives both from what the declaration says about test material, so the three declared rows are
  counted by reading their declarations rather than by grepping the table.
- The level's own name in its own source is
  `grep -rnoE '"crates/rules/nomos-rules/src/checks[^"]*"' src/`, and the self-exemptions among
  them are `grep -rn '^const OWN_IMPLEMENTATION' src/checks/`.

One measurement was attempted and is not reported: a topical grouping of the modules, to see
whether an eight-way one falls out. It does not fall out, it is composed, and a grouping this
record's author composed would be measured against nothing — which is the failure this record
exists to refuse. What is reported instead is the two axes the crate already declares, and what
each of them does to the level when it is made a directory tree.

## The census

**The level is forty modules: twenty-one facade-and-directory pairs and nineteen loose module
files, over twenty-one subdirectories.** That is the number the check counts pairs from.

**Seventy-three rules are composed, over thirty-two of those forty modules.** The distribution
is long-tailed and its tail is the bulk of it: sixteen modules hold exactly one rule, six hold
two, four hold three, three hold four, and one each holds five (`go_text`), six (`rust_text`)
and ten (`naming`).

**Eight modules hold no composed rule, and they are two different things.** Four are shared
plumbing — `code_prefix`, `declaration_scan`, `finding_shape` and `test_support` — each extracted
after more than one rule had built the same thing by hand, and each named for what it provides
rather than for what it judges. The other four — `facade`, `constant_scope`, `domain_type_alias`
and `role_surface_pair` — export rules the table does not compose, the population `OD-RULES-034`
measured and `Test_Every_Exported_Rule_Is_Composed_Or_Accounted_For` keeps honest in both
directions.

**What a module is, read off the modules that hold more than one rule.** `security_text`'s three
rules share `Is_Test_Or_Fixture_Source`, `Is_Own_Implementation_File`, one
`OWN_IMPLEMENTATION_FILES` list and one `Finding_For_Line`. `rust_text`'s share one gate, one
self-exemption and, for three of them, a whole interpreter. `naming`'s ten share a case
vocabulary. In every case the module is the unit of *shared reading*: the detector, the scanner,
the vocabulary and the exemption that more than one rule needs. It is not a topic. The
corollary is what makes the level's width what it is — two modules at this level share no
reading, because if they did they would already be one module.

**The two axes the crate actually declares do not produce a subsystem tree.** `RuleDescriptor`
carries `subject` and `requires`, and both are per-rule:

- *Subject kind* has three values — thirty-nine `SourceFacts`, twenty-nine `SourceText` and two
  `Workspace` among the linked rows, forty, thirty-one and two once the three declared rows are
  counted. It does not partition the modules at all: `flakiness_text`, `placement`,
  `script_discipline` and `structure` each hold rules of more than one kind by the linked rows
  alone, and `rust_text` is a fifth once `EVERY_ALLOW_DECLARATION` is counted beside its
  `SourceText` siblings. An axis that cuts a module in half cannot be the axis a directory tree
  is built on.
- *Required family* has sixteen members, of which ten are required by exactly one rule:
  `WordsPolicy`, `ScriptingPolicy`, `ReviewFindings`, `RequirementTrace`, `Reachability`,
  `NestedLocks`, `LintDiagnostics`, `GoalsPolicy`, `DependencyPolicy` and `CopyClones`. A
  directory per family is sixteen directories, which is twice the ceiling the check names, and
  ten of them would hold one rule each. It also fails to partition, for the same reason subject
  kind does.

**The level names itself in its own source twenty-four times, across fifteen files.** Thirteen
of those, held by ten `OWN_IMPLEMENTATION_*` constants in ten module files, are self-exemptions —
the literal path that keeps a rule from reporting the source of its own detector, which
necessarily spells out the construct the rule looks for. `rust_text` exempts a directory;
`closure_bounds`, `concurrency_text` and `security_text` each exempt a module file and one file
beneath it; `constant_scope`, `domain_type_alias`, `enum_shape`, `error_text`, `flakiness_text`
and `scalar_range` each exempt their own file. The remaining eleven are in test modules, which
assert against the same spellings.

## The decision

**A rule belongs to the module that owns the reading its judgment needs, and that module is its
subsystem. The level at `crates/rules/nomos-rules/src/checks` is flat, and flat is what this
membership rule produces rather than what it tolerates.**

The membership rule, stated so that it answers for a rule nobody has written yet:

> Take the reading the rule's judgment performs — the construct it detects, the vocabulary it
> resolves it against, and the source it must not report itself over. If a module at this level
> already owns that reading, the rule joins that module. If none does, the rule is a new reading
> and takes a module of its own, named for the reading and never for the topic.

Three corollaries make it decidable rather than merely stated:

1. **The test is the reading, not the subject and not the topic.** Two rules that are both
   "about security" but detect different constructs are two modules. Two rules about unrelated
   topics that drive the same scan over the same masked line are one module. `copy_clones` and
   `nested_locks` arrived together from one provider at `7edd095f` and are two modules, because
   they are two readings; `security_text` holds three rules and is one module, because they are
   one.
2. **A module with no rule is plumbing, not a subsystem.** The four that hold none are named for
   what they supply. They sit at this level because more than one module needs them, and their
   presence is not evidence the level needs grouping.
3. **The width is measured, not chosen.** The number of modules is the number of distinct
   readings this crate can perform. That is a property of the rule population, so it is not a
   budget a layout decision can spend down: the only ways to reduce it are to delete a reading
   or to merge two that were never distinct, and both of those are findings about the rules
   rather than about the directory.

## How this classifies both judgment arms

It does not distinguish them, and it structurally cannot. `OD-RULES-034` made the arm private
with no accessor, gave `RuleJudgment` a hand-written `Debug` that withholds it, and settled that
`Judges` answers identically for a linked `fn` pointer and a `&'static DeclaredTextRule`. A
membership rule that asked which arm a rule holds would be re-opening a question that record
closed, through a field nothing is permitted to read.

It does not need to. Both arms have a reading, and the reading is what the rule above is keyed
on. A linked rule's detector is the private predicate in its module; a declared rule's is a
`DeclaredDetector` member, which the declaration *names* and never contains, and whose closed
vocabulary the interpreter owns.

That the answer is the same for both is measured rather than argued. All three declared rules
live in `src/checks/rust_text/declarations.rs`, the module whose linked functions they replaced;
all three name `rust_text`'s own `OWN_IMPLEMENTATION_MODULE` as their self-exemption; and all
three sit as rows in the one `DESCRIPTORS` table beside the linked ones. Three rules changed arm
at `219c26f0` and not one of them changed module. A rule's subsystem is therefore invariant
under the change of arm, which is the strongest form of "the taxonomy classifies both" available:
it never had to ask.

## Why there is no coarser tree above this level

Four axes were available and each was measured against the ceilings the check names.

**Subject kind and required family are already declared per rule, in `DESCRIPTORS`.** A directory
tree over either would be the same fact written in two places with nothing comparing them — the
accretion `OD-GATE-020` measured going silently out of step twice and `OD-RULES-034` refused by
name when it declined a second table of declared rules. Both also fail on the arithmetic: three
buckets that cut five modules in half, or sixteen buckets against a ceiling of eight.

**Language is not an axis this population has.** Most rules declare none, and the ones that do
declare it per rule, through `SourceFile::Is_Written_In` or a declaration's `language` field.
Grouping by it would put the majority of the level in one directory named for the absence of a
property.

**Topic has no authority in this repository at all.** `nomos-architecture.json` is the one zone
declaration and it is crate-granular: its `components` divide workspace *members*, and
`nomos-rules` is one member in one component. A sub-crate topical vocabulary would be a second
zone vocabulary at a granularity that declaration has no room for, invented here and read by
nothing. `OD-RULES-029` settled that the layering declaration a rule judges against is the
repository's own data; a topical tree inside the rules crate would be neither that declaration
nor derived from it.

So a subsystem tree here is either a duplicate of a per-rule declaration or a vocabulary with no
source. There is no third option, and that — rather than the inconvenience of moving files — is
why the level is flat.

## Why the taxonomy is not readable by code, deliberately

Because it already is, through the one mechanism that cannot go out of step with itself.

The enumeration of subsystems is the `mod` statements in `crates/rules/nomos-rules/src/checks.rs`.
A module not declared there does not compile, so the list cannot omit a member that exists; a
module declared and unreachable is what this crate's own `no-orphan-modules` rule reports, so the
list cannot carry a member that does not. The membership of a rule in a subsystem is the
directory its identifier constant is declared in, which the compiler also resolves.

A `subsystem` field on `RuleDescriptor`, or a declaration file naming which module belongs to
which group, would be a second place the same fact is written, and the second copy would be the
one nothing checks. This record adds neither, and that is a decision rather than an omission.

## What a reorganisation would cost, measured

Stated because the cheap reading of this record is that flat was chosen to avoid work, and the
cost is worth knowing in either direction.

Twenty-four path literals across fifteen files name `src/checks/...` by spelling. Moving a module
under a subsystem directory invalidates each of them, and the thirteen held by the ten
self-exemption constants fail *silently* and in the dangerous direction: a rule whose exemption no
longer matches its own source begins reporting the detector constants that spell out what it looks
for. This crate has paid for that class of defect before, and it is invisible to every gate step,
because a stale path literal compiles. The eleven in test modules fail loudly, which is the better
half and is not the half that decides the cost.

Beyond the literals, `OD-RULES-015` and `OD-RULES-016` couple a file's name to the types it
declares, so a move is also a rename question, and this repository's own file-size and
one-public-type-per-file rules are judged over this crate. A regroup performed to move a counter
would trade one reported finding for a set of unreported ones.

## What this record does not decide

- **It does not decide for any other level.** `crates/substrate/nomos-analysis/src`,
  `crates/spec/nomos-spec-store/src` and `crates/corrections/nomos-corrections/src` carry the same
  declared shape in their own territories and are each their own claim. The argument above rests
  on a property this crate has — that its modules are readings, and that two of them share none by
  construction — and a level that does not establish that property for itself gets nothing from
  this record. A holder of one of those may cite this record for its *method*; citing it for its
  *conclusion* would be the generalization this record's own method forbids.
- **It does not decide the detector vocabulary's membership.** `OD-RULES-034` left that to be read
  off the rules when a declared rule needs one, and it stays there.
- **It does not decide file layout inside a module.** How many files a module carries, what each
  is named and when one splits are the filename and file-size rules' questions, judged per file.
  This record is about which module a rule is in, not about which file inside it.
- **It does not decide which rules run, in what order, or against what.** Selection is
  `OD-HOST-004`'s and resolution is `OD-RULES-022`'s, and a directory layout that implied either
  would be a second answer to a settled question.
- **It does not decide the shape of `crates/rules/nomos-rules/src/lib.rs`'s facade.** That the
  crate root re-exports every rule by name is a question about re-export form and about who
  contends for one file; it is not a question about subsystems, and this record neither answers it
  nor is evidence about it.
- **It does not decide whether `check-folder-organization`'s signal is right.** That check belongs
  to another tool and this record has no standing over its thresholds. It decides only what this
  crate's answer to it is.
- **It does not amend `OD-RULES-034`, `OD-RULES-027` or `OD-RULES-011`.** Each is read against the
  population above and none is edited, because amending a record is the work of an item that holds
  it.
- **It does not decide what happens if this crate is split.** A subsystem question that arrived as
  a crate-boundary question would be answered by `nomos-architecture.json`'s own vocabulary, which
  is where crate-granular grouping already lives.

## The finding stays reported, and this is the reason it cites

`check-folder-organization` at `crates/rules/nomos-rules/src/checks:0` is not cleared by this
record and is not meant to be. It stays a reported finding whose answer is written down, which is
a different state from an open question and from a silenced one.

No entry is added to `suppressions.json` and no in-code allow marker is added. Both were available
and both were refused: a suppression would record that somebody decided this was acceptable
without recording what they decided, and the marker route the check offers is not open for this
signal in any case. A record is the form in which an answer stays legible to the next holder.

This record performs no reorganisation and changes no source. There is no item deferred behind it,
because there is no move owed.

## What would change this

Not a count. The level's width is the number of readings, so it rising is the rules population
growing and is not evidence about this decision.

Two observations would be:

- **Two modules whose detectors turn out to be the same reading under different names.** That is a
  merge, owed on its own terms, and it makes the level narrower by removing a duplicate rather than
  by grouping distinct things.
- **A rule that cannot be placed because its reading is owned by two modules at once.** That would
  mean the two were never distinct readings, and it is the same merge arriving from the other
  direction. If it happened repeatedly, and the pairs it named had something in common beyond being
  pairs, that commonality would be the first candidate for a real grouping axis this crate has —
  which is what would have to exist before a tree over this level could be anything but composed.

## Status

Accepted.
