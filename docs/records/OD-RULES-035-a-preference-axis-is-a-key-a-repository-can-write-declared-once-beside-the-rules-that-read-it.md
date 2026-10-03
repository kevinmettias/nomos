---
id: OD-RULES-035
type: decision
title: A preference axis is a key a repository can write, declared once beside the rules that read it
status: accepted
version: 5
authority: canonical-normative-record
tags:
  - rules
  - policy
  - configuration
  - measurement
relations:
  - target: ARC-CONFORMANCE-003
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-019
    type: relates-to
  - target: OD-HOST-009
    type: relates-to
  - target: OD-PACKAGE-015
    type: relates-to
  - target: OD-CAPABILITY-008
    type: relates-to
---

# A preference axis is a key a repository can write, declared once beside the rules that read it

## Question

`ARC-CONFORMANCE-003` opens its second milestone with one sentence: *"A preference axis is one
declaration: the new-axis cost falls from the ten paths `P125` reserves to a bounded one."* Its
`G2` gives the sub-goals — every configurable axis resolves through one shared read step, and
the cost of a new axis is measured against a stated bound rather than assumed.

That record authored no task for M2 on purpose: a milestone's tasks are written when it opens,
because their territory is grepped from the code they change. M2 opened when
`P149-A-DECLARED-STANDARDS-CORPUS-IS-READABLE-WHERE-IT-IS-DECLARED` landed at `ed8d92dd`. This
record answers what M2 has to make true, and it starts by measuring the cost M2 was stated
against, because that cost turned out to be the cost of something else.

## What Was Measured

Every figure below was taken at `dc7ee475` from committed history and committed source.

**A key inside a row-shaped family already costs one rule file.** The naming and limits
families carry rows of `(scope, key, value)`, and neither contract crate interprets its keys:
`limits_policy_payload/policy_row.rs` and naming's `policy_row.rs` say so in their own docs,
and the limits provider accepts any key under the block — `Limits_Row` in
`crates/repository/nomos-repo-policy/src/limits/reading.rs` validates the value and nothing
else. Three landings show the cost:

| Commit | Key | Paths | Paths the key needed |
|---|---|---|---|
| `57abccb9` | limits `parameter-count-max` | 2 | 1, the rule |
| `1ce99ab4` | limits `nesting-depth-max` | 10 | 3: the rule, a visibility widening of another rule's private resolver, and the `requires` entry on its descriptor row |
| `d73b3e18` | words `vague`, `vague_exempt` | 10 | 5: the contract payload's fields and codec, its snapshot, its summary, the provider's reading, the rule |

The first two are row-shaped and cost almost nothing. The third is struct-shaped: scripting,
words, goals and test-material each carry a payload struct, and a key there is a field, a
codec arm, a surface snapshot and a provider edit before any rule reads it.

**A new family costs about sixteen existing files and a crate, and that is deliberate.** The
two most recent families landed at `2bb017c3` (test material, 46 paths) and `ed8d92dd`
(standards corpus, 40 paths). Excluding the rule body, a new repository-policy family edits
roughly sixteen existing files — the provider crate's manifest and root, five registration
sites in `nomos-check-orchestration`, three in `nomos-rules`, four workspace files and two
snapshots — and adds a contract crate and its snapshot. The copies across the seven policy
contract crates are large and structural: `Refusal` is identical in all of them, and the
contract-test body is identical in five and differs by two lines in a sixth. None of that is an accident.
`crates/repository/nomos-repo-policy/src/scaffolding.rs` states it — *"the duplication is the
interface"*, and a shared writer *"would make two providers of one capability agree by
construction and prove nothing"* — and `OD-RULES-019`, `OD-PACKAGE-015` and `OD-CAPABILITY-008`
each decided it.

**`ARC-CONFORMANCE-003`'s M2 baseline is the cost of a family, not of an axis.**
`P125-A-RULE-THAT-NEEDS-A-METRIC-FACT-2` reserves ten paths: a metric capability contract, a
Rust complexity provider, the composition sites that wire them, the four workspace files, the
rules crate and its snapshot. Seven of the ten are a *computed* capability — a value measured
from source. The preference that item carries is one limit, and its territory reserves no
policy crate, no provider module and no declaration file, because the limit travels as a key
in the existing limits family. The preference's own cost inside those ten paths is zero paths
of its own. So the milestone was stated against the price of the adjacent quantity, and
reducing it would mean collapsing contract crates three records decided to keep separate.

**The limits family reads a block no repository can write.** Its four keys are
`file-size-review-lines`, `file-size-hard-lines`, `nesting-depth-max` and
`parameter-count-max`, read from `standards.json`'s `limits` block and from
`languages.<lang>.limits`. `code-standards` decodes that same file into one `Limits` struct
with `DisallowUnknownFields` (`kernel/config/limits/config_loading.go:171`, whose own comment
reads *"DisallowUnknownFields turns a typo'd key into a loud error rather than a no-op"*), and
neither that struct (`limits.go:93`) nor its per-language `Overrides` names a `limits` field.
So in any repository whose `standards.json` code-standards also reads, a `limits` block breaks
the other tool on its next run. Measured across the ecosystem: this repository, `xvpe`, `kwb`
and `code-standards` itself declare no `limits` block, at the top level or under any language.
Every limits rule in every one of them judges against its compiled default. The family is
configurable in its code and unconfigurable in every repository that exists.

**Nothing says which keys a family accepts, so a misspelling is silent.** Because the provider
accepts any key and each rule looks up the one string it knows, a repository that writes
`nesting-depth-maxx: 5` gets a row nobody reads, no finding, and the default. code-standards
decided the opposite for its own keys, in the comment quoted above.

**Each rule resolves its own policy, and they have already diverged.** There is no shared
resolver on the rule side. `Resolve_Limit` exists twice, as a private function in
`crates/rules/nomos-rules/src/checks/function_shape.rs` returning `u32` and in
`checks/structure.rs` returning `usize`, each beside its own copy of
`Limits_Policy_Requirement`; `nesting_depth` reaches the second only because `1ce99ab4` widened
it to `pub(super)`. Every rule site that reads a policy family declares its own requirement
for it. What an undeclared value means is chosen per rule and the choices differ: the
limits rules judge against a compiled default, the tooling-language rule judges nothing, and
`P125-A-RULE-THAT-NEEDS-A-METRIC-FACT-2` requires an undeclared limit to be reported as
undeclared rather than defaulted.

**What is already closed, and therefore not M2's.** A rule reading a family its descriptor does
not declare was the defect `NESTING_DEPTH` had: selected alone, it demanded no limits fact and
fell back silently to its default. Two things now hold it. Demand is derived from each
descriptor's `requires` (`Test_A_Rule_Selected_Alone_Should_Demand_The_Family_It_Declares`),
and the supporting-fact trail compares what every selected rule actually read against what its
descriptor declares (`Test_Every_Selected_Rules_Reads_Should_Be_Declared_By_Its_Descriptor`).
A measurement pass for this record reported that no guard existed in that direction; reading
the test showed one does.

## The Decision

**1. An axis is a key, and M2 bounds the cost of a key.** A family is a capability contract,
and its cost stays what `OD-RULES-019`, `OD-PACKAGE-015` and `OD-CAPABILITY-008` built. M2 does
not share contract crates, does not merge providers, and does not make a new family cheaper.
`ARC-CONFORMANCE-003` is amended so that its M2 row states this bound rather than the family's
cost.

**2. Every axis of a row-shaped family is declared once, in `nomos-rules`, beside the rules
that read it.** One table declares each such axis a rule reads: its family, its key, the kind
of value it takes, and what the rule does when the repository declares nothing. A rule reads a
row-shaped family only through an axis in that table and the one resolver that family has; no
rule writes its own resolver or its own requirement for it. The table is private to the crate —
it is the rules' vocabulary, and nothing outside the rules has a use for it. That is why the
contract crates stay ignorant of their keys, as `OD-RULES-011` already built them: the key's
meaning belongs to the rule that reads it. A struct-shaped family needs no table, because its
payload type already names its axes as fields.

**3. What an undeclared axis means is declared on the axis, not chosen inside the rule.** Each
axis carries one of three meanings, and the table says which: the rule judges against a stated
default; the rule reports that the axis is undeclared; or the rule judges nothing. Which one a
given axis takes is its own decision and is not made here — the point is that it is written
where a reader can find it, instead of being inferred from a constant in a function body.

**4. An axis is readable only from a place the repository can write it.** A block in a file
another tool decodes strictly, which that tool does not name, is not a place: writing it breaks
the other tool. The limits family therefore reads `nomos-limits.json` at the repository root,
the convention `OD-HOST-009` set for a declaration Nomos owns outright and that
`nomos-architecture.json`, `nomos-test-material.json` and `nomos-standards-corpus.json` follow,
with the same repository-wide and per-language scoping it has today. It stops reading
`standards.json`'s `limits` block. There is one source per axis: a second one would need a
precedence rule, and a precedence rule for a block nobody can write would decide nothing. No
repository in the ecosystem declares that block, so moving the source breaks no declaration
that exists.

**5. A key a repository writes that no axis declares is a finding — in a file Nomos owns.** In
`nomos-limits.json` and in every row-shaped declaration file Nomos owns outright, a key the axis
table does not name is reported, `Blocking`, as a declaration that will not be applied. That is
code-standards' own choice for its own keys, and it is the same equality `G3` asks of a waiver
that matches nothing. It is **not** applied to a block of a shared file: a key Nomos does not
read there may be read by the file's other owner, as `standards.json`'s `tiers` is by
code-standards, so "unread here" is not "unread". A block another tool owns is read in the
shape that tool declares, and Nomos adds no key to it.

**6. A new axis of Nomos's own goes into a row-shaped family.** A struct-shaped family's fields
are fixed by the schema that owns its block — code-standards' for scripting, words and goals —
so a new field there is not Nomos's to add. Where a new preference is Nomos's own, it is a key
in a row-shaped family read from a file Nomos owns, and never a new field on a payload struct.

**The bound.** With the above in place, adding an axis to an existing row-shaped family edits
files under `crates/rules/nomos-rules/src/` and nowhere else: no contract crate, no provider,
no composition site, no workspace registration file and no surface snapshot. That is a
property a later landing can be checked against, and it is stated as one rather than as a
count, because a count of files is the number the next refactor moves.

## What This Does Not Do

**It does not make a family cheaper**, for the reasons three records already give.

**It does not convert the struct-shaped families to rows.** Scripting, words and goals read
blocks code-standards owns, in the shape it owns them; test material is Nomos's own and holds
one axis, which is not a reason to reshape it.

**It does not decide whether a defaulted value is visible in a report.** Decision 3 makes the
meaning declared. Whether a verdict reached against a stated default says so is a separate
question, and `OD-RULES-011` version 3 decides it. It is not an empty population, because the
rule judged real subjects. The version 2 amendment below says why none of the records it read
owned the question then, and the version 3 amendment records where it is decided now.

**It does not touch waivers.** That is M5, and `ARC-CONFORMANCE-003` already says where it
starts.

**It does not re-decide `standards.json`.** It records one more consequence of a file two tools
share: a block only one of them names cannot be declared by a repository that runs both.

## The Items That Build It

Authored after this record lands, with territory grepped then rather than forecast here. A
live item, `P127-THE-CHECK-SERVICE-RECEIVES-ITS-PROVIDERS-FROM-A-COMPOSITION-ROOT-3`, is moving
provider composition into a new crate as this is written, and a territory taken before it lands
would be stale when it does. Three increments, in this order: the axis table and the one
resolver per family, with every existing row-shaped read site moved onto it; the limits
family's source moved to `nomos-limits.json`; and the finding for an undeclared key in a file
Nomos owns.

## Amendment, Version 2: A Verdict Reached Against A Default Is Not An Empty Population, And No Record Owned Its Visibility Then

**What was wrong.** Version 1's "What This Does Not Do" sent the visibility of a verdict
reached against a default to `OD-ANALYSIS-012`, as the empty-population question that record
accepted, and to `P99-A-RULE-THAT-JUDGED-AN-EMPTY-POPULATION-REPORTS-IT-AS-OD-ANALYSIS-012-DECIDED-3`
as the item that would build it. Neither holds any longer. `OD-ANALYSIS-012` version 2 decides
that a rule's population is the files its norm is about, at the grain of a language or kind.
It also decides that a norm the repository did not declare is not a population: the subjects
were there, and nothing was declared to hold them to. It names that case as `OD-RULES-011`'s
optional-read question and not its own. A rule that judged against a defaulted axis judged
real subjects, so its population is not empty, and the per-rule report that record decides will not
name it. The item was declined, because its `done_when` computed the population from the slice
`run_context` holds, which is not the population of any rule that narrows its own subjects.

**Where the question stood at version 2, read from the records.** Four records came near it,
and none of them decided it.

- `OD-ANALYSIS-012` version 2 excludes it, as above.
- `OD-RULES-011` version 2 decides what a rule does when an optional read finds nothing. It
  applies no override, keeps the rule's own prior behaviour, and produces or alters no finding
  because of the absence itself. Its reason that no reader is owed the difference belongs to its
  first family: whether a name conforms is the same statement whichever source supplied the
  case. It decides this for the six casing rules and leaves every other family to be decided as
  its own later instance.
- This record's decision 3 makes the meaning of an undeclared axis a declared property of the
  axis. One of its three meanings is visible by construction: a rule that reports the axis
  undeclared judges nothing and says why. A verdict on an axis judged against a stated default
  says nothing about where its value came from, and version 1 left that open.
- `OD-POLICY-001` names `Default` as the lowest of its ten configuration layers. It decides that
  an effective policy carries, for each field, the layer and the artifact that decided it, which
  is the provenance a report would need. It does not settle this question, for three reasons.
  It re-homes the rule-parameter families under its resolver only when a second layer states one
  of their fields, and none has. It carries the provenance of a policy, not of a verdict judged
  under that policy. And it leaves rendering undecided.

So no record owned the question at version 2. `OD-RULES-011` is the record `OD-ANALYSIS-012`
hands it to, and its version 2 answer covered one family by an argument it did not extend.
Nothing then decided whether that argument carried to the other families, or whether a verdict
reached against a value the repository never declared owes its reader the provenance of that
value.
`P189-NO-RECORD-DECIDES-WHETHER-A-VERDICT-REACHED-AGAINST-A-VALUE-THE-REPOSITORY-NEVER-DECLARED-SAYS-SO`
was boarded to decide it, and `OD-RULES-011` version 3 is where it is decided. The version 3
amendment below says so.

**What this changes in the decision.** Nothing. Decisions 1 to 6 and the bound stand as
accepted. Only the sentence that routed an open question changed, and at version 2 it said the
question was open.

## Amendment, Version 3: The Visibility Of A Verdict Reached Against A Default Is Decided In OD-RULES-011

**What was wrong.** Version 2 said, in "What This Does Not Do", in its amendment and in its
status, that no governing record decides whether a verdict reached against a value the
repository never declared says so, and it named
`P189-NO-RECORD-DECIDES-WHETHER-A-VERDICT-REACHED-AGAINST-A-VALUE-THE-REPOSITORY-NEVER-DECLARED-SAYS-SO`
as the item that would. That item landed at `457f1e2d`, and `OD-RULES-011` version 3 decides the
question. A reader following this record was told the question was open and sent to an item that
was done.

**What changed.** Each of those three places now names `OD-RULES-011` version 3 as where the
question is decided. The version 2 amendment keeps its account of what the records said at
version 2, in the tense of version 2.

**What this does not do.** It does not restate `OD-RULES-011`'s decision, which is read there:
a second statement here would be a copy that could drift from it. It does not change decision 3,
so what an undeclared axis means is still declared on the axis.

**What this changes in the decision.** Nothing. Decisions 1 to 6 and the bound stand as
accepted.

## Amendment, Version 4: A Value Read Through A Refinement Of Its Key Reads The Refinement First

**What was wrong.** The axis table declares three function axes: `function`, and its two
refinements by visibility, `function.exported` and `function.unexported`.
`function-naming-convention` read the first alone, repository-wide, for every language it judges.
The two refinements were read only by the two Go function rules, for Go. xvpe declares its Rust
function case only under the refinements, in `languages.rust.naming`, so the rule never read what
xvpe wrote and judged xvpe's Rust against the substituted upper-snake. That agreed with xvpe's
declaration by coincidence. Had xvpe changed it, every Rust function would have been held to the
old convention with every test green, and the list of values a repository never declared, which
`OD-RULES-011` version 3 decides a run prints, would name xvpe's function case, which is false. No
record decided which keys a rule reads for one value: `OD-RULES-011` version 3 names the question
and leaves it, and decision 2 above declares axes one key at a time.
`P194-THE-RUST-FUNCTION-NAMING-RULE-NEVER-READS-THE-KEYS-XVPE-DECLARES-ITS-FUNCTION-CASE-UNDER`
asked for it to be decided by measurement.

**What was measured.** The keys each repository in reach writes for a Rust function's case, read
from its committed `standards.json` on 2026-10-03:

| Repository | Revision | Keys | Where |
|---|---|---|---|
| this repository | `5f6d395b` | `function`, upper-snake | `naming` and `languages.rust.naming` |
| kwb | `1b197ee8` | `function`, upper-snake | `naming` |
| hex's calibration fixture | `5f6d395b` | `function`, lower-snake | `naming` |
| xvpe | `ea0d401d` | `function.exported` and `function.unexported`, both upper-snake | `languages.rust.naming` |
| code-standards | `f0d82072` | none for Rust; `function.exported` and `function.unexported` for Go | `languages.go.naming` |

The code-standards corpus under `docs/standards` states rules and declares no naming key: no
document in it names `function.exported` or `function.unexported`. So of the four repositories that
declare a Rust function case, three write only the plain key, one writes only the refinements, and
none writes both. A rule that reads the plain key alone misses xvpe, which is the defect. One that
read the refinements alone would miss the other three, hex's `lower-snake` among them, and hex's
calibration would gain a finding for every function it declares, judged against a case hex never
chose.

code-standards, the other tool that reads this block, already reads both. Its `Override_Case`
(`rules/general/style/shared/naming/overrides.go`) takes the key refined by the symbol's visibility
and falls back to the plain key. It looks them up in a block where a language's key has already
replaced the repository's key of the same name (`merged_Naming`, in
`kernel/config/limits/overrides.go`). For a Rust symbol, exported means a visibility of exactly
`pub` (`is_Symbol_Public`, in `language-kernels/programming/rust/rustlang/rust_symbols.go`), so
`pub(crate)`, `pub(super)` and `pub(in path)` are not exported, and neither is a member of a trait
declaration, which writes no visibility of its own. The syntax fact this workspace's rules read
draws the same line: only an item it labels `Public` is public, and a restricted visibility carries
a label of its own.

**7. A value read through a refinement of its key reads the refinement first.** A key may be
refined by visibility, as `function.exported` and `function.unexported` refine `function`. A rule
that reads one value through a refinement and the key it refines names both axes in the table, the
refinement first. The value is the first of them the repository declares, in this order: the
refinement for the language, the refinement repository-wide, the plain key for the language, the
plain key repository-wide. When neither key is declared anywhere, the plain axis's undeclared
meaning applies and the refinement's own does not. That is code-standards' order, and decision 5
already reads a block another tool owns in the shape that tool declares, so a declaration in that
block now resolves to one value in both tools. For the report `OD-RULES-011` version 3 decides, such
a value counts as declared when either of its keys is declared at either scope, and a value declared
under neither is named under the plain key, whose meaning was the one applied. That record decides
what is reported; this says only when a value read through two keys is one the repository declared.

`function-naming-convention` reads a Rust function this way. A function declared exactly `pub` is
judged against the first of `function.exported` and `function` the repository declares, and every
other Rust function against the first of `function.unexported` and `function`. With neither
declared, both are judged against upper-snake, as before. The rule now also reads `function` under
`languages.rust.naming`, which it did not. Read from the declarations above, nothing it reports
changes in any of the five repositories: this repository declares one case at both scopes, kwb and
hex declare no refinement and no language-scoped key, and xvpe's refinements declare the case the
rule already substituted.

**What this does not decide.**

- **The rule's other languages.** It judges every source with a syntax fact, and a function in any
  other language still reads `function` repository-wide and nothing else. Whether it should read
  that language's own keys, which would change what it reports for xvpe's and code-standards' Go,
  is not decided here. Version 5's decision 8 decides it.
- **The two Go function rules.** They read their refinement alone, for Go, against Go's own
  defaults, and do not fall back to `function`. code-standards does fall back, so for a repository
  that declares `function` and no Go refinement, as this one and kwb do, the two tools judge an
  unexported Go function against different cases. Making them agree changes what those rules read,
  and no other axis changes here. Version 5's decision 8 decides it.
- **`method`.** This repository, kwb and xvpe each declare a method case beside their function
  case, as `method` or as its refinements, and no rule reads either. A Rust method is judged under
  the function keys. Version 5's decision 8 decides it.
- **What a finding says.** `OD-RULES-011` version 3 decision 4 decides it, and this amendment
  changes no finding's text.

**What this changes in the decision.** Decision 7 is added. Decisions 1 to 6 and the bound stand.
The change that builds decision 7 is inside the bound: it edits the axis table and the rule that
reads it, under `crates/rules/nomos-rules/src/`, and no contract crate, provider or composition
site.

## Amendment, Version 5: Every Rule That Judges A Function's Name Reads The Keys Declared For Its Language

**What was wrong.** Decision 7 decided which keys `function-naming-convention` reads for a Rust
function, and named three gaps of the same kind that it left open. The rule judged a function in any
other language against `function` repository-wide alone, never against a refinement and never against
a key declared for that language. The two Go function rules read their refinement for Go and never
fell back to `function`, as code-standards' `Override_Case` does. And this repository, kwb and xvpe
each declare a method case that no rule read. A declared case nothing reads is invisible: it can be
changed or mistyped with every test green, and the report `OD-RULES-011` version 3 decides names the
values a repository never declared while staying silent about values it declared that nothing read.
`P198-NAMING-KEYS-REPOSITORIES-DECLARE-THAT-NO-RULE-READS-FOR-THE-LANGUAGE-THEY-WROTE-THEM-FOR` asked
for each gap to be decided by measurement.

**What was measured.** The function and method keys each repository in reach writes, read from its
committed `standards.json` on 2026-10-03, and the Go sources it tracks:

| Repository | Revision | Go sources | Function keys | Method keys | Where |
|---|---|---|---|---|---|
| this repository | `703971c7` | 1 | `function`, upper-snake | `method`, upper-snake | `naming` and `languages.rust.naming` |
| kwb | `6714d3a` | 0 | `function`, upper-snake | `method`, upper-snake | `naming` |
| hex's calibration fixture | `703971c7` | 0 | `function`, lower-snake | none | `naming` |
| xvpe | `eed12d02` | 631 | Go: `function.exported` upper-snake, `function.unexported` mixed-snake. Rust: both upper-snake | `method.exported` and `method.unexported`, the same cases | `languages.go.naming` and `languages.rust.naming` |
| code-standards | `f0d82072` | 5528 | Go: `function.exported` upper-snake, `function.unexported` mixed-snake | `method.exported` and `method.unexported`, the same cases | `languages.go.naming` |

Three things follow from it. No repository declares a function or method key for any language but
Rust and Go. Every repository that declares a method case declares the same case for a function, at
the same scope and on the same side of visibility, so no verdict in any of them turns on whether a
method key is read. And the code-standards corpus under `docs/standards` still declares no naming
key: no document in it names `function.exported`, `function.unexported`, `method.exported` or
`method.unexported`, and none writes a `function` or `method` key.

code-standards, the other tool that reads this block, judges every declaration through
`Override_Case` (`rules/general/style/shared/naming/overrides.go`). It reads the key refined by the
declaration's visibility, then the plain key of its kind, over the language-merged block, and then
the language's own convention for that kind. A function and a method are two kinds there, and a
method never falls back to a function's keys. A Go method is a function declared on a receiver, or
an interface's method specification (`go_symbols.go`). A Rust method is a callable with a `self`
receiver, and an associated function without one is a function (`rust_Callable_Kind`, in
`rust_symbols.go`). A Rust declaration is exported there only when it is `pub` and sits at the
file's top level (`rust_Named_Symbol`), so a `pub` function inside an `impl` block is unexported
there.

What the syntax fact this family reads can tell apart is narrower. A Go function is declared at a
package's top level and nests under nothing but its receiver, which the fact writes as the qualifier
of its name (`nomos-lang-go`'s `ItemKind`), so a qualified Go function is a method. An interface's
method specification is not an item in the fact, so no rule here judges those names at all. A Rust
function's shape counts its parameters, a receiver among them, and does not say which one is a
receiver, so `fn new(value: u8)` and `fn get(&self)` are the same record but for their names.

**8. Every rule that judges a function's name reads its case through the same keys, for the
language the function is written in.**

- **Every language reads as decision 7 reads Rust.** `function-naming-convention` judges a function
  in any language against the first of two keys the repository declares: the refinement its side of
  visibility selects, then `function`. Each is looked up for the language the source is written in
  before repository-wide. A function is exported when the syntax fact labels it public, which in
  Rust is exactly `pub` and in Go is a name whose first letter is upper case. With neither key
  declared it is judged against upper-snake, as before. That is the block's own shape, read in the
  order its owner reads it, and a language-scoped refinement is the only place xvpe and
  code-standards write their Go function case.
- **The Go function rules fall back to `function`.** Each reads its refinement and then `function`,
  in decision 7's order, for Go. With neither declared, each keeps its refinement's own default:
  upper-snake for an exported function, and mixed-snake for an unexported one. Decision 7 had the
  plain key supply the undeclared meaning, which for the Rust read is the axis that rule read alone
  before. The general rule is this: a value read through several keys takes, when none of them is
  declared, the undeclared meaning of the axis its rule read alone before it read the others. A
  repository that declares none of them is then judged as it was. For a Go rule that axis is its
  refinement. Taking `function`'s upper-snake instead would hold every unexported Go function in a
  repository that declares nothing to a case no unexported Go name can take, because Go exports a
  name by its first letter.
- **A Go method reads the method keys first.** In `function-naming-convention` and in both Go rules,
  a Go function declared on a receiver reads `method.exported` or `method.unexported`, then
  `method`, ahead of the function keys and in the same order. With no method key declared it reads
  the function keys. With none of either declared, it takes the same undeclared meaning a function
  does. code-standards falls back from a method's keys to its language's own method convention, not
  to a function's keys, and that is not followed here. A method was judged as a function before, and
  this fallback keeps every verdict where no method key is declared. No repository in reach has a Go
  method and no method key.
- **A Rust method's keys are not read.** What code-standards means by a Rust method is a callable
  with a `self` receiver, and the syntax fact cannot say which Rust function takes one. Reading
  `method` for every function inside an `impl` block would apply a method's case to associated
  functions code-standards judges as functions. Wherever the two cases differ, one declaration would
  then resolve to two values in the two tools, which is what decision 7 exists to prevent. So a Rust
  function is judged under the function keys, whatever its repository declares for a method. A
  repository that declares a different Rust method case is judged against its function case here
  and its method case in code-standards; no repository in reach does that. The key becomes readable
  when a fact this family requires states a receiver.
- **The unexported Go rule does not judge `main` or `init` declared without a receiver.** The Go
  specification gives a function those names, and no declaration can rename it. Without this, the
  read in the second point would report this repository's own `main` against the upper-snake its
  `function` declares, and the finding's only remedy would break the program.
  `function-naming-convention` already exempts `main` in every language for the same reason.
- **What counts as declared.** For the report `OD-RULES-011` version 3 decides, a value read through
  several keys counts as declared when any of them is declared at either scope. A value declared
  under none is named under the axis whose undeclared meaning applied: `function` for
  `function-naming-convention`, and its refinement for each Go rule. It is never named under a
  method key.

**What this changes, measured.** Fresh binaries were built from the parent tree and from this one.
This repository's `nomos check --root .` reported 475 findings both times, and no naming rule's
finding moved. Three lines changed, and none of them is a verdict. Two clippy findings that predate
this change, in files it edits, moved down by one line and by 57 lines. The cyclomatic-complexity
advisory counted 14848 functions where it had counted 14826, which are the functions and tests this
change adds. hex's calibration fixture reported the same six findings, byte for byte.

This repository's one Go file, `crates/languages/nomos-lang-go-types/helper/main.go`, is judged by
this repository's own declaration, and its verdict does not change: none of the three function rules
reports anything in it, before or after. What it is judged against does change. Its one unexported
function is `main`, which the unexported Go rule held to mixed-snake before. The rule now reads this
repository's upper-snake `function`, and `main` is the one name it does not judge. Every other
function in the file is exported and upper-snake. It was judged against upper-snake before and is
judged against upper-snake now, because this repository declares it.

The other repositories were read from their declarations and not run. kwb and hex track no Go
source, so nothing changes there. In xvpe and code-standards, `function-naming-convention` now
judges an unexported Go function against the mixed-snake they declare for Go rather than
upper-snake, so it stops reporting names their own declaration accepts. The Go rules read their
refinements there as before, and every method key those repositories declare names the same case as
the function key.

**What this does not decide.**

- **Methods in any other language.** A function in a language other than Go reads the function keys
  alone, a C# method among them. No repository in reach declares a key for any such language.
- **`function-naming-convention`'s undeclared case for an unexported Go function.** It stays
  upper-snake. In a repository that declares no function case at all, that rule and the unexported
  Go rule therefore judge an unexported Go function against two cases. No repository in reach that
  tracks Go source declares nothing.
- **Which side of visibility a nested `pub` Rust function is on.** Decision 7 described
  code-standards' Rust test as exactly `pub`, and code-standards also requires the declaration to sit
  at the file's top level. A `pub` function in an `impl` block or a nested module is exported here
  and unexported there. No repository in reach declares different Rust cases for the two sides, so
  no verdict differs. Making the two agree changes which key a Rust function reads, and it is not
  done here.
- **A Go `init` under `function-naming-convention`.** That rule exempts `main` alone, in every
  language.
- **What a finding says.** `OD-RULES-011` version 3 decision 4 decides it, and this amendment
  changes no finding's text.

**What this changes in the decision.** Decision 8 is added. Decision 8 states generally decision
7's rule for which axis supplies an undeclared meaning, and what that rule decides for the Rust
read does not change. Decisions 1 to 6 and the bound stand. The change that builds decision 8 is
inside the bound: it edits the axis table and the rules that read it, under
`crates/rules/nomos-rules/src/`. The table gains the three method axes and every read the three
rules make. No contract crate, provider or composition site changes.

## Status

Accepted. `ARC-CONFORMANCE-003`'s M2 is re-stated by this record, and the amendment to that
record says so.

Amended at version 2 by
`P172-OD-RULES-035-SENDS-A-DEFAULTED-VERDICT-TO-A-POPULATION-QUESTION-OD-ANALYSIS-012-NO-LONGER-ANSWERS`,
which corrected where the record sent the visibility of a verdict reached against a default. No
governing record decided that question then.

Amended at version 3 by
`P193-OD-RULES-035-STILL-SAYS-NO-RECORD-DECIDES-WHAT-OD-RULES-011-VERSION-3-NOW-DECIDES`, which
points that question at `OD-RULES-011` version 3, the record that decides it. Nothing the record
decided changed at either version.

Amended at version 4 by
`P194-THE-RUST-FUNCTION-NAMING-RULE-NEVER-READS-THE-KEYS-XVPE-DECLARES-ITS-FUNCTION-CASE-UNDER`,
which adds decision 7: a value read through a refinement of its key reads the refinement first, and
the key it refines supplies its undeclared meaning. Decisions 1 to 6 did not change.

Amended at version 5 by
`P198-NAMING-KEYS-REPOSITORIES-DECLARE-THAT-NO-RULE-READS-FOR-THE-LANGUAGE-THEY-WROTE-THEM-FOR`,
which adds decision 8. Every rule that judges a function's name reads the keys declared for the
language the function is written in, and a Go method reads the method keys first. A Rust method's
keys are not read, because the syntax fact cannot tell a Rust method from an associated function.
With nothing declared, a rule keeps the undeclared meaning it had. Decisions 1 to 7 did not change.
