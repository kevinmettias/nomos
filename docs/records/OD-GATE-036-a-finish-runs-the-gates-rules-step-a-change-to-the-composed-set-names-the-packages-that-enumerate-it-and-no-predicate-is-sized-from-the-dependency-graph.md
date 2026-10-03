---
id: OD-GATE-036
type: decision
title: A finish runs the gate's Rules step, a change to the composed set names the packages that enumerate it, and no predicate is sized from the dependency graph
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - work-ledger
  - verification
relations:
  - target: OD-LEDGER-003
    type: affects
  - target: OD-GATE-004
    type: relates-to
  - target: OD-GATE-027
    type: relates-to
  - target: OD-GATE-033
    type: relates-to
  - target: OD-GATE-035
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
---

# A finish runs the gate's Rules step, a change to the composed set names the packages that enumerate it, and no predicate is sized from the dependency graph

## Question

`work finish` runs two things: the gate's `Lint` step, derived from `.github/workflows/gate.yml`
as `OD-LEDGER-003` decided, and the item's own predicate. A predicate names the packages its
author chose, which is usually the packages the change touches. `OD-LEDGER-003` left the rest of
the workspace to "the gate itself, at push time", and `OD-GATE-027` records that the gate has run
nowhere since 2026-09-07, because GitHub refuses the workflow on a billing failure. So nothing runs
a dependent package's tests, or the gate's `Rules` step, unless some session happens to.

Five landings between 2026-09-22 and 2026-09-27 passed their own predicate and left `HEAD` red in
a place no predicate ran. What should catch a change like that, at what cost, and where should it
be built?

The owner was offered four options, (a) to (d) below, and asked that the work proceed
autonomously. The decision was delegated to the claimant of
`P188-A-PREDICATE-COVERS-ITS-OWN-PACKAGES-AND-FIVE-LANDINGS-REDDENED-A-DEPENDENT-NOBODY-RAN` and
is made here, on the measurements below.

## The five landings

Each is the commit that landed the change, the package or step it left red, what repaired it, and
how long `HEAD` stayed red, from the landing commit's timestamp to the repair's.

| # | Landing | What it changed | Left red | Repaired at | Red for |
|---|---|---|---|---|---|
| 1 | `88306432`, `P123-CSHARP-JOINS-THE-RUN-AND-THE-WALK-2` | the shared walk learns the `cs` extension | nomos-cli's profile tests, `Test_A_Rich_Root_Should_Render_Every_Row_The_Profile_Carries` among them | `5cff7cd9` | 5.0 h |
| 2 | `fff5408c`, `P149-A-DECLARED-STANDARDS-CORPUS-IS-READABLE-WHERE-IT-IS-DECLARED` | composes `standards-corpus` | nomos-integration-tests' `Test_Table_Names_Exactly_The_Composed_Rule_Set` | `160a3ac5` | 12.1 h |
| 3 | `7edd095f`, `P123-THE-COMPILER-CONTRACTS-LEAVE-THEIR-PROVIDER-AND-ITS-RULES-JOIN-THE-RUN-2` | composes `copy-clones` and `nested-locks` | nomos-cli's `Test_An_Admitted_Gap_Should_Be_Reported_Without_Failing` | `160a3ac5` | 109.4 h |
| 4 | `be59c109`, `P128-A-JUDGMENT-LEAVES-IN-ONE-FORMAT-AND-EVERY-OTHER-CONSUMER-IS-UNSERVED-2` | a GraphML export and its test helper | the `Rules` step: one Blocking `abbreviations` finding | `b98e6846` | 4.3 h |
| 5 | `df79bad6`, `P128-C-SHARP-IS-READ-ON-ITS-FACE-AND-NO-COMPILER-ANSWERS-FOR-IT` | a new C# compiler crate, not yet composed | the `Rules` step: two Blocking findings in that crate | `2a42e117` | 5.4 h |

Three are dependent suites and two are the `Rules` step. Every dependent red was in one of two
packages, nomos-cli and nomos-integration-tests. Each red was found only because a later, broader
run happened to include it: a session measuring nomos-cli's suite for 1, which became
`P159-NOMOS-CLIS-PROFILE-TESTS-LIST-THE-LANGUAGES-BY-HAND-AND-C-SHARP-MADE-THEM-RED-AT-HEAD`;
the twelve-package predicate of `P127-THE-CHECK-SERVICE-RECEIVES-ITS-PROVIDERS-FROM-A-COMPOSITION-ROOT-3`
for 2 and 3; a hand-run `Rules` step for 4, which became
`P170-A-GRAPHML-TEST-HELPER-IS-THE-ONE-BLOCKING-FINDING-THE-GATES-RULES-STEP-REPORTS`; and
`P171-THE-CSHARP-CONDITIONAL-COMPILATION-FACT-JOINS-THE-RUN-FOR-THE-BUILDS-A-REPOSITORY-DECLARES`,
whose territory held the crate, for 5.

## What Was Measured

Everything below was taken on 2026-10-03 at `0000e0e3`, the `HEAD` this record was written
against, unless a row says otherwise.

### Conditions

- **Machine.** The i9-13900K `OD-GATE-035` describes, with the toolchain `rust-toolchain.toml`
  pins.
- **Tree.** A private detached worktree at `0000e0e3`, with its own target directory on F:. The
  nomos binary was built fresh there and copied outside the root before it ran. `GOROOT` named the
  full Go toolchain in `C:\Program Files\Go`, and no corpus variable was set.
- **Load.** This is not `OD-GATE-035`'s quiet machine, and the difference is the point of one of
  the figures. Other sessions were working throughout: during the gate runs a peer's
  `nomos check --root .` held about 2 GB and a rust-analyzer about a core; total CPU stood at 39 to
  45 percent without this record's runs, available memory at 5.3 to 8.1 GB, and the pager moved 7
  to 11 thousand pages a second. During the workspace run a peer ran nomos-gate-orchestration's own
  suite beside it. Finishes on this board happen under exactly this load, so it is measured rather
  than waited out.

### A real gate run, which is what (d) adds to a finish

`nomos gate run --root .`, one discarded warm-up and three measured runs:

| Run | Seconds | Exit | Findings |
|---|---|---|---|
| warm-up (cold clippy in the new target) | 292.2 | 0 | 475, none can fail a build |
| 1 | 209.3 | 0 | 475, none can fail a build |
| 2 | 210.5 | 0 | 475, none can fail a build |
| 3 | 236.7 | 0 | 475, none can fail a build |

Median **210.5 s**, against the 52.6 s `OD-GATE-035` measured on a quiet machine. The three
measured runs' outputs are byte-identical apart from the per-run `run:` line, so the difference is
load and not work. One run of each family under the same load: `copy-clones` alone 81.8 s,
`nested-locks` alone 41.0 s, the other 75 rules 63.8 s, against 17.2, 18.3 and 18.1 quiet. Every
family slowed, the compiler-backed two most.

**`HEAD` carries no Blocking finding**, so the `Rules` step exits 0 there and (d) would refuse no
finish over the tree as it stands.

**The `Rules` step catches landing 4, measured.** In the worktree, the three spellings of
`Assert_Well_Formed_Graph_Document` were renamed back to `Assert_Well_Formed_GraphML`, which is
the helper `be59c109` landed and `b98e6846` renamed. `nomos gate run --root .` then exited 1 in
92.5 s with 476 findings, exactly one of them Blocking: `abbreviations` on that helper. The file
was restored and its blob compared equal to `HEAD`'s. Landing 5's two Blocking findings are the ones
`OD-GATE-035` recorded at `d0581b4e`, where every full run "exited 1 because of them", so the
`Rules` step catches it too, from that record's own measurement.

### The whole workspace, which is what (c) runs and what (a) approaches

`cargo test --workspace --no-run` built 239 test executables in 69.4 s with dependencies already
built. `cargo test --workspace --no-fail-fast` then ran for **7,202.9 s** and exited 0: 5,769 tests
passed, none failed, 4 ignored. Wall time per package, attributed from each test binary's start to
the next one's:

| Package | Seconds |
|---|---|
| nomos-gate-orchestration | 3,897.5 |
| nomos-cli | 1,021.9 |
| nomos-check-orchestration | 870.9 |
| nomos-api | 729.9 |
| nomos-workspace | 367.7 |
| nomos-integration-tests | 99.5 |
| the other 81 members together | 201.5 |

Four packages are 90.7 percent of the run, and each of them judges this repository's own tree for
real, several times. nomos-cli's binary tests alone took 909.4 s.

### Who depends on whom, which is what (a) reads

The reverse-dependency closure of a change is every member that depends on a package it touches,
transitively through normal and build dependencies, plus every member that names one of those as a
dev-dependency. It was computed for each landing from that commit's own manifests, read out of git,
with each changed path assigned to the member that owns it.

| # | Packages touched | Closure | Closure's test time | Own predicate's packages' test time | Red package in the closure? |
|---|---|---|---|---|---|
| 1 | 4 | 15 | 6,660.7 s | 890.4 s | yes, nomos-cli |
| 2 | 5 | 17 | 6,661.5 s | 4,800.6 s | yes, nomos-integration-tests |
| 3 | 7 | 18 | 6,668.0 s | 996.8 s | yes, nomos-cli |
| 4 | 2 | 7 | 5,665.1 s | 3,897.5 s | yes, nomos-cli |
| 5 | 4 | 4 | 120.3 s | 20.8 s | **no** |

The times are this record's per-package figures, so they are `HEAD`'s suites standing in for each
landing's, and a predicate that ran fewer targets than its whole packages cost less than its row.

Two facts decide most of what follows.

- **nomos-cli's tests are the `Rules` step and more.** Its binary tests include
  `Test_Host_Variant_Should_Compose_Into_A_Real_Run_That_Judges_This_Workspaces_Own_Tree`, which
  runs `gate run` over this repository and requires exit `Ok` with no Blocking finding, and its
  `check_command` tests include `Test_This_Workspace_Should_Have_Nothing_That_Can_Fail_A_Build`.
  Both existed in that form, with nothing accepted, at all five landing commits. So any predicate
  that runs nomos-cli catches a Blocking finding anywhere in the tree, at about five times the
  `Rules` step's cost under the same load. Measured: with landing 4's helper name reintroduced as
  above, the first of those tests failed on exactly that Blocking finding. That run took 1,916.6 s
  and is not a cost figure: from about 03:31 a lock on cargo's shared package cache, held by a
  process outside this measurement, stalled the run's nested clippy for 1,800 s and then a 600 s
  idle bound.
- **The dependency graph cannot see landing 5.** Its crate was new, and nothing but the two test
  packages depended on it yet. 70 of the 87 members have nomos-cli in their closure at `HEAD`; a
  crate on its first day is usually one of the other 17. A Blocking finding in it is still a
  Blocking finding in the tree.

### The board, which says how often each option would have paid

Every item finished from 2026-09-20 to `HEAD` with a `cargo test` predicate: **172 finishes over
thirteen days**, between 1 and 99 a day. 136 of them reserved territory owned by some member.

- **Against (a).** 27 of the 136 predicates already covered their closure. Closures held a median
  of 11 packages, predicates a median of 2. 91 predicates omitted nomos-cli while the closure held
  it, and 72 omitted nomos-integration-tests. Priced with the per-package figures above, the
  closure would have added a median of **1,887 s** to each of the 136 finishes, 5,770 s at the
  75th percentile, and **98.8 h** in all, against 36.1 h for the predicates' own packages.
- **Against (b).** 42 of the 172 reserved a path under `crates/rules`, `crates/capabilities`,
  `crates/languages`, `crates/packages`, `crates/repository`, `crates/composer`,
  nomos-check-orchestration or nomos-workspace-discovery. Three of those already named both
  packages. Adding the missing ones would have cost **11.8 h** in all.
- **Against (d).** 172 finishes at 52.6 to 210.5 s each is **2.5 to 10.1 h** in all.

Two items on the board as this was written reach those paths and do not name nomos-cli:
`P192-A-FINDING-STATES-THE-VALUE-IT-WAS-JUDGED-AGAINST-AND-NOT-WHERE-IT-CAME-FROM` and
`P194-THE-RUST-FUNCTION-NAMING-RULE-NEVER-READS-THE-KEYS-XVPE-DECLARES-ITS-FUNCTION-CASE-UNDER`.
They predate this decision and are left as they are.

## The options

### (a) Size each predicate from the reverse-dependency closure

**Catches 1, 2, 3 and 4. Misses 5.** It catches 4 only because nomos-cli is in that closure and
its tests run the gate for real, not because the graph knows anything about findings.

**Costs** about the whole workspace for exactly the changes it exists for. The three
composed-set landings' closures each ran 6,660 to 6,668 s, 92.7 to 92.8 percent of the
workspace's 7,188.9 attributed seconds, and the board's 136 finishes would have paid 98.8 h more,
2.7 times what their own predicates cost. It needs a cargo-metadata reader and a
territory-to-package map in or beside a ledger that KWB also uses and that knows nothing about
cargo today, and a territory path no member owns, such as `Cargo.lock` or `README.md`,
contributes nothing to it.

**Refused.** `P188` refused raising predicates to the whole workspace unless its cost was measured
and accepted. It is measured here and not accepted: a median of 31 minutes on every finish, for a
catch that one cheap step and one narrow rule below reproduce, and that still misses the class the
graph cannot see.

### (b) A change to the composed set names nomos-cli and nomos-integration-tests

**Catches 1, 2, 3 and 5. Misses 4.** Landings 1 to 3 changed composed-set paths, and their reds
were in exactly these two packages. Landing 5 changed `crates/languages` and `crates/capabilities`,
so nomos-cli would have run, and its real-tree tests fail on a Blocking finding. Landing 4 changed
only nomos-gate-orchestration's export, which composes nothing.

**Costs 1,121.4 s** on each finish that triggers it, nomos-cli's 1,021.9 s and
nomos-integration-tests' 99.5 s, and nothing on any other. That was 42 of 172 finishes and 11.8 h
over the window. The machinery is a repository declaration, a refusal when an item is added or
widened, and the declaration's own drift: its paths and packages are a hand-kept list.

**Adopted**, as an authoring rule and not as a step finish runs. See the decision.

### (c) A scheduled whole-workspace run, whose red becomes an item

**Catches all five, after they land.** It shortens a red to at most its period plus its run, and
prevents none.

**Costs** at least **7,482.8 s per run** under this load: 69.4 s to build, 7,202.9 s to test and
210.5 s for the `Rules` step, before the steps not timed here. At one run a day that is about 27 h
over the window and at two about 54 h, more than (b) and (d) together. It also needs things this
repository cannot hold: a scheduler on the machine, which is the owner's configuration rather than
a file in the tree, and an author that boards an item for a red, which has to guess a territory for
a failure nobody has diagnosed.

**Refused for this question.** Against the five landings it costs more than (b) and (d) and
catches later. What it would buy beyond them is the gate's other steps running at all while CI is
dark: Boundaries, Supply chain, the floors, the crossing and the projections. That is
`OD-GATE-027`'s question and this record does not decide it.

### (d) A finish runs the gate's Rules step as well as its Lint step

**Catches 4 and 5. Misses 1, 2 and 3**, which are test failures and not findings. Landing 4 is
measured above by reintroducing it at `HEAD`; landing 5 by `OD-GATE-035`'s own runs.

**Costs 52.6 s** on a quiet machine and **210.5 s** under the load this board actually runs at,
on every finish: 2.5 to 10.1 h over the window. To that add whatever `cargo run` builds first,
nothing when the binary is fresh, about 3 s after an edit to one member, and a cold build of about
67 s in an empty target, all three from `OD-GATE-035`. The machinery is one more step name beside
`Lint`, read by the same derivation, and one more recorded outcome.

**Adopted.** It is the only option that sees a Blocking finding in a crate nothing depends on, and
it sees every one of them for about a fifth of what nomos-cli's tests cost under the same load.

## Decision

**(b) and (d) together.** Between them they catch all five landings before they land, at 14.3 to
21.9 h over the window, against 98.8 h for (a) alone and at least 27 h for (c) alone. Each covers
what the other misses: (d) takes the `Rules` step everywhere, and (b) takes the two dependent
suites where a change can reach them.

### 1. A finish runs the `Rules` step, derived from the gate

`work finish` runs the workflow's step named `Rules` after its `Lint` step and before the item's
predicate. It is derived the way `OD-LEDGER-003` derives `Lint`, by name and from the file, never
copied. Six parts, each one load-bearing.

- **Zero is the only success.** `OD-GATE-004` decided it for the step and nothing here relaxes
  it. Exit 1 is a Blocking finding, 6 is a run that judged nothing, and 5 is a run that could not
  be assembled. Every one of them refuses the finish as a failed gate step, with the step's argv and
  the tail of its output, which names the finding.
- **The order is `Lint`, then `Rules`, then the predicate**, and the first failure ends the finish,
  which is `OD-LEDGER-003`'s third part extended rather than restated.
- **A workflow that declares no `Rules` step runs none, and the finish says so.** This ledger
  serves KWB too, whose gate declares `Lint`, `Test` and `Contract` and no `Rules`. A repository
  that never declared the step has made no claim for a finish to honour, so its absence is not a
  refusal. It is recorded as absent, never as passed.
- **A `Rules` step that cannot be derived refuses.** A scripted body is `GateUndetermined`, exactly
  as it is for `Lint`, because a guessed command looks like a checked one.
- **It runs under the item's own bound**, the same `Runner` the `Lint` step uses. That means half
  the item's timeout as an idle bound, and `gate run` prints nothing until it finishes. At the 600 s
  default the idle bound is 300 s, above every figure measured here. Eight predicates on the board,
  finished or not, carry 300 s or less, and those would be cut off at 150 s or less under load.
- **The outcome is recorded on the verification record, never backfilled**, like `gate` beside
  it. A new field means every copy of the binary taken before it refuses the ledger once, by
  `OD-LEDGER-008`'s guard, and that is accepted.

**What it costs a session, beyond seconds.** The step judges the tree it runs in. In the shared
tree that includes every peer's uncommitted file, so a peer's in-flight Blocking finding refuses an
unrelated finish. The `Lint` step already has the same exposure, since clippy lints the whole
workspace, and the remedy is the one `.claude/skills/nomos-spec-change` already prescribes:
finish from a worktree at the commit you will publish. A Blocking finding that reaches `HEAD`
refuses every finish until it is repaired. That is the intended pressure, and it is the same one
`Lint` applies.

`OD-LEDGER-003` says finishing runs the gate's lint step and that the test step stays scoped. The
second clause stands. The first becomes "the gate's `Lint` and `Rules` steps", by an amendment the
building item makes.

### 2. A change to the composed set names the packages that enumerate it

An item whose territory reaches a declared path must carry a predicate that names each declared
package, or `work add` refuses it. `work widen` into such a path refuses the same way, because a
predicate cannot be edited after the item exists.

- **The rule is declared by the repository, not written into the ledger.**
  `nomos-predicate-coverage.json` at the repository root, read where `work add` already learns
  which records are published, declares each rule: the paths that trigger it, the arguments a
  predicate must carry, and any argument that satisfies it on its own, such as `--workspace`. The ledger compares argument tokens and does not
  interpret cargo. A repository with no such file has no such rule, so KWB is untouched.
- **This repository declares one rule.** Its paths are `crates/rules`, `crates/capabilities`,
  `crates/languages`, `crates/packages`, `crates/repository`, `crates/composer`,
  `crates/orchestration/nomos-check-orchestration` and
  `crates/orchestration/nomos-workspace-discovery`, because those are where the composed rules,
  providers, policies and walk are defined. Its packages are nomos-cli and nomos-integration-tests,
  because every measured dependent red was in one of them and both enumerate the composed set.
- **Authoring time, not finish time.** At authoring, the author sees the 1,121 s and chooses the
  item's timeout with it in view. A step added to finish would run under a bound chosen without it
  in view, and a 600 s item would time out on it.
- **What keeps the list honest is existence, not completeness.** A test can hold that every
  declared path exists and every declared package is a member, so a rename cannot leave the rule
  silently matching nothing. Whether the list is complete cannot be tested; a measured red outside
  it is what amends it.

### Items

The building is not done under `P188`. Two items carry it, each depending on `P188`:

- `P195-WORK-FINISH-RUNS-THE-GATES-RULES-STEP-AFTER-ITS-LINT-STEP` builds part 1 and amends
  `OD-LEDGER-003`.
- `P196-A-CHANGE-TO-THE-COMPOSED-SET-NAMES-THE-PACKAGES-THAT-ENUMERATE-IT` builds part 2.

## What Holds It

This record's own figures are held by the measurements above, not by a test, and the commands are
named so that anyone can retake them. The two building items each carry tests: one that a workflow
naming a `Rules` step makes a finish run it, refuse on its exit, and record it, and that a workflow
without one finishes with the step recorded as absent; one that an add or a widen reaching a
declared path without a declared package is refused, that a predicate carrying it is accepted, and
that the declaration's paths and packages exist.

## What Would Reopen It

- **GitHub CI running the gate again.** Its `Test` step would then catch a dependent red within one
  push, and (b)'s 1,121 s per triggered finish would be weighed against a catch after landing
  rather than against none. (d) is not reopened by that alone, because CI also catches only after
  landing. (c) becomes CI itself.
- **A dependent red outside (b)'s list**: a package other than the two, or a change from a path
  the rule does not name, measured red at `HEAD` after a predicate that satisfied the rule. One is
  an amendment to the declaration. A second of a different shape reopens (a) and (c).
- **(d) refusing finishes for findings that are not the finisher's**, measured as a share of
  refusals, once it is built. If the worktree remedy proves not to be enough, the answer is a
  comparison against `HEAD`'s own run, which `gate compare` already computes, and not dropping the
  step.
- **The `Rules` step's cost moving.** A re-measurement whose median under ordinary load passes the
  default item's 300 s idle bound, or the compiler-backed families leaving the process, which
  `OD-GATE-035` already names.
- **The `Rules` step stopping being one bare command.** It would become underivable, and a finish
  would refuse, which `OD-GATE-004` already forbids.

## What This Record Does Not Decide

- **Whether a scheduled run should stand in for CI's other steps while CI is dark.** That is
  `OD-GATE-027`'s and `OD-GATE-033`'s question, and nothing in the five landings measured it.
- **That a finish runs the gate's `Test` step.** It does not, for the reason `OD-LEDGER-003` gave and
  this record's 7,202.9 s confirms.
- **Adding nomos-contract-tests to (b)'s packages.** Its suite costs 12.3 s and it has been
  reddened by a new declared list before, but none of the five was that, so it is an amendment for a
  measured landing of its own.
- **The declaration's syntax**, which belongs to
  `P196-A-CHANGE-TO-THE-COMPOSED-SET-NAMES-THE-PACKAGES-THAT-ENUMERATE-IT`, within the semantics
  above.
