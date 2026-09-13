---
id: OD-GATE-025
type: decision
title: include selects what a gate run reports and never what it judges
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - rules
relations:
  - target: OD-GATE-017
    type: relates-to
  - target: OD-GATE-023
    type: relates-to
---

# include selects what a gate run reports and never what it judges

## Question

`nomos gate run --include <one file>` reports `Blocking` findings about correct code that a
whole-workspace run does not. A caller reaches for `--include` to be told *less* and is told
something *false*, in the one category the tool asks to be trusted on.

`OD-GATE-017` decided the opposite direction for rules: selection is real, `Run` skips a
deselected rule's own computation rather than discarding its finding afterwards. So the
question is not merely how to stop the false finding — it is whether file selection may be
real in the same sense rule selection is, and if not, why the two differ.

## What Was Measured

Measured against this repository at `15bdbb74`, with the shipped binary.

**The invented finding, reproduced.** `nomos gate run --root . --include
crates/rules/nomos-rules/src/checks/orphan_modules.rs` reports:

> `[Blocking] no-orphan-modules: … is not reachable from its crate's module tree: no
> declaration names it, so rustc never parses it and nothing in it compiles, runs or is
> linted -- declare it, or delete it`

The file is declared. `crates/rules/nomos-rules/src/checks.rs:45` reads `mod orphan_modules;`.
The advice is to delete or re-declare correct code.

**The whole-workspace run disagrees.** The same tree, no `--include`: **zero**
`no-orphan-modules` findings. The two runs contradict each other about the same file.

**The cause.** `Scoped_Sources` in `gate_environment.rs` filters the walked source set
*before* `nomos_check_orchestration::Run` sees it. A rule answering a cross-file question
then answers it against a truncated world. `no-orphan-modules`' own module doc states the
assumption that makes narrowing safe — a file the walk did not collect can neither back a
declaration nor be reported as an orphan, so the two answers move together — and `--include`
is exactly what falsifies it: the file is collected, its declaring root is not.

**`--include` is not a speed lever, which is the measurement that decides the cost.** The
item that filed this named "costs `--include` its role as a speed lever" as the price of the
mechanism chosen below. There is no such role to lose:

- A whole-workspace `gate run` takes **4.6 seconds**.
- Narrowing does not narrow the expensive work at all. The narrowed run above still reported
  194 findings, including `lint-diagnostics` advisories for `crates/substrate/nomos-ledger`,
  `crates/substrate/nomos-scope-verification`, `tests/contract` and `tests/integration`, and
  every `dependency-policy` advisory — all about files outside the include set. Those come
  from provider subprocesses that never consulted the scope.

So scope today narrows exactly the rules it breaks (the ones reading walked source) and not
the ones that dominate the runtime. The present behaviour is not a principled trade between
speed and completeness; it is a filter applied at the one layer where it does harm and no
good.

**Which rules answer cross-file questions today.** `no-orphan-modules` (a file's
reachability is a fact about its crate's declaring tree, not about the file) and
`completeness-mirror` (a declared table resolves against checks elsewhere in the workspace).
Both are reserved by the item implementing this. Every other shipped rule judges a file, a
name, or a line from what that file itself contains.

## The Decision

**`--include` and `--exclude` select what a run reports. Judging is always whole.**

The walked source set reaches the rules entire. Scope is applied to the findings afterwards:
a finding is reported when any of its locations is in scope, and a finding carrying no
location is not narrowed by a path filter, because a path filter has nothing to say about it.

### Why this does not contradict `OD-GATE-017`

`OD-GATE-017` is untouched, and rule selection stays exactly as real as it made it. The two
are different because of one asymmetry, which is the whole answer to that precedent:

> **Selecting fewer rules cannot change a remaining rule's answer. Selecting fewer files
> can.**

Rules are independent judges over a shared world. Removing one leaves every other rule's
question and evidence exactly as it was, so skipping its computation is a pure saving and
the answers that remain are the answers a full run would have given. Files are the world
itself. Removing one changes what a cross-file rule *sees*, so a narrowed run does not
compute a subset of the full run's answers — it computes different answers to different
questions and labels them the same. "Real selection" for files would mean handing rules a
false world and reporting their conclusions as though the world were true.

That is not a weaker form of `OD-GATE-017`'s principle; it is the same principle. Selection
is real where it is honest, and a saving that changes the answer is not a saving.

### A scope that admits no source is still not a clean pass

If no walked file is in scope, the run reports `NoSource` and is `Indeterminate`, exactly as
it did before. The judging happens over the whole walk and is discarded, rather than never
running — what a caller is told is unchanged, and that is deliberate. A typed-wrong
`--include` must not read as a repository with nothing to say, which is what
`Test_A_Scoped_Out_Source_Should_Not_Be_Judged` has always protected, and moving the scope
off the source set gives no reason to weaken it.

This is also what keeps the change inside one crate. Reporting a *judged* outcome with an
empty in-scope finding set would have been defensible, and would have made a scoped-to-nothing
run exit clean rather than vacuous unless the host's exit-code mapping learned a new case — a
change to `nomos-cli`'s reporting, for no difference a caller could see.

### What a caller gains that it did not have

A narrowed report becomes genuinely narrow. Because scope now filters findings rather than
source, provider-backed findings about out-of-scope files stop being reported too — the
`lint-diagnostics` and `dependency-policy` advisories that a narrowed run previously printed
about unrelated crates. `--include` starts meaning the one thing a reader would guess it
means.

## What This Record Does Not Do

It does not make `--include` a performance control, and does not claim the run became faster.
Judging was already whole for the expensive providers; what changes is that it is now whole
for everything, and the 4.6-second measurement above is what makes that affordable to say
plainly rather than a cost to apologize for. If a real performance need appears, it is a
different question with a different answer — narrowing the *providers*, which is where the
time is — and it does not begin by lying to a rule.

It does not change `no-orphan-modules` or `completeness-mirror`'s logic. Their assumption
about narrowing becomes true again rather than needing repair, and only their docs move.

It does not introduce a partial-source signal, the mechanism the filing item named second. It
would preserve narrowing as a real saving at the cost of threading a parameter through
`RunContext`'s roughly 45 construction sites, and it earns that cost only if the saving is
real. The measurement above says it is not.

It does not touch rule selection, `RuleSelector`, or anything `OD-GATE-017` decided.

## Status

Accepted. A narrowed run and a whole run now agree about every file both judge, because both
judge the same world and differ only in what they print. `OD-GATE-017`'s selection-is-real
direction stands for rules, and the reason it does not extend to files is recorded rather
than left as an inconsistency: selecting fewer rules cannot change a remaining rule's answer,
and selecting fewer files can.
