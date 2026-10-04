---
id: OD-ANALYSIS-012
type: decision
title: A rule that judged an empty population is reported apart from one that judged clean
status: accepted
version: 3
authority: canonical-normative-record
tags:
  - analysis
  - architecture
  - completeness
relations:
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: OD-COMPLETENESS-004
    type: relates-to
  - target: P43-SCRIPT-RULES-CANNOT-FIRE
    type: relates-to
  - target: OD-RULES-014
    type: relates-to
---

# A rule that judged an empty population is reported apart from one that judged clean

## Question

`CheckOutcome::NoFacts` catches one instance of a general shape: a run whose syntax facts
never materialized reports that fact rather than rendering as a clean tree. Nothing
generalizes the same protection to a single composed rule whose own subject population was
empty while the rest of the run judged real subjects. `P43-SCRIPT-RULES-CANNOT-FIRE`
measured a real, live instance: four composed rules — `scripts-use-a-portable-shebang`,
`a-script-declares-its-purpose`, `executed-scripts-set-nounset`,
`declared-tooling-language-for-scripts` — now walk a real, reachable population that is
simply empty in this tree today, and their zero findings render exactly as if they had
examined real scripts and found them clean. A person required a decision on how a run owes a
reader that distinction, independent of whichever fix the walker itself got.

## What Was Measured

**Every existing outcome vocabulary was read for a shape that already fits, and none does.**
`nomos_contracts::Applicability`'s ten variants are all per-*finding* dispositions — each one
presupposes a `Finding` with a `subject` to attach itself to. An empty population has no
subject at all: there is nothing to build a `Finding` around, so no `Applicability` value,
including `ProviderUnavailable` and `NotApplicable`, can carry this fact. `Claim_Of` (`crates/
orchestration/nomos-check-orchestration/src/examined/claim.rs`) proves the gap directly:
`Test_Claim_Of_Should_Report_Complete_For_An_Empty_Findings_List` shows an empty findings
list already reports `Claim::Complete` — the identical verdict a rule that judged real
subjects and found them clean would produce. `Examined { files, facts }` is the nearest
structural precedent — "two denominators, not one," because "0 findings over 400 files" and
"0 findings over 400 files none of which produced a fact" are different claims — but it is
computed once for the whole run's syntax layer, not per composed rule, and does not reach
`Check_Goals_And_Parts_Line_Up` (a `SubjectKind::Workspace` rule with no per-file population
at all) or the four script rules (`SubjectKind::SourceText`, reading whatever the walk
collected rather than a syntax fact).

**The four rules `P43-SCRIPT-RULES-CANNOT-FIRE` names are the real, present instance, not a
hypothetical.** That correction fixed the walkers that fed them and measured, directly, that
this repository has zero real `.sh`/`.ps1`/`.psm1`/`.bat`/`.cmd` files anywhere outside
`target`/`.git` today. All four rules are now composed against a real, reachable, currently
empty population. Their own zero findings are honest about the tree — nothing wrong exists —
but a reader cannot tell that from a rule that examined a hundred real scripts and found
every one compliant. `P43-SCRIPT-RULES-CANNOT-FIRE`'s own commit message states this outright:
"the four rules will fire the moment a real script with a defect exists to walk into" — which
is a promise about the future, not a fact this run's own report states about the present.

**Where the verdict would be computed is already known, because the population itself is
already known there.** `run_context.rs`'s `Findings_For_Selected_Rules`/`With_Composed_Rules`
already holds, for every `ComposedRule`, the exact slice its check closure is about to read —
`sources` for the text and syntax-fact rules, `capabilities.dependency_sources`/
`lint_sources`/`policy_sources` for the capability-backed ones. The population size is not a
new fact to materialize; it is a count already sitting in scope at the one place every
composed rule's own subjects are threaded through, before its closure is called.

## The Decision

**An empty population is a new fact reported alongside a run's findings, not a new
`Applicability` variant and not a new `CheckOutcome` variant.** It is not `Applicability`
because there is no subject for a per-finding disposition to describe. It is not a new
`CheckOutcome` arm because `CheckOutcome::NoFacts` already owns the coarser claim — the whole
syntax layer failed for every source — and a per-rule empty population is a narrower, still-
real fact that can be true while the rest of the run judges plenty. The right extension is
`CheckOutcome::Judged` itself, alongside `findings`, `examined` and `claim`: a third
denominator, in `Examined`'s own words, naming which composed rules examined a real,
nonempty population and which examined none — computed at the same point in `run_context.rs`
that already holds each rule's own source slice, at no new materialization cost.

**Its disposition is Advisory, the same as `ProviderUnavailable`'s existing `GateCategory`,
and it does flip `Claim` to `Incomplete`.** `Claim::Incomplete`'s own definition is "the run
did not reach a judgment about it" — a rule with zero subjects reached no judgment about
anything, the identical shape a coverage-debt finding already represents, not merely a milder
version of it. Reporting it as `Complete`, the way an empty findings list does today, is the
exact lie this record exists to name: `Claim` currently cannot distinguish "every rule judged
real subjects and found them clean" from "some rules judged nothing at all," and a person
reading a clean, `Complete` run has no way to learn that four of the rules that would have
told them about a broken script never got the chance to look for one.

**Applied to the four rules `P43-SCRIPT-RULES-CANNOT-FIRE` names: today, in this tree, all
four report an empty population under this decision**, since the same repository-wide search
that correction already ran found zero real scripts of any of the five recognized
extensions. This is not a defect in either correction — the walker fix was necessary and
correct on its own terms, and today's population really is empty — it is exactly the fact
this record's own mechanism exists to surface rather than hide.

## What This Record Does Not Do

**No code moves here.** `CheckOutcome::Judged`'s new field, the population count computed in
`run_context.rs`, and `Claim_Of`'s own extension to read it are named precisely enough for a
follow-up item's territory to be declared completely, rather than a decision to build
against.

It does not touch `Applicability` or add a variant to it. The measurement above found every
existing variant presupposes a subject this fact does not have, and inventing one anyway
would misuse a per-finding vocabulary for a per-rule fact.

It does not decide `P41-APPLICABILITY-IN-THE-PLAN`. That item asks which subjects a rule
*declines* once a plan resolves applicability in advance — a question about a plan that does
not exist yet, per `P41-RUN-PLANNER`'s own stranded state. This record's own question — what a
run owes a reader about a rule whose population was already empty when it ran — holds with or
without a plan, is true today, and is answered here rather than left waiting on a planner
that has not been built.

It does not change what `P43-SCRIPT-RULES-CANNOT-FIRE` already did. That correction remains
the right fix to the walkers; this record adds the reporting layer that would have made its
own prior silence visible without needing that specific investigation to find it by hand.

## Amendment, Version 2: A Rule Declares The Population It Judges, And An Empty One Is Reported Beside The Claim Rather Than In It

Version 1 decided three things that cannot all be built, and
`P99-A-RULE-THAT-JUDGED-AN-EMPTY-POPULATION-REPORTS-IT-AS-OD-ANALYSIS-012-DECIDED-3` was declined
before any edit for saying so. `P172-OD-ANALYSIS-012-NAMES-A-POPULATION-THE-RULES-DO-NOT-HAVE`
asked three questions in its place: where a rule's population comes from, whether an emptiness
the repository itself causes flips `Claim`, and what the four script rules report here. This
section answers each, on measurements taken at `0802e5b6`. Version 1's text above is left as it
was written, and this section says which of it still holds.

### What version 1 got wrong, measured

**The slice `run_context` holds is not the population most rules judge.** Version 1's "What Was
Measured" says each rule is handed the exact slice its check reads: the walked sources for the
text and syntax-fact rules, a capability slice for the rest. At `0802e5b6` that mapping is
`Judged_Sources` in `crates/orchestration/nomos-check-orchestration/src/run_context/judging.rs`.
`nomos_rules::DESCRIPTORS` has 76 rows, 73 written through `Descriptor_For` and 3 through
`Descriptor_For_Declaration`. `Judged_Sources` hands 8 of them their capability family's slice
and the other 68 the whole walk. Forty-five of those 68 then narrow inside their own bodies
before judging anything:

| Population a row judges | Rows | How the row is handed it |
|---|---|---|
| Rust sources only | 26 | the whole walk, narrowed in the body |
| Go sources only | 14 | the whole walk, narrowed in the body |
| Rust or Go sources | 2 | the whole walk, narrowed in the body |
| a source whose first line is a shebang naming an absolute interpreter path | 3 | the whole walk, narrowed in the body |
| every walked source | 19 | the whole walk |
| a capability family's own slice | 8 | `Judged_Sources`' six arms |
| the one workspace, read as a single fact | 4 | the whole walk, unread |

The item's own why counted 83 composed descriptors. The table has 76 rows, at `0802e5b6` and at
`df79bad6` both.

So a count of the slice cannot see the instance version 1 names. This tree's walk holds 2,134
sources and hex's calibration fixture holds 3. No source slice is empty in either, and the
three shebang rules' slice is the whole walk in both. The one slice that is empty in both trees
is `review-finding`'s, which `Materialize_Review` returns empty for every caller.

**A count of the real population flips every single-language run.** Counted per rule instead,
18 of the 76 rows judge an empty population in this tree and in hex alike: the 14 Go-only rows,
the 3 shebang rows and `review-finding`. hex's calibration asserts `claim == Complete`
(`tests/integration/tests/calibration.rs`) and would fail, with 18 causes. This repository's own
`nomos check --root .` already reads `claim: incomplete` at `0802e5b6`, for seven
`DependencyUnavailable` findings on `tests/corpus/analysis/gamma/broken.rs`, the corpus's
deliberately invalid file. Version 1 would add 18 causes to that, and nothing short of adding Go
files and shell scripts to the tree could clear them. Version 1 never weighed this.

**Three of the four script rules have an empty population, not four.** Decision 3 counts them.

### The owner's decision

The item left its second question to the repository owner. Asked on 2026-09-27 whether a
repository that uses only one language can ever get a `Complete` result, the owner answered:
"yes, that should be obvious." That binds this amendment. **An empty population caused by the
repository holding no file of the rule's own language or kind does not flip `Claim`, and a
single-language repository can be `Complete`.**

On 2026-10-03 the owner ratified decision 2 below, which extends that ruling from the cause the
repository makes to every cause of an empty population, with one condition: **the rules that
judged an empty population are named prominently in the default output of every rendering of a
verdict, and never behind a flag.** Without it, reported beside the claim degrades into not
reported, because a list a reader has to ask for is one most readers never see, and every rule
on it goes on reading as a clean judgment. That ratification and its condition bind version 3,
whose amendment below says what the condition required.

### Decision 1: a rule declares its population on its descriptor, as the one statement its body also filters by

The item named three candidates. A declaration on the descriptor splits in two, by whether the
root partitions the slice by it or only counts it, so four are costed, each at `0802e5b6`:

| Candidate | Registration sites | Rule signatures | Refused by |
|---|---|---|---|
| reported by the rule with its findings | `RuleJudgment`'s function-pointer type and all 73 linked rows | the 42 linked functions that narrow change their return type, and 232 lines referring to them outside the table and its re-export lists, in 31 files and nearly all of them unit tests, follow | nothing; it is the most expensive by an order of magnitude |
| the slice after all, with the narrowing rules mapped at the root | `Judged_Sources` grows from 6 arms naming 8 rules by one arm per narrowing rule, 45 more rule identifiers imported into `nomos-check-orchestration`; mapping the script rules alone would leave the 14 Go-only rows' emptiness as invisible as it is now | none | `OD-RULES-014`'s "Why Not An Orchestration-Side Partition Of The Source List"; and a new language rule would be declared twice, the defect `OD-GATE-020` measured |
| declared on the descriptor, the root partitioning the slice by it | one field, one builder, 42 rows | none | the same section of `OD-RULES-014`: it is still a partition at the root, whichever crate holds the column |
| **declared on the descriptor, the body filtering by the same declaration, the root only counting** | one field, one builder, 42 rows; the 3 declared rows derive theirs | none; 34 filter sites in 25 files read the declaration in place of their own literal | |

**The fourth is decided.** `RuleDescriptor` carries the population its rule judges, as a value
fixed when the rule is written and never computed from run state, which is the line
`Judged_Sources`' own doc draws against the planner `OD-RULES-009` declines. It defaults to
every source the rule is handed, which is right for the 19 rows that judge the whole walk and
for the 8 capability rows judged over their family's slice. A Workspace row's population is the
one workspace its `SubjectKind` already names, so it is never empty. The 42 linked rows that
narrow declare theirs. The 3 declared
rows need no new site: `DeclaredTextRule::language` already is their population, and the
interpreter in `checks/rust_text.rs` already filters by it, so those three are one statement
today. `FunctionAritySource::Language`, which `go-helpers-package-five-inputs` reaches through
`For_Language`, is the form this already takes inside one linked rule.

**One statement, because two is what `OD-RULES-014` decided against.** That record makes a
rule's language restriction a comparison against the language the composition root carries on
`SourceFile`, and it declined both a second classifier and a partition of the source list at
the root. A descriptor column beside an unchanged body filter would be "a second answer to
applicability rather than a first". A descriptor column that the root partitions by would be the
partition. So the declaration is the very value the body's filter reads, and the root reads it
only to count. Every body still receives exactly the sources it receives today, and no rule's
findings change.

**The grain is the language or kind, and nothing finer.** A rule's population is the files its
norm is about. What a body excludes within that kind is judgment: its own implementation file,
test material, a Go file that is not a test, a field with no range attribute. A population
wholly excluded that way still counts as judged. A norm the repository did not declare is not a
population either: no naming case, no scripting policy, no standards corpus, no requirements
directory. The subjects were there and nothing was declared to hold them to, which is
`OD-RULES-011`'s optional-read question and not this record's.

### Decision 2: no empty population flips `Claim`, and every one is reported beside it

The owner decided the case the repository causes. **This amendment gives every cause the same
answer**, on two measurements.

**No count can tell the owner's case from a walker defect.** Before `a98d1f5d` every walker
admitted only `.rs` and `.go`, so the three shebang rules counted zero in any tree, whether or
not it held scripts. That is the defect `P43-SCRIPT-RULES-CANNOT-FIRE` found by hand. Today this
tree holds no shebang file of any extension anywhere the walk reaches, so it counts the same
zero for the opposite reason. A flip that spared the one would spare the other.

**A failure a run can detect already flips `Claim`, through a finding of its own.** A syntax layer that produced no fact is `CheckOutcome::NoFacts`. A capability family
whose materialization failed returns no sources and one `ProviderUnavailable` finding, which is
coverage debt; `dependencies.rs`, `lint_materialization.rs`, `policy_materialization.rs` and
`compiler_materialization.rs` each raise one. A walked subject whose fact could not be read is
reported by the rule that tried, as the seven `broken.rs` findings above are. The one slice
empty in both trees is no failure. `review-finding` is empty because no run names a review
to fetch, which `ARC-CONNECTOR-001` declines to invent ahead of a caller that needs one.
Flipping on it would make every full run of every repository `Incomplete`, which is a stronger
form of what the owner ruled out.

**It is reported rather than left silent, because of what each choice shows the two readers.**

- *Not reported.* This repository's claim reads as it does today, and hex's stays `complete`. In
  both, 18 rules' zeros read exactly like judgments of real subjects found clean, which is the
  lie this record was written to name, made permanent. The calibration table carries the fact
  by hand instead, in its 18 verdict reasons for exactly those rules: 17 read "not applicable" or
  "Go-only", and `review-finding`'s is `Uncalibrated`. Two of the table's reasons for the script
  rules have already drifted from the code. `declared-tooling-language-for-scripts`' says the fixture has no script
  source, but hex's `standards.json` declares no `scripting` policy, so the rule returns before
  reading any source. `scripts-use-a-portable-shebang`'s says the fixture is "exactly two named
  .rs files", but `Fixture_Sources` reads three.
- *Reported beside the claim.* Both claims read exactly as they would unreported. Both reports
  name the same 18 rules as having had nothing to judge, and the calibration can assert that
  list rather than restate it in prose.

So the population is reported, and it does not enter the claim. Version 1's placement stands:
`CheckOutcome::Judged` gains a per-rule population beside `findings`, `examined` and `claim`.
So does its refusal of an `Applicability` variant, since the fact still has no subject. `Claim`
and `Claim_Of` are untouched, so `nomos-gate-orchestration`'s `Reduced_With_Coverage` answers
exactly what it answers today. Version 1's "Advisory" disposition goes with the flip: this fact
is not a finding and carries no `GateCategory`. `nomos check`'s own report names the selected
rules whose population was empty. Whether the gate report, the SARIF log or the API response
render it is left open, as
`P99-A-RULE-THAT-JUDGED-AN-EMPTY-POPULATION-REPORTS-IT-AS-OD-ANALYSIS-012-DECIDED-3` already
left it. Version 3's decision 4 closes that question: each of them does, by default.

### Decision 3: what the four script rules report here

Counted at `0802e5b6` over this tree's walk of 2,134 sources, every one of them `.rs`:

| Rule | Population | Sources in it | Reported |
|---|---|---|---|
| `scripts-use-a-portable-shebang` | a source whose first line is a shebang naming an absolute interpreter path | 0 | empty, and `Claim` is unaffected |
| `a-script-declares-its-purpose` | the same | 0 | empty, and `Claim` is unaffected |
| `executed-scripts-set-nounset` | the same | 0 | empty, and `Claim` is unaffected |
| `declared-tooling-language-for-scripts` | every walked source, since it judges each path's extension | 2,134 | not empty: this repository declares `scripting.tooling_language` as `rust` with five forbidden extensions, every path was judged against them, and none matched |

Four walked sources open with `#!`, and all four are Rust inner attributes: three
`#![forbid(unsafe_code)]` and one `#![doc = …]`. The three rules' shared predicate refuses them,
because it requires an absolute path after the two bytes. A search of every file the walk would
reach, whatever its extension, found no shebang at all.

`declared-tooling-language-for-scripts` does not have an empty population, because its norm is
that no path carries a forbidden extension. Every walked path is a subject that norm is checked
against, and finding no match is its clean answer. In hex it judges nothing for a different
reason: no `scripting` policy is declared there. That is the undeclared-norm case of decision 1,
not a population.

### What version 1 still decides

An empty population is a fact apart from a clean judgment, and a reader is owed it. It is not
an `Applicability` variant and not a new `CheckOutcome` arm. It lives in `CheckOutcome::Judged`
and is computed where each rule is judged, at no new materialization cost. `P41-APPLICABILITY-IN-THE-PLAN`
is not decided here, since a declared population is read when a rule is judged and is not
resolved in a plan. `P43-SCRIPT-RULES-CANNOT-FIRE`'s walker fix stands.

What version 1 said this mechanism would have done for that fix is now narrower. A script rule
that judges nothing is visible in every run, and it makes no run `Incomplete`.

### The items this amendment boards

`P172-A-RULE-DECLARES-THE-POPULATION-IT-JUDGES-AND-A-RUN-REPORTS-EVERY-EMPTY-ONE-BESIDE-ITS-CLAIM`
builds decisions 1 and 2, with the owner's decision in its `done_when` as binding. Its territory
was grepped at `0802e5b6`: the descriptor and its declared rows, the bodies of the narrowing
rules, the counting in `judging.rs`, the new field and every site that constructs or wholly
binds `CheckOutcome::Judged`, `nomos check`'s report, the hex calibration and the two surface
snapshots. It waits on
`P171-THE-CSHARP-CONDITIONAL-COMPILATION-FACT-JOINS-THE-RUN-FOR-THE-BUILDS-A-REPOSITORY-DECLARES`,
which holds the crates it edits and adds a C#-only rule whose population is one more to declare.

`P172-OD-RULES-035-SENDS-A-DEFAULTED-VERDICT-TO-A-POPULATION-QUESTION-OD-ANALYSIS-012-NO-LONGER-ANSWERS`
corrects three passages that send this record a question it no longer answers. `OD-RULES-035`'s
"What This Does Not Do" routes the visibility of a verdict reached against a default here, as
the empty-population question that
`P99-A-RULE-THAT-JUDGED-AN-EMPTY-POPULATION-REPORTS-IT-AS-OD-ANALYSIS-012-DECIDED-3` builds. A
defaulted verdict judged real subjects, so it is the undeclared-norm case this amendment
excludes, and that item is declined. The module docs of
`crates/capabilities/nomos-cap-requirement-trace/src/provider.rs` and
`crates/rules/nomos-rules/src/checks/requirement_trace.rs` say an absent requirements corpus
waits on this record's mechanism, which will not cover it. They are corrected there rather than
here, because none of the three is this amendment's territory.

## Amendment, Version 3: Every Default Rendering Of A Verdict Names The Rules That Judged An Empty Population

The owner's ratification of 2026-10-03 and its condition are recorded in version 2's "The owner's
decision" above, beside the 2026-09-27 ruling they extend. This section says what the condition
required, and decides the one placement it left to whoever built it.

### What the condition found unmet

`P172-A-RULE-DECLARES-THE-POPULATION-IT-JUDGES-AND-A-RUN-REPORTS-EVERY-EMPTY-ONE-BESIDE-ITS-CLAIM`
built decisions 1 and 2 and rendered the population in one place, `nomos check`'s text report.
The renderings decision 2 left open are where most readers meet a verdict: a CI log reads
`nomos gate run`'s report, a code-scanning view reads the SARIF log, and an editor or an agent
reads `nomos-api`'s responses. None of them carried it. Over this repository's own tree the
check names 5 rules as having judged nothing -- `review-finding`, the three shebang rules and
`uncompiled-conditional-branch` -- and over hex's calibration fixture
`tests/integration/tests/calibration.rs` asserts 19, row by row. In each of those three renderings
every one of them read as a clean judgment, which is the condition unmet everywhere but one place.

### Decision 4: every default rendering of a verdict names them beside the claim, and says nothing new when none was

| Rendering | Where it names them | When none was empty |
|---|---|---|
| `nomos check`'s text report | after the claim and its coverage, under `N rule(s) judged nothing, because no source was in their population:`, one rule a line (version 2) | it prints nothing more |
| `nomos gate run`'s text report | last, after the counts, the exceeded baselines and the unmatched policy, in the same words | it prints nothing more |
| the SARIF log `nomos gate run --sarif` and `nomos check --sarif` write | `runs[0].properties.emptyPopulations`, an array of rule ids, each resolvable against `runs[0].tool.driver.rules` | the property is absent |
| `nomos-api`'s check response | `empty_populations`, beside `claim` in the `judged` outcome | the field is absent |
| `nomos-api`'s gate response | `check_outcome.empty_populations`, beside `complete` | the field is absent |

None of them is behind a flag, a verbosity level or a verb of its own. Each reads
`nomos_check_orchestration::Populations::Empty` off the check outcome and counts nothing a second
time. None of them reaches `Claim`, `Claim_Of`, the gate's disposition, an exit code, or the
SARIF invocation's `executionSuccessful`, so decision 2's answer holds on every surface: the claim
is what the findings support, and the empty populations are reported beside it.

Absent rather than empty when there is nothing to say, on the two machine surfaces as on the two
text ones. A run in which every selected rule judged something is rendered byte for byte as it
was before this amendment, which is the property each rendering's test pins.

### Where the SARIF log carries it

SARIF 2.1.0 admits three places, and two are refused.

- **Not a `result`.** An empty population is not a finding, which decision 2 already says when it
  takes the `GateCategory` away. It has no subject, no location and no level, and a code-scanning
  view would raise an alert for a rule that looked at nothing.
- **Not one of the invocation's `toolExecutionNotifications`.** In this projection a notification
  there is what makes `executionSuccessful` false, and `executionSuccessful` is the log's
  rendering of the claim. A notification beside a successful execution would break the
  invocation's own invariant, and one that flipped it would be the flip the owner ruled out.
  `toolConfigurationNotifications` is no better: it describes the tool's configuration, and this
  is a fact about the tree.
- **A property of the run.** §3.8's property bag on §3.14's `run` is neither a result nor part of
  the invocation. It is read by key, it sits beside `invocations` rather than inside them, and it
  names rules by the ids the run's own driver lists, so a consumer can follow each to the record
  that rule cites. This is the placement decided.

### What proves it

Each rendering is proven by a test that renders a run in which one rule judged an empty
population and another judged real sources, and asserts that only the first is named and that
everything else the rendering says -- the claim among it -- is what it says with no population
reported at all:

- `nomos gate run`'s report: `Test_Render_Run_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Verdict`.
- the SARIF log: `Test_A_Gate_Runs_Log_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Verdict`
  and `Test_A_Check_Runs_Log_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Claim`.
- `nomos-api`'s check response: `Test_A_Judged_Response_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Claim`.
- `nomos-api`'s gate response: `Test_A_Judged_Outcome_Should_Name_Only_The_Empty_Population_Beside_An_Unchanged_Claim`,
  and through `Handle_Gate_Run` over a real tree,
  `Test_A_Rule_That_Judged_Nothing_Should_Reach_A_Headless_Caller_Beside_An_Unchanged_Verdict`.

Each was shown able to fail by naming every judged rule in place of the empty ones.
`P186-THE-GATE-REPORT-SARIF-AND-THE-API-DO-NOT-SAY-WHICH-RULES-JUDGED-AN-EMPTY-POPULATION` built
this amendment.

## Status

Accepted, version 3. An empty population is reported apart from a clean judgment, in
`CheckOutcome::Judged` itself, and it never flips `Claim`. The owner decided on 2026-09-27 that
a repository holding no file of a rule's language or kind can be `Complete`. This amendment
gives every other cause the same answer, because no count separates the owner's case from a
walker defect, and every failure a run can detect already flips the claim through a finding
of its own.

A rule's population is declared on its descriptor, as the one statement its body filters by,
and defaults to every source the rule is handed. The root counts it and partitions nothing,
which is the only placement `OD-RULES-014`'s refusal of a root-side partition leaves. Counted at
`0802e5b6`, 18 of the 76 composed rules judge an empty population in this tree and in hex's
calibration fixture alike. Three of the four script rules `P43-SCRIPT-RULES-CANNOT-FIRE`
measured are among them; `declared-tooling-language-for-scripts` judged all 2,134 of this
tree's walked sources and is not.

Version 1's claim that all four script rules report an empty population, its `Claim` flip with
the Advisory disposition that went with it, and its reading of the slice `run_context` holds are
superseded. `P172-A-RULE-DECLARES-THE-POPULATION-IT-JUDGES-AND-A-RUN-REPORTS-EVERY-EMPTY-ONE-BESIDE-ITS-CLAIM`
built decisions 1 and 2.

The owner ratified decision 2 on 2026-10-03, on the condition that the rules that judged an empty
population are prominent in the default output and never behind a flag. Version 3's decision 4
meets it: `nomos check`'s and `nomos gate run`'s text reports, the SARIF log both write, and
`nomos-api`'s check and gate responses each name every such rule beside the claim, and say
nothing new when there is none. The SARIF log carries them as the run's own
`properties.emptyPopulations`, never as a result and never as a notification that would move
`executionSuccessful`.
`P186-THE-GATE-REPORT-SARIF-AND-THE-API-DO-NOT-SAY-WHICH-RULES-JUDGED-AN-EMPTY-POPULATION` built
it.
