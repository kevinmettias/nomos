---
id: OD-RULES-011
type: decision
title: A rule's parameters are a fact a repository declares, not a constant the rule compiles with
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - rules
  - capability
  - naming
  - configuration
relations:
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-CAPABILITY-004
    type: relates-to
  - target: OD-RULES-035
    type: relates-to
  - target: OD-ANALYSIS-012
    type: relates-to
  - target: OD-POLICY-001
    type: relates-to
---

# A rule's parameters are a fact a repository declares, not a constant the rule compiles with

## Question

Every rule `crates/rules/nomos-rules` ships judges its subject against a value baked into
its own source: a casing predicate (`Is_Pascal_Snake_Case`, `Is_Upper_Snake_Case`,
`Is_Lowercase_First_Letter_Convention`), a line-count threshold (`REVIEW_TRIGGER_LINES`,
`GO_HARD_TRIGGER_LINES`), a suffix or a symbol-kind filter. A repository that wants a
different convention, a different threshold, or the same rule scoped to a different
symbol kind has no lever to pull short of forking this crate. Whether that is acceptable —
whether "which case a name takes" is properly this crate's own decision, compiled once for
every consumer — or whether it is a fact each repository declares and this crate resolves,
is undecided, and every naming/threshold rule built so far assumed the first answer without
the question ever being asked.

## What Was Measured

This workspace's own `standards.json` already declares a `naming` block —

```json
"naming": { "function": "upper-snake", "method": "upper-snake" },
"languages": { "rust": { "naming": { "function": "upper-snake", "method": "upper-snake" } } }
```

— schema-validated by this same file's own `require` section (`standards.json`'s
`data_contracts` entry for `standards.json` itself lists `naming` as a required `object`
field). It has held this declaration since before any of `crates/rules/nomos-rules`'
casing rules existed. Nothing in `nomos-rules`, `nomos-check-orchestration`, or any
capability provider in this workspace reads it: `grep -r "naming" crates/rules` finds only
the crate's own hardcoded predicates, never a reader of this file. The repository already
states its own convention as data and every rule that judges that convention ignores the
statement and recompiles a guess instead.

The guess is not even self-consistent. `standards.json`'s `"upper-snake"` is a real,
externally defined vocabulary word — `C:\Users\kmett\source\repos\kevinmettias\code-
standards\rules\general\style\shared\naming\case.go` names eight closed case styles
(`upper-camel`, `lower-camel`, `underscore-camel`, `lower-snake`, `upper-snake`,
`screaming-snake`, `mixed-snake`, `lower-kebab`) and `case_validation.go` gives each one an
exact shape: `upper-snake` is `Compute_Total` (`^[A-Z][A-Za-z0-9]*(_[A-Z0-9][A-Za-z0-9]*)*$`),
`screaming-snake` is `COMPUTE_TOTAL` (`^[A-Z0-9]+(_[A-Z0-9]+)*$`) — two different shapes, not
two names for one shape. `nomos-rules`' own `Check_Naming_Convention`
(`checks/naming/violations.rs::Is_Pascal_Snake_Case`) correctly implements `upper-snake`.
Its own Go sibling shipped in this same build window,
`Check_Exported_Go_Functions_Use_Upper_Snake_Case`
(`checks/naming/go_function_names.rs::Is_Upper_Snake_Case`), is named for the same
convention and implements `screaming-snake` instead — checked directly against `case.go`'s
own worked example (`Compute_Total` vs `COMPUTE_TOTAL`) while drafting this record. A second
rule shipped in the same window, `Check_Unexported_Go_Functions_Lowercase_Only_The_First_
Letter`, mirrors that same wrong shape rather than the `mixed-snake` shape
(`compute_Total`, `^[a-z0-9]+(_[A-Za-z0-9]+)*$`) code-standards' own vocabulary already
names for exactly "the first word lower, the rest cased normally." Two real, shipped,
ledger-verified rules drifted from the vocabulary their own rule id names, in the direction
a hand-rolled predicate always drifts: plausible, untested against the source of truth, and
wrong in a way `cargo test` cannot see because nothing checked it against anything external.

`OD-RULES-010` already answered the adjacent question for a different shape of input: a
`ToolProvider`'s output is a fact a native rule reads through `FactReader::Require` and
judges, never a value the rule embeds or a `Finding` a tool emits directly. The reasoning
transfers without alteration. A repository's declared naming/threshold policy is exactly as
external to a rule's own judgment as `cargo clippy`'s diagnostics are — the rule's job is to
compare a subject against a standard, and a standard a repository can restate is data the
rule must be handed, not a constant it was compiled to already agree with.

## The Decision

A rule's configurable parameters are read as a capability fact, the same
`Require`-then-judge-then-emit shape `OD-RULES-010` already establishes, not embedded as a
Rust constant or a hand-rolled predicate. The first instance is `nomos.cap.naming.policy`:
a repository's resolved naming convention, read from `standards.json`'s `naming` and
`languages.*.naming` blocks (falling back to a stated default when a repository declares
none), materialized once for the whole workspace
(`IncrementalGranularity::WholeWorkspace`, `FactVariant::Syntactic` — the fact is the
declaration itself, not an inference over it).

The eight case styles are read off code-standards' own closed vocabulary and its exact
`case_validation.go` shapes, not reinvented: `UpperCamel`, `LowerCamel`, `UnderscoreCamel`,
`LowerSnake`, `UpperSnake`, `ScreamingSnake`, `MixedSnake`, `LowerKebab`, plus `Any` (no
rule). Each wire-round-trips to code-standards' own lowercase-hyphenated spelling
(`"upper-snake"`, not an invented Rust-side name), so a value written in `standards.json` by
someone who has never seen this workspace's Rust source still resolves correctly. A
symbol's required case resolves by the most specific key present — a per-language override
(`languages.<lang>.naming.<symbol>`) before the repository-wide default
(`naming.<symbol>`) before an unconfigured rule's own prior hardcoded behavior, so an
unconfigured repository regresses nothing.

This is a capability contract, not a provider's own type (`OD-CAPABILITY-002`): the
contract lives in its own crate, `nomos-cap-naming-policy`, below both its provider (which
reads `standards.json` through `nomos_platform::FileSystem`, the same port-supplied-by-
caller shape every filesystem-reading provider in this workspace already uses) and the
`nomos-rules` functions that read it. The provider is not a `nomos-lang-*` crate: it reads
one repository-wide configuration file, not one language's source or manifest format, and
naming it as a language provider would misstate what it does. It is the first crate under a
new `crates/repository/` top-level directory, parallel to `crates/languages/` for the same
reason `crates/languages/` exists apart from `crates/capabilities/` — grouped by what kind
of input a crate reads, not by band alone.

Reading this capability is optional, not required, and `OD-CAPABILITY-004` already
decided how an absent optional capability is reported: never through a rule's
`Applicability`. `Applicability::MissingCapability` says a rule *requires* a capability
and none is installed, which points a reader at installing something; a naming-policy
override is not required by any of the six rules below — each is fully self-sufficient on
its own prior hardcoded default, the same "declaring nothing leaves each language's own
convention in force" default `check-naming`'s own `spec.go` already states. So a rule that
calls `FactReader::Require` for `nomos.cap.naming.policy` and receives
`Err(Applicability::MissingCapability)` — whether because no provider is composed into the
current run, or because the repository's `standards.json` declares nothing for that
symbol — treats it as exactly that: no override, fall back to the rule's own prior
behavior, no `Finding` produced or altered by the absence itself. This is narrower than
`OD-CAPABILITY-004`'s own "packet carries its resolution" obligation: that record binds a
knowledge-context packet with its own downstream consumer that must tell "built without
knowledge" from "built with knowledge that had nothing to add"; a naming judgment has no
such consumer; the `Finding` a name conforms or does not is the identical statement
whether the case it was judged against came from a default or a declared override, so
there is no distinction here for a packet to lose.

`crates/rules/nomos-rules`' six already-shipped casing rules
(`Check_Naming_Convention`, `Check_Project_Owned_Function_Names_Use_Upper_Snake_Case`,
`Check_Module_And_Field_Names_Stay_Lower_Snake`, `Check_Go_Type_Names_Use_Camel_Case`,
`Check_Exported_Go_Functions_Use_Upper_Snake_Case`,
`Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter`) are refactored onto this
one engine as its first real consumers, each keeping its own `RuleId` and gate behavior —
a repository's `standards.json` gains the power to override any of the six without a second
compiled rule, and the `UpperSnake`/`ScreamingSnake`/`MixedSnake` confusion this record
found is corrected as a direct consequence of routing every one of them through the same
tested vocabulary rather than six independent guesses at it.

## What This Does Not Do

It does not generalize every rule in `nomos-rules` in one stroke. Casing is the first
family because it is the most duplicated shape already shipped and the one this record's
own measurement found a real defect in; a threshold family (the file-size triggers), a
suffix family, and whatever else the code-standards corpus's remaining ~1,120 rule
documents turn out to cluster into are each their own future instance of this same
decision, not decided here.

It does not give `nomos-rules` general file-system access or a second way to read
`standards.json` outside the capability/provider seam `OD-RULES-010` already established —
a rule still takes only `&[SourceFile]` and a `FactReader`, per `crates/rules/nomos-rules/
src/lib.rs`'s own stated invariant, and the provider is still the only place in this
increment that touches a path.

It does not compose the new provider into `nomos-check-orchestration::Run`. Several rules
in this crate already ship additive and unwired for one build cycle before composition
follows (`Check_Declared_Role_Matches_Surface`, `Check_Cross_Language_Correspondence` at
their own first landing); this capability follows the same sequencing, as its own later
item.

It does not decide anything about a repository that declares a case style outside the
eight code-standards names — such a value is a load error at the provider, the same
"a casing scheme this check cannot read is one it would silently ignore" refusal code-
standards' own `spec.go` already states for the identical reason.

## Amendment, Version 3: A Value Nobody Declared Is Named Once Per Run, Beside The Claim, And Never In A Finding

`OD-ANALYSIS-012` version 2 hands this record the case of a norm a repository did not declare:
the subjects were there and nothing was declared to hold them to, so it is not a population.
`OD-RULES-035` version 2 then read the records for whoever decides whether a verdict reached
against such a value says so in what a run reports. It found that version 2 of this record
decides it for six casing rules, by an argument particular to casing, and that nothing decides
it for any other family.
`P189-NO-RECORD-DECIDES-WHETHER-A-VERDICT-REACHED-AGAINST-A-VALUE-THE-REPOSITORY-NEVER-DECLARED-SAYS-SO`
asked this record to decide it for every family a rule reads optionally. Every figure below was
taken at `c131760d`, in a detached worktree at that revision, with a binary built there.

**The decision was delegated.** The repository owner asked for the board to be completed
autonomously and has not ruled on this question. It is decided here on the measurements below,
and the last subsection but one says what would reopen it.

### What a composed rule reads that a repository can leave undeclared

`nomos_rules::DESCRIPTORS` has 77 rows, 74 linked and 3 declared. Thirty of them read at least
one family a repository may leave undeclared, and what an undeclared value does falls into four
kinds.

| What an undeclared value does | Family, and where a repository declares it | Composed rules | Rows |
|---|---|---|---|
| the rule judges against a value this workspace substitutes | limits, `nomos-limits.json` | `1500-lines` against 1500 lines; for Go, `five-hundred-line-review-trigger` against 500 and `one-thousand-line-hard-trigger` against 1000; `parameter-count` and, for Go, `go-helpers-package-five-inputs` against 4 value parameters; `nesting-depth` against 3 | 6 |
| | naming, `standards.json`'s `naming` and `languages.<lang>.naming` | `function-naming-convention` (`function`, upper-snake); `data-names-stay-lower-snake` (`module` and `field`, lower-snake); for Go, `exported-functions-use-upper-snake-case` (`function.exported`, upper-snake), `unexported-functions-lowercase-only-the-first-letter` (`function.unexported`, mixed-snake) and `types-use-upper-camel-case-lower-camel-case` (`type.exported`, upper-camel; `type.unexported`, lower-camel) | 5 |
| the rule reports the value undeclared and judges nothing | limits | `cyclomatic-complexity` (`cyclomatic-complexity-max`), by `OD-RULES-035` decision 3 | 1 |
| the rule judges nothing for want of a norm | scripting, `standards.json`'s `scripting` | `declared-tooling-language-for-scripts`, when no tooling language is declared | 1 |
| | goals, `standards.json`'s `goals` | `goals-and-parts-line-up`, when no goal is declared; with goals and no `max_subsystems_per_goal`, it judges the two-way audit and not the spread bound | 1 |
| | standards corpus, `nomos-standards-corpus.json` | `standards-corpus`, for a payload with no roots | 1 |
| | requirement trace, `tests/contract/requirements/` | `requirement-trace-staleness`, whose payload cannot tell a missing directory from every entry resolving | 1 |
| nothing is substituted or withheld: a declaration only adds exceptions to the rule's own norm, or subjects to the rule | words, `standards.json`'s `words.approved_abbreviations` | `abbreviations`, judged against code-standards' shipped vocabulary, which a repository can only add to | 1 |
| | test material, `nomos-test-material.json` | the 14 rows that read it, 13 linked and `every-allow-carries-a-justification`, which classify test material by fixed clauses a repository can only add locations to; the two parameter-count rows are counted above | 12 |
| | limits, read as keys rather than values | `undeclared-policy-key`, whose subjects are the keys a file declares; a repository that writes no file wrote no key | 1 |

The standards corpus and the requirement trace are not families this record built, but a rule
reads each optionally, and `OD-ANALYSIS-012` version 2 names both among the undeclared norms it
sends here, so they are decided with the rest. Two other declarations are outside the question,
and are named so the boundary is visible. `nomos-architecture.json` is a description the rules
judge against (`OD-RULES-029`), and a failed read of it is reported, so the read is not
optional. Undeclared, `dependency-completeness` reports every member `NotApplicable`
(`OD-RULES-003`), and
`dependency-direction` and `dependency-write-authority` judge nothing. `nomos-csharp-builds.json`
reaches no rule: undeclared, the composition reports one `NotApplicable` advisory for a
repository that has C# sources. Both are already visible through findings their own records
decided, and this amendment does not touch them.

### What a reader sees today

**This repository.** `nomos check --root .` examined 2,210 files, 2,209 of them with a syntax
fact, and reported 475 findings, none of which can fail a build, under `claim: incomplete` for
eight `DependencyUnavailable` findings. `nomos gate run --root .` printed the same 475 findings
and no claim line. There is no `nomos-limits.json`, and `standards.json` declares `function` and
`method` and no other naming key, so ten rows judged against values this workspace
substituted: the six limits rows, `data-names-stay-lower-snake` and the three Go naming rows.
The six Go rows among them judge the tree's one Go source,
`crates/languages/nomos-lang-go-types/helper/main.go`. Not one of the ten reported a verdict. Each found its subjects clean, apart from one unreadable
corpus file, and a reader cannot tell those zeros from a repository that declared the same values
and kept to them. `goals-and-parts-line-up` and `standards-corpus` judged nothing, and their zeros
read as clean audits. The one undeclared value a reader can see is `cyclomatic-complexity-max`,
through `cyclomatic-complexity`'s own advisory.

**hex's calibration fixture.** `nomos check` examined 3 files and reported 5 findings, 3 of them
`Blocking`, under `claim: complete`. The fixture declares its naming case for `function`,
`module` and `field` and nothing else, so `1500-lines`, `parameter-count` and `nesting-depth`
judged against substituted values, `cyclomatic-complexity` reported its value undeclared, and
`declared-tooling-language-for-scripts`, `goals-and-parts-line-up`, `standards-corpus` and
`requirement-trace-staleness` judged nothing. The fixture holds no Go source, so the Go rows
judged an empty population, which is `OD-ANALYSIS-012`'s to report. The calibration's verdict
reasons carry where each value came from by hand: "well under the default 4-parameter ceiling",
"far under the default 1500-line justification-trigger ceiling", "the fixture declares none".
One of them is wrong. `data-names-stay-lower-snake`'s reason says the rule judges against "this
rule's own hardcoded default" with "no fixture-side override needed", and the fixture's
`standards.json` declares `module` and `field` as lower-snake, so the rule judged against the
declared value. `OD-ANALYSIS-012` version 2 found `declared-tooling-language-for-scripts`' reason
wrong about what the fixture declares as well. Nothing the run reports could have caught either.

**hex as its upstream ships it.** The fixture's `standards.json` is "**not** part of the
upstream crate", says `PROVENANCE.md`, and was written so the naming rules resolve against hex's
own convention "rather than silently falling back to this workspace's own
`Pascal_Snake_Case`/`upper-snake` default". With it removed, in a copy, the same check reports 13
`function-naming-convention` findings the declared fixture does not have, each of the form
"`from_hex` is not Pascal_Snake_Case: README.md's Conventions section requires function names to
be Pascal_Snake_Case, and Cargo.toml disables rustc's own non_snake_case lint specifically
because this workspace uses a different convention". The fixture has no `README.md`, and its
`Cargo.toml` disables no lint. The verdict was reached against this workspace's house convention, and the
finding attributes it to the judged repository's own files. That is what a third-party Rust
repository with no naming declaration is shown.

**Across the four repositories**, read from their files on 2026-10-03: none has a
`nomos-limits.json`; none declares `module` or `field`; kwb declares no scripting policy; none
declares a goal, and code-standards declares `subsystems` with no `goals`. xvpe declares its Rust
function case only under `function.exported` and `function.unexported`, and
`function-naming-convention` reads the repository-wide `function` key, so xvpe's Rust functions
are judged against the substituted upper-snake, which agrees with what xvpe declared under keys
the rule does not read. code-standards' own resolver reads the visibility-refined key first
(`rules/general/style/shared/naming/overrides.go`).

**What the findings themselves say.** Nine composed rules write findings that misstate or omit
the value they were judged against, or say something about it that only one of its sources
makes true:
`function-naming-convention` names `Pascal_Snake_Case` and this workspace's `README.md` and
`Cargo.toml` whatever case it judged against; `data-names-stay-lower-snake` says "lower snake
case", and the two Go function rules say `Upper_Snake_Case`, whatever case they judged against;
`parameter-count` and `go-helpers-package-five-inputs` say "the configured value parameter cap"
when nothing was configured; `five-hundred-line-review-trigger` and
`one-thousand-line-hard-trigger` call their number Go's when a repository declared it; and
`nesting-depth` names the depth it found and not the limit it judged against.

### What each answer shows a reader, and what it costs

**1. Nothing is reported, version 2's answer extended to every family.** Nothing to build. A
reader of this repository takes ten zeros for declared values kept and two for audits passed.
hex's calibration goes on stating provenance in prose, two pieces of which have been found wrong.
A reader of hex as shipped gets 13 findings that attribute this workspace's convention to hex's
own files, and no sign that a declaration is a remedy.

**2. Each finding says where its value came from.** A reader sees provenance only where there is
a finding, and none of the ten substituting rows reported one in this repository, so this
answer shows that reader nothing. A clean verdict has no finding to carry it. It is also not
free: a finding's summary is hashed into both of its identities, since `OccurrenceLineageId::Of`
and `FindingOccurrenceId::Of` both take its rule, its subject and its summary. A repository that
wrote `nomos-limits.json` restating the defaults would see every such finding end and a new one
begin in occurrence history and in `gate compare`, with no verdict changed. It alters a finding
because of the absence, which version 2 refused, and it states one fact about a run once per
finding.

**3. A finding of its own for each undeclared value, the shape `cyclomatic-complexity` uses.** A
reader sees it on every surface. But `cyclomatic-complexity`'s advisory is that rule's own
verdict that it did not bind, and `NotApplicable` is true of it. A rule that judged against a
substituted value did bind, and a `NotApplicable` beside its own findings would contradict them.
Its subject would have to be the file a value would be written in, which is not what was judged:
the fact is about a verdict, and a verdict is not a subject. It is also permanent. Twelve rows
in this repository and seven in hex would carry one or more each in every run, clearable only by
a file restating defaults, and every one would enter occurrence history, SARIF results and
`gate compare` as a finding.

**4. `Claim` turns `Incomplete`.** Every run of every repository measured would be incomplete for
as long as it writes no limits file, which is all four of them. hex's calibration asserts
`claim == Complete` and would fail. This repository's claim is already incomplete for eight
`DependencyUnavailable` findings and would gain causes that no change of code can clear. And
`Claim::Incomplete` says a run did not reach a judgment, where these runs did: a value this
workspace substituted is a stated value judged against, not the absence of one. This is the
measurement the item required before any change to `Claim`, and it is why there is none.

**5. Named once per run, beside the claim, by rule.** A reader of this repository gets one list
naming thirteen rules: the ten substituting rows with the value each judged against,
`cyclomatic-complexity` with its value reported undeclared, and the two rules that judged nothing.
A reader of hex gets eight, and its calibration can assert them rather than restate them. A reader
of hex as shipped gets ten, and beside the 13 findings a line saying `function-naming-convention`
judged against upper-snake, which hex never declared. The cost is one value per selected rule in
`CheckOutcome::Judged`, a declaration on each descriptor that reads an optional family, and one
rendering per surface. Findings, their identities, `Applicability`, `Claim` and a gate's
disposition do not move.

**The fifth is decided.**

### The arguments the item named, and one it did not

**Version 2's casing argument holds for the finding and fails for the reader, for a threshold
exactly as for a case.** Version 2 reasoned that whether a name conforms "is the identical
statement whether the case it was judged against came from a default or a declared override".
For the finding that is right, and it is why provenance stays out of the finding: a statement and
its identity should not move when a repository writes down the value it already had. It does not
follow that no reader is owed the difference. The remedy differs, a change of code or a
declaration, and a clean verdict has no finding to carry anything. A threshold is the same on both
counts: "exceeds the 500-line review trigger" is one statement whichever source supplied 500, and
a reader holding a file of 620 lines is owed whether anybody chose 500. So no family differs from
casing on this ground, and casing gets the answer every other family gets. Nor was the argument
true of what was built on it: `function-naming-convention`'s summary names a source, and a false
one in every repository but this.

**`OD-RULES-035` decision 3's reported-as-undeclared meaning is what a value takes when no
default is defensible, and it is visible by construction.** This amendment changes no axis's
meaning. A value reported undeclared keeps the finding that is its rule's verdict, and is also
named in the run's list, so the list is the whole of what a run found undeclared and a reader, or
a calibration, consults one place.

**`OD-POLICY-001` supplies the vocabulary and does not own the answer.** An undeclared value is
the `Default` layer's contribution, and its artifact is the build, by that record's decision 2.
The list is, for the rule-parameter families, the part of an effective policy the `Default` layer
decided. It does not re-home those families under the resolver, because decision 5's trigger, a
second layer stating one of their fields, has not fired. It is not a second statement of an
effective policy: when the trigger fires, the list is that family's `Default` rows. And it carries
what `OD-POLICY-001` does not, which values a verdict rested on rather than which layer decided a
field, and it is rendered, which that record left open.

**`OD-CAPABILITY-004`'s obligation is not extended, and its principle is applied.** No knowledge
packet is involved, so its decision 1 binds nothing here. Version 2 declined the obligation
because "a naming judgment has no such consumer". Two consumers were measured above: hex's
calibration, which restates by hand what the fixture declares and has been wrong about it twice,
and a reader choosing between a change of code and a declaration. Its principle is this case exactly: absent
and empty are different facts with different remedies, and graceful degradation is what makes
their conflation silent. Its refusal of "an empty packet with a flag beside it", because "a flag
nothing is required to read is exactly as silent as no flag", is why the list is printed in
default output and never behind an option. That is also the condition the owner set on
2026-10-03 for the population report the list sits beside.

**The sibling tool already places this at the run.** code-standards'
`kernel/config/configidentity/configidentity.go` carries a run's configuration identity as "the
run-level twin of the finding fingerprint", and its `Governed` exists because "'judged by shipped
defaults' and 'judged by rules this repository wrote' are different claims about a green run, and
only one of them says the repository chose what it is being held to". This amendment takes that
placement at a finer grain. A file is too coarse here: this repository's `standards.json` exists
and still leaves `module` and `field` undeclared.

### The decision

**1. What a run reports.** For every rule a run selected and judged over a nonempty population,
every value it read from a family in the first three kinds of the table above that the repository
did not declare, at the grain a repository declares it: the family, the key, and the language the
rule read it for. With each, what the rule did: the value it judged against, that it judged
nothing, or that it reported the value undeclared. A goal ceiling that is absent and one declared
as zero read alike, as code-standards' schema reads them, and are named as a spread bound not
judged rather than as either. A rule whose population was empty is reported there, by `OD-ANALYSIS-012`,
and not here, because it reached no verdict for a value to rest under. A value the run could not
read is not undeclared. It is coverage debt, reported by its own finding, as `OD-ANALYSIS-012`
version 2 decision 2 already says.

**2. What is not reported.** The table's last kind: the words additions, the test-material
locations, and the keys `undeclared-policy-key` reads. A declaration there only adds exceptions
to a norm the rule owns, or subjects to a rule, so the rule reaches its verdict by its own norm
whether or not anything was declared, and nothing is substituted or withheld. An absent list of
exceptions is the same list as an empty one, and a line saying that none was declared would tell a
reader nothing they could act on.

**3. Where, and what it is not.** In `CheckOutcome::Judged`, beside the claim and beside the
population, and apart from both. It is not a finding and carries no `GateCategory`; it is not an
`Applicability` variant and not a `CheckOutcome` arm; `Claim`, `Claim_Of` and a gate's disposition
do not read it. `nomos check`'s text report prints it in default output, and prints nothing new
for a run with nothing undeclared. The gate report, the SARIF log and the API response print it
wherever and however they print the population, as machine-readable as that, and in SARIF never
as a result.

**4. What a finding says.** A finding states the value it was judged against, and nothing about
that value that only one of its sources makes true, so its text and both of its identities are
the same whether the value was declared or substituted. A rule that judges only a declared value,
as `cyclomatic-complexity` does, may say it was declared, since nothing else could be true.
Version 2's rule stands beside this one: an absent read produces, alters and suppresses no
finding. The nine rules measured above break the first half today.

**5. Where the declaration lives.** Which optional values a rule reads is declared on its
descriptor, as the one statement its body also resolves by. That is the placement
`OD-ANALYSIS-012` version 2 decision 1 gave a population, for the same reason: a second statement
beside the body's own is the defect `OD-GATE-020` measured. Whether the repository declared a
value a rule reads is a fact about the repository's declarations and not about the rule's
subjects, so it is answered after the rule is judged, from the run's own materialized facts, by
`nomos-rules`, which keeps the axis table its own as `OD-RULES-035` decision 2 decided. The
composition root records the answer and interprets no key.

**6. The requirement trace says it judged nothing.** `requirement-trace-staleness` is the one
rule whose payload cannot tell its undeclared norm from a clean answer. Its payload tells a
repository with no `tests/contract/requirements/` directory from one whose every entry resolves,
and neither from a provider that did not run. What an unreadable directory or a refused entry
reads as is not decided here.

### What this does not decide

- **No axis's undeclared meaning changes.** Whether `function`'s substituted value should stay
  this workspace's house style, which hex as shipped shows blaming a third-party repository's own
  files, is `OD-RULES-035` decision 3's question for that axis. The list makes it visible and does
  not answer it. code-standards' own resolver says "Which case a name takes is a house style, not
  a property of the language".
- **Which naming keys a rule reads.** Whether `function-naming-convention` should read the
  visibility-refined keys code-standards reads first is not decided here.
- **Whether a rule parameter's value enters a gate run's provenance**, so that `gate compare` can
  tell a loosened limit from fixed code, which is code-standards' `configidentity`'s own question.
- **What `requirement-trace-staleness` does with a corpus it cannot read.**
- **Anything about `nomos-architecture.json` or `nomos-csharp-builds.json`**, or about any
  population, which stays `OD-ANALYSIS-012`'s.

### What would reopen it

- **`OD-POLICY-001` decision 5's trigger firing for a rule-parameter family.** The list is then
  that family's `Default` rows in an effective policy, rendered by the effective policy's own
  assembly, and its placement here is reopened rather than kept as a second statement.
- **A family whose undeclared state depends on the subjects a rule judged** rather than on the
  repository's declarations. Decision 5's placement assumes there is none.
- **A consumer that reads only findings and must learn provenance**, such as a code-scanning view
  that reads SARIF results and nothing else. That would reopen decision 4, and would have to answer
  the identity cost measured above.
- **A list long enough to bury the claim it sits beside.** Counted here at thirteen rules in this
  repository and eight in hex.
- **The owner ruling otherwise** on the question delegated here.

### The items this amendment boards

`P190-A-RUN-NAMES-EVERY-VALUE-ITS-RULES-READ-THAT-THE-REPOSITORY-NEVER-DECLARED-BESIDE-ITS-CLAIM`
builds decisions 1, 2, 3, 5 and 6 for `CheckOutcome::Judged`, `nomos check`'s report and hex's
calibration, and corrects the two module docs that say no record decides this. It waits on
`P172-A-RULE-DECLARES-THE-POPULATION-IT-JUDGES-AND-A-RUN-REPORTS-EVERY-EMPTY-ONE-BESIDE-ITS-CLAIM`,
which builds the population it sits beside and holds most of the files it edits.
`P191-THE-GATE-REPORT-SARIF-AND-THE-API-NAME-THE-VALUES-NOBODY-DECLARED-BESIDE-THE-CLAIM` renders
decision 3 on the other three surfaces, after it and after
`P186-THE-GATE-REPORT-SARIF-AND-THE-API-DO-NOT-SAY-WHICH-RULES-JUDGED-AN-EMPTY-POPULATION`.
`P192-A-FINDING-STATES-THE-VALUE-IT-WAS-JUDGED-AGAINST-AND-NOT-WHERE-IT-CAME-FROM` builds decision
4 for the nine rules, after the population item, whose territory holds most of them. Each item's
territory was grepped at `c131760d`.

## Status

Accepted. `nomos-cap-naming-policy` is this decision's first capability contract,
`nomos-repo-standards` its first provider.

Version 2 adds the read-side rule this record's own text needed before the six rules
could be refactored: how a rule reacts to an absent *optional* capability, settled by
citing `OD-CAPABILITY-004` rather than re-deciding it, once refactoring the first rule
onto this capability made the gap in version 1 concrete.

Version 3 was amended by
`P189-NO-RECORD-DECIDES-WHETHER-A-VERDICT-REACHED-AGAINST-A-VALUE-THE-REPOSITORY-NEVER-DECLARED-SAYS-SO`,
on a decision the owner delegated by asking for the board to be completed autonomously. For every
family a rule reads optionally, it decides that a value the repository did not declare is named
once per run, beside the claim and apart from the population, with what the rule did with it; it
is never a finding and never moves `Claim`. A finding states the value it was judged against and
nothing that only one of that value's sources makes true. Version 2's rule for an absent read
stands. Its reason that no reader is owed the difference is superseded, and it no longer declines
`OD-CAPABILITY-004`'s obligation for want of a consumer, only because no packet is involved.
