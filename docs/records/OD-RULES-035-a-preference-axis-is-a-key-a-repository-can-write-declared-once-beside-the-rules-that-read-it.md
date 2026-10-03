---
id: OD-RULES-035
type: decision
title: A preference axis is a key a repository can write, declared once beside the rules that read it
status: accepted
version: 3
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
