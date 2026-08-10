---
id: OD-GATE-004
type: decision
title: The rule layer becomes a gate step, and zero is the only exit code that passes it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - continuous-integration
  - rules
  - enforcement
  - verification
relations:
  - target: OD-GATE-001
    type: relates-to
  - target: OD-GATE-002
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-002
    type: relates-to
  - target: OD-COMPLETENESS-001
    type: relates-to
  - target: D-134
    type: relates-to
---

# The rule layer becomes a gate step, and zero is the only exit code that passes it

## Why This Identifier

`OD-GATE-003` is not free. No file bears it, but `P10-VACUITY-HOME` reserves both
`docs/records/OD-GATE-003` and `crates/spec/nomos-spec-store/records/OD-GATE-003.record` in its
territory, and `OD-LEDGER-016` decided that a record identifier and the file it names are one
subject. Taking 003 because no file exists yet would be reading a reservation as vacant because
it has not been spent. So this is 004.

## Question

`nomos check` is the only thing in this repository that judges code rather than paperwork. It
runs, it produces findings, it distinguishes five outcomes by exit code — and until this record
**nothing in CI ran it.** `D-134` deferred wiring it and said so. `OD-RULES-001` re-deferred it
and sharpened the question by making the command a composition root that declares a capability,
registers a provider and materializes one fact per file: a gate step that could not materialise
a fact must fail rather than report clean, and nothing was running the command, so nobody would
have known either way.

So the question is not whether to wire it. It is what a wired step is allowed to claim. Three
things had to be decided, one had to be measured, and one thing that looked like a detail turned
out to be the reason the item was worth doing.

- Which exit codes fail the step. `1` is obvious. `6` — a run that judged nothing — is the one
  that decides whether the step is worth having.
- Whether the workflow restates the codes or defers to `check.rs` for their meaning.
- What happens to `tests/corpus/analysis/gamma/broken.rs`, a committed and deliberately
  unparseable fixture, once every CI run judges it.
- Measured: whether the step, on the day it lands, can fail for the reason it exists.
- And: whether the assertion this repository already had for that failure was worth anything.

## What Was Measured

Every figure below was taken in this session with **no `NOMOS_*` variable set** — the condition
CI runs under — against binaries built in private target directories, over `git archive`
extractions, because a bare archive extraction is what `actions/checkout` produces. Two commits,
because the answer to the central question changed between them.

The phantom is one added line: ``/// Mirrored by `Test_This_Check_Does_Not_Exist_Anywhere`.`` on
`RowKind::All` in `crates/spec/nomos-spec-model/src/table.rs`, a file the parser reads perfectly
well. Nothing else differs between the two columns.

**At `a5da564`, the commit this step was wired on:**

| | this tree | one phantom planted |
|---|---|---|
| files examined | 194 | 194 |
| files with a syntax fact | 193 | 193 |
| findings | 14 | 14 |
| findings that can fail a build | **0** | **1** |
| exit code | **0** | **1** |

Green on the unplanted tree, red on the planted one. The fourteen findings are twelve
unmirrored universes — advisory by `D-134` decision 4 — and the two `broken.rs` lines; the
planted run reclassifies one of the twelve rather than adding a fifteenth, because the plant
claims a mirror on a universe that was already an admitted gap.

The counts in `P10-CHECK-GATE`'s `why` — thirteen unmirrored universes — and in the design
written against `6a585b8` — 192 files, 191 facts — are stale and are **not** carried forward.
Twelve, 194, 193, re-measured here. The figure the item rests on, *0 blocking over an unplanted
tree*, is confirmed.

All five exit codes propagate exactly through the invocation the step uses,
`cargo run --quiet -p nomos-cli --bin nomos -- check …`: `0` over this tree, `1` over the
planted one, `2` for `--rooot x`, `5` for a root that is not a directory, `6` for an empty
directory. Re-measured, not carried.

**And the step itself was run.** Not the command in a test — the step's exact argv, from the
root of a clean extraction with this commit's files in it, with nothing set:

```text
195 file(s) examined, 194 with a syntax fact, 14 finding(s), 0 of which can fail a build
```

exit `0`. One more file and one more fact than the table above because this commit adds
`crates/host/nomos-cli/tests/gate_step.rs`; the twelve advisories are the twelve unmirrored
universes and the remaining two are `broken.rs`'s. A step nobody has run is a step nobody has
checked, so this figure is the one a first pull request should reproduce.

### The assertion this repository already had, and what it was worth

`crates/host/nomos-cli/tests/check_command.rs` held `Test_A_Phantom_Mirror_Should_Fail_The_Command`,
called "the headline" in its own doc comment, asserting exit `1` over a **one-file** tree holding
a phantom. It was green. The tree this step judges is never one file, and it always holds
`broken.rs`.

Measured at `fb55790`, the commit before `OD-RULES-002`, with a binary built from that tree:

| fixture | exit | the phantom rendered |
|---|---|---|
| the repository's one-file fixture | `1` | `[Blocking]` |
| the same fixture plus a byte-for-byte copy of `broken.rs` | **`0`** | **`[Advisory]`** |

So the binary-level headline test passed for a reason that did not hold on the real workspace.
It was true of its fixture and false of the tree a gate step would judge, and a CI step wired
then would have rested on it. `OD-RULES-002` fixed the code and added the two-file case as a
`check.rs` unit test through `check::Run`; it did not touch `check_command.rs`.

At `a5da564` the two-file fixture exits `1` with the phantom `[Blocking]`. So the property is
sound now, and this record is where the *fixture* stops modelling a tree nobody has — because
this is the item that makes the binary's exit code a CI outcome, and **a gate step is worth
exactly what its headline test is worth.**

## The Decision

**One step, appended after `Required projections`, whose `run:` is one bare command, and zero is
the only exit code that passes it.**

```yaml
      - name: Rules
        run: cargo run --quiet -p nomos-cli --bin nomos -- check --root .
```

### 1. Zero is the only success, and the workflow does not say so

Actions fails a step on any non-zero exit, and that default *is* the policy. `1` fails because a
finding can fail a build. `6` fails because a run that judged nothing and a run that judged
everything and approved are the two states this binary exists never to render the same. `5`
fails because a tree that could not be read has not been checked. `2` fails because, from a
workflow step, a usage error means the workflow's own argv is wrong, and a gate that mistypes
its invocation and passes is checking nothing. Anything else — `101` from a build failure or
from the composition root's own `expect`, a signal, a truncation — fails, because absence,
unknown and error must not become success.

### 2. The codes are not restated in YAML, and adding a branch would be a regression

A `case $?` block listing 1, 5 and 6 in the workflow would be a second place for one contract to
be spelled — the objection `OD-LEDGER-003` makes about the lint command and `OD-GATE-002` about
the surface snapshot. `check.rs`'s `ExitCode` and its usage text are the authority; the step's
comment points at them rather than copying them.

This reads `P10-CHECK-GATE`'s `done_when` narrowly, and the narrowing is stated rather than
slipped past. That sentence asks the workflow to distinguish "the three outcomes the binary
already distinguishes". **The workflow distinguishes two — green and red. The run distinguishes
three, in the step's own log**, on different streams and in different words: the counts line
with `0 of which can fail a build`; one `[Blocking]` line per finding plus a non-zero count; or,
on stderr, ``no Rust source found under `.`, so nothing was judged`` or `N file(s) were read …
and no syntax fact was materialized for any of them`, each followed by a sentence saying what a
clean result would have meant.

A YAML branch would add nothing a reader does not already have, and would cost the next sentence
of the same `done_when` — that the codes be "read from there rather than restated". Both halves
cannot be honoured by a branch, and the half that survives is the one that keeps the contract in
one place. It would also need `run: |`, which `Derive_Step` refuses outright. **A later author
should not add a branch here to be helpful.**

### 3. The step is named `Rules`, its `run:` stays one bare command, and both are load-bearing

`OD-LEDGER-003` decided that `work finish` derives its lint argv from
`.github/workflows/gate.yml` rather than copying it, and `nomos_ledger::Derive_Step` finds that
argv **by the step's name**. Two consequences, and they are requirements on any future edit
because breaking either is silent:

- **A step in this file must never be named `Lint`.** If this one were, `finish` would derive
  `cargo run … check --root .` and run *that* as the gate's lint step before every item's
  predicate. Clippy would stop running at finish time and nothing would say so.
- **A scripted `run:` is refused rather than guessed** (`GateUnknown::NotASingleCommand`), so
  keeping this step a bare argv keeps the file uniformly derivable — which is what lets a test
  assert the step *exists* by deriving it rather than by grepping for its text.

Measured against the shipped `Derive_Step`: with this step appended, the derived `Lint` argv is
byte-identical to what it was, and the same holds with the step written as a script, with the
step inserted above `Lint`, and with a block-scalar body containing the line `echo "name: Lint"`.
Adding a step does not change what `finish` runs. Confirmed here by `work finish` deriving and
running the clippy argv on this very item.

**One residual hazard, named because it is the only way this can bite.** A *scripted* step placed
**above** `Lint`, whose block-scalar body contains a line whose trimmed form begins `name: Lint`
followed by a line beginning `run:`, is read by `Derive_Step` as a step declaration and
mis-derived. A bare-argv step appended last cannot reach it. Do not turn this step into a script
and move it up the file in the same commit.

### 4. `broken.rs` is walked, tolerated, and load-bearing

It contributes exactly two `[Advisory]` findings to every run — one from the rule's own parser
refusing it, one from the fact layer reporting no fact for it — and `Can_Fail_A_Build` consults
applicability as well as gate, so neither can ever fail the step. Two log lines is the whole
cost.

Excluding it from the walk was the tempting close and it is refused. It would buy a complete
check index, which is exactly what would have made the blocking arm reachable without
`OD-RULES-002` — and it would buy it by deleting the case the third state exists for.
`OD-RULES-001` built "the rule could not answer" for unreadable subjects and `OD-RULES-002`
refused the same close for the same reason. Three further costs are specific to a gate: the
completeness would hold *because of* the exemption, so the first genuinely unreadable file to
appear in this workspace would change CI's behaviour with nobody having touched CI; a path
exemption inside `check.rs` is the easiest place in a check to hide something, which
`OD-GATE-002` says in those terms; and the fixture's presence in the judged tree is what
demonstrates, on every run, that a phantom in a read subject blocks while an unreadable subject
sits beside it.

The two advisory lines are recorded here as **expected output**, so that a later reader does not
take them for a defect and delete the fixture. It cannot be quietly repaired either:
`tests/integration/tests/analysis_slice.rs` asserts the corpus README's `broken.rs | Parses | no`
row.

### 5. No corpus is required, and the fact layer arriving did not change that

Worth stating because the question became live with `OD-RULES-001` and the answer is not obvious.
The gate sets no environment — `.github/workflows/gate.yml` mentions no `NOMOS_*` variable — and
`crates/host/nomos-cli/src/main.rs`, the only file in the workspace that reads the environment,
passes none of it to `check`. The fact layer could plausibly have introduced an external
dependency and introduced none: the provider runs in process over text the walk already holds,
and the store is a `MemoryFactStore` created per run. What it did add is a **build**-time
dependency — `Host_Variant()` reads four `NOMOS_*` values captured by `build.rs` from cargo's own
environment — and a runner missing those cannot link the binary, so it fails at `Lint` rather
than here. Confirmed by running the command over a bare archive extraction with nothing set: the
full answer, 194 files, 193 facts, 14 findings, exit `0`.

## What This Step Is Worth, And What It Was Worth Before

A declared guarantee must not exceed what is demonstrated, and what this step demonstrates
changed at `52dbf1a`. Both states are recorded — the weaker one because it is why the step
waited, and because it is what holds again if that work is ever reverted.

**Before `OD-RULES-002`, the step could not fail on a phantom.** Measured: exit `0` under a
planted one over this workspace. It was not worthless, and the four things it caught then are
the four it catches *in addition* now:

- a run that materialises no fact exits `6` and fails, which is `OD-RULES-001`'s guard finally
  being consulted by something other than its own tests;
- an unreadable tree exits `5`;
- a rotted argv exits `2`;
- a panic out of the composition root's registry expectations exits `101`, where before this
  nothing ran the command at all and so a panicking root shipped green.

It also prints `194 file(s) examined, 193 with a syntax fact, 14 finding(s), 0 of which can fail
a build` on every pull request, which is `OD-GATE-001`'s "declare the size of the hole" applied
to the rule layer: the day 193 becomes 140, somebody sees it.

**After `OD-RULES-002`, it fails for the reason it exists.** Measured against committed code: the
unplanted tree exits `0`, the planted phantom reports `[Blocking]` with `1 of which can fail a
build` and exits `1`. That is the whole difference between a step that reports and a step that
enforces, and it is why the ordering between the two items was not a preference — `Territory`
answered it before anybody had to.

That is also why this item does **not** reserve the whole of `crates/host/nomos-cli` in its turn.
That crate is reserved whole by `P10-VACUITY-HOME` and in part by `P10-REQUIRED-PROJECTIONS`;
reserving it here would serialize the same items again one item later, which is `OD-LEDGER-007`'s
subject exactly.

## What A Failure Looks Like

`D-134` decision 6 says a finding is identified by the universe's name and never by its path. The
step needs nothing added to honour it. The whole of what a reviewer sees, measured:

```text
[Blocking] completeness-mirror: RowKind::All (`Test_This_Check_Does_Not_Exist_Anywhere` resolves
to no check, so this rule is declared enforced and never runs — and the check index is short 1
subject(s), none of whose text spells `Test_This_Check_Does_Not_Exist_Anywhere`, so no reading of
them could have declared it) — crates/spec/nomos-spec-model/src/table.rs

194 file(s) examined, 193 with a syntax fact, 14 finding(s), 1 of which can fail a build
```

The universe's name leads and is the identity; the path is a location after the em-dash; the
caveat about what the run did not see travels on the finding rather than instead of it.

One caveat, recorded because it will otherwise be mistaken for a defect: two findings in every
run over this workspace carry a *path* in the name position, the two for `broken.rs`. Those are
about a file rather than about a universe, their identity is the digest of the file's text, and
`subject_name` carries the path because that is what a reader needs. It is correct and is not to
be "fixed".

Emitting GitHub Actions annotations — `::error file=…::` — was considered and rejected. It would
put a CI vendor's protocol inside a binary with no other reason to know about one, and would be a
second rendering of a finding beside `Finding::Describe`.

## Two Things Measured While Guarding This, Which Change What The Guards Are

Both were found by running the controls red rather than by reading, and both are recorded because
the obvious guard is weaker than it looks.

**`Derive_Step` returns at the first match, so "the step must not be named `Lint`" is not caught
by deriving `Lint`.** Renaming this step to `Lint` in a copy of the real workflow leaves
`Test_The_Derived_Lint_Step_Should_Still_Be_Clippy` **green**, because the genuine `Lint` step
comes earlier in the file and the scan returns at the first `run:` inside the first matching
step. A duplicate `Lint` *above* the real one silently replaces linting; a duplicate *below* it is
invisible to the derivation. Both are wrong, and neither is a question of which command is
derived. So the guard that closes it is `Test_Only_One_Step_Should_Be_Named_Lint`, which counts
the name and cannot depend on position, and the assertions that actually fired on the rename were
the ones about the `Rules` step having gone missing.

**The exit-code census cannot live on `ExitCode` without a row in a file this item does not
hold.** `pub const fn All()` in an inherent implementation is a *declared universe* —
`nomos-rules` finds it by that exact name — so putting the list on `ExitCode` requires a row in
`tests/contract/tests/completeness_universes.rs` classifying it. Measured: with `ExitCode::All()`
added and a mirror claim naming a real test, the workspace stays at 14 findings and `nomos check`
resolves the claim; with the claim removed it becomes 15 and the universe is reported. Either
way `Test_The_Declared_Table_Should_Match_What_Is_Derived` and
`Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror` go red, naming `ExitCode::All` and
reporting seventeen scanned universes against sixteen classified. That is the rule working, not
an obstacle — a universe nobody classified is what `OD-COMPLETENESS-001` exists to stop — but the
file is outside this item's territory, so the census stays private in the test module, where it
is not a universe at all. **Promoting it is worth doing by whoever holds that file next**, and it
is the tidier shape.

## Two Guards For The Lint Derivation, Kept On Purpose

`crates/host/nomos-cli/tests/gate_step.rs` carries
`Test_The_Derived_Lint_Step_Should_Still_Be_Clippy`, a near-duplicate of
`Test_This_Repository_Gate_Should_Still_Yield_A_Lint_Step` in
`crates/substrate/nomos-ledger/tests/gate_covers_finish.rs` (around `:439`). Both are kept, and
the newer one carries a comment naming the older and saying what each catches, because without
that comment a later reader deletes one as a duplicate — and a duplicate without that comment
*is* a duplicate.

They fail for different authors, which is coverage at two altitudes rather than two answers to
one question. The ledger's sits in the crate that owns `Derive_Step` and fires when the workflow
rots under the derivation. This one fires when an author editing the workflow to add a step
reaches for the wrong name; it lives in the crate whose suite that author is already running and
which their own item's predicate executes. If one home is ever preferred, the ledger's is the
older and this is the one to drop.

## What This Does Not Do

- It does not make the rule sound about checks a macro generated. That bound is
  `Syntax_Requirement`'s `Assurance::Unknown` completeness and is untouched.
- It does not close the twelve unmirrored universes. They print as advisories on every run, which
  is the point of an admitted gap; blocking on them would make a gate that can never be green.
- It does not exclude `broken.rs`, add an exemption list, or add a flag that could narrow the
  judged tree. `--root .` is written out precisely so that narrowing it is visible in a diff.
- It does not derive the gate's `Test` step, or any step other than `Lint`. That remains
  `OD-LEDGER-003`'s deliberate gap.
- It does not decide where the vacuity guard belongs. `P10-VACUITY-HOME` holds that, and a gate
  step now depending on exit `6` is evidence for that item rather than an answer to it.
- It does not prove that Actions fails a step on a non-zero exit. That is the platform's
  contract, not this repository's, and a test of it would be a test of a fixture. What is
  asserted is that every code except `0` is non-zero, and that no step carries an excuse.
- It does not settle the step's cost in CI. Measured at about a second of wall clock locally with
  a warm target directory; unknown on a cold `ubuntu-latest` runner. The experiment is reading the
  step's duration on the first pull request after this lands.
- It does not verify that exit `6` renders as `exit code 6` in the Actions log rather than being
  collapsed. Immaterial to the policy — the step fails either way — and the experiment is a
  deliberately vacuous root on a pull request.

## Controls

Each was run red in a scratch checkout and restored, and the message each produced is what a
future author will see.

| Weakening | What fires |
|---|---|
| the `Rules` step deleted | "the gate workflow has no step named \`Rules\`, so the step an item's predicate is missing cannot be derived" |
| the `Rules` step renamed `Lint` | "2 step(s) in the gate are named \`Lint\`" — plus every `Rules` assertion |
| its `run:` made a block scalar | `Derive_Step` returns `NotASingleCommand`, which is the control on the assertion above |
| `--root .` narrowed to `--root crates/rules` | "the Rules step must judge the whole workspace", printing the derived argv |
| `continue-on-error: true` added to any step | `the gate excuses a step from failing via ["continue-on-error"]` |
| the workflow truncated | `3 named step(s) in the gate workflow` |
| `Vacuous = 0` beside `Ok = 0` | the compiler: `discriminant value 0 assigned more than once` |
| `Ok` and `Vacuous` swapped so zero means vacuous | `Ok exits 3, and the gate reads zero and only zero as success` |
| a sixth code added and not documented | `the usage text and ExitCode disagree about what this command can exit with` |
| a sixth code added and not enumerated | the compiler: `non-exhaustive patterns: ExitCode::Surprising not covered` |
| the `facts == 0` guard downgraded to `Ok` | `a run that materialized no fact judged nothing, and a clean result would mean only that the analysis never ran` |
| a phantom planted in this workspace | `something in this workspace can now fail a build, and the gate's Rules step is red` |

And the one that is falsifiable by a real prior state of the code rather than by a hypothetical
weakening: the strengthened headline exits `0` at `fb55790` and `1` at `a5da564`. Restoring a
run-wide incompleteness downgrade turns it red.

The vacuity guards are not optional and are named so they are not later read as noise: the
acceptance test over this workspace asserts a floor of 150 files examined, and `gate_step.rs`
asserts the workflow holds at least five named steps. Every other assertion in that file reads
the workflow as text, and an empty string contains no `continue-on-error`.
