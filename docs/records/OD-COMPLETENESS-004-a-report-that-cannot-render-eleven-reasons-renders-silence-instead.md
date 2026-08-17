---
id: OD-COMPLETENESS-004
type: decision
title: A report that cannot render eleven reasons renders silence instead, and silence reads as clean
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - verification
  - completeness
relations:
  - target: OD-COMPLETENESS-001
    type: affects
  - target: OD-RULES-001
    type: relates-to
  - target: OD-RULES-002
    type: relates-to
  - target: OD-GATE-004
    type: relates-to
---

# A report that cannot render eleven reasons renders silence instead, and silence reads as clean

## Question

`nomos_contracts::Applicability` opens by calling itself the load-bearing expression of the
product's first principle — unknown is not pass — and every `Finding` this workspace produces
already carries one. `crates/host/nomos-cli/src/check/report.rs`, the surface a person actually
reads, consumed `Finding::Can_Fail_A_Build` (which reads applicability) and never the field
itself. Its `Examined` struct carries two denominators on purpose, and its own doc comment
already makes the argument this record extends: "0 findings over 400 files" and "0 findings
over 400 files none of which produced a fact" are different claims. That argument stops one
step short of the eleven-way distinction `Applicability` exists to carry, and the gap is a
silence with a specific shape — a subject whose provider is missing, whose toolchain is
absent, whose parse failed, or which needs a model contributes nothing to either denominator
and produces no distinguishing line in the rendered report.

## What Was Already True And Is Kept

Two things this workspace had already got right, named so the remedy below does not quietly
re-decide them.

`nomos-rules` already puts an `Applicability` on every finding it produces, including the
debt states. `mirror/unread.rs` emits one finding per subject whose syntax fact could not be
read, before any universe is judged — `Check_Completeness_Mirrors` extends its findings with
`index.unread.iter().map(Unread_Subject)` unconditionally. So a subject is never dropped
*silently* at the rule layer today; what was missing sat one layer up, in the surface that
renders the findings into something a person reads.

`Examined`'s two denominators are a property of the walk — decided before a single subject is
judged — and are orthogonal to what judgment each subject received. That reasoning is
unchanged and the type is unchanged: `files` and `facts` still answer "did the walk see a
plausible amount of the world", which a per-subject breakdown cannot answer on its own (a
tree the walk never entered has no subjects to break down).

## The Decision

**The report reads the enum.** `report.rs` gains `Coverage`, a counter per `Applicability`
variant, built by a match with no wildcard arm — the same shape `Applicability` itself is
built with, so a twelfth variant fails the build at this match rather than landing in
whichever bucket happened to be there. `Coverage::Nonzero` is what `Counts` renders: one line
per bucket the run actually put something in, labelled by `Applicability::Label`.

**Grouping is by variant, not by capability or by language.** This binary registers one
capability (`nomos_cap_syntax`) and one language (`nomos_lang_rust`); grouping by either
collapses to a single bucket today and would hide exactly the distinction this record is
about. `Finding::rule` is already a capability's proxy, so a second grouping remains
composable when a second capability or language exists to make it informative — nothing here
forecloses it, and adding one is not this record's subject.

**`Examined` is not extended with an integer.** Adding a third `usize` — a bare debt count —
is the shape this item was opened to stop: one number cannot tell a reader which of eleven
reasons a subject was not judged for, only that some number of them weren't. `Coverage` is a
sibling type, not a field, because the two answer different questions and forcing one into
the other's shape is how a report ends up naive about the general problem while having
already solved a special case of it (`Examined`'s own history).

**The roll-up verdict is `Claim`, read off `Applicability`'s own predicates.**
`Claim::Complete` when no finding's applicability answers `Is_Coverage_Debt` or
`Requires_Agent`; `Claim::Incomplete` otherwise. It is computed by asking `Applicability` the
question directly (`finding.applicability.Is_Coverage_Debt() || …Requires_Agent()`), not by
re-deriving the same classification from `Coverage`'s counters — one classification, asked
once, so a change to what counts as debt cannot drift between the verdict and the breakdown
that explains it. `NotApplicable` and `ConfigurationDisabled` do not flip the verdict, for the
same reason `Applicability::Is_Coverage_Debt` already excludes them: a rule that does not bind
and a human's deliberate switch are decisions, not gaps.

## Whether The Exit Code Should Stop Sharing

`done_when` asks this directly: complete-and-clean and incomplete-and-nothing-found currently
share `ExitCode::Ok`, and whether they should stop sharing it is this record's to settle.
**They keep sharing it.** Measured, not assumed — the measurement is what the decision rests
on.

`nomos check --root .` was run against this workspace both before and after this change.
Before: 768 files examined, 767 with a syntax fact, 13 findings, 0 blocking, exit 0. After
(counts moved slightly from unrelated concurrent work in the tree; the shape did not): 770
files examined, 769 with a syntax fact, 13 findings, 0 blocking, exit 0, and now additionally:

```
claim: incomplete
  Supported: 12
  DependencyUnavailable: 1
```

Twelve of the thirteen findings are `Applicability::Supported` — `Admitted_Gap`, a real
judgment that a universe declares no mirror, not debt. One is
`Applicability::DependencyUnavailable`, for `tests/corpus/analysis/gamma/broken.rs`, a fixture
this workspace keeps deliberately unparseable (`OD-COMPLETENESS-002` measured the same file
under the same rule). That file is permanent, not incidental: `check.rs`'s own module
documentation already describes it as printing "on every run".

A run over this actual, real workspace is therefore incomplete-and-nothing-found *today*, on
every invocation, including the one `OD-GATE-004` wired into `.github/workflows/gate.yml` as
the `Rules` step. `check.rs`'s own documentation states the policy this decision has to
respect: "Zero is the only success... Actions fails a step on any non-zero exit... that
default *is* the policy." A new exit code firing whenever `Claim::Incomplete` holds would
therefore turn this repository's own gate permanently red, over a gap the workspace has
already, explicitly chosen to admit rather than close — the same asymmetry
`OD-COMPLETENESS-001` names about `UNMIRRORED_TOTAL`: a gate that can never be green is a gate
everybody learns to bypass, and a gate that is red from the moment it is wired teaches the
same lesson faster.

`ExitCode::Vacuous` was considered and does not fit: it means the answer is empty because
*nothing* was judged, and a run that judged twelve subjects and could not judge one is not
that claim — it is a strictly more informative one, which is exactly why it needs a state of
its own inside the successful exit rather than a code that already means something narrower.

So the decision is: **`ExitCode::Ok` continues to mean "nothing found can fail a build",
unchanged, and does not additionally promise "and nothing was left unjudged."** That second
promise is now made in the text — `claim: incomplete` and the per-variant breakdown — where a
reader who wants it can read it, and a future gate step that wants to fail a build over
`Claim::Incomplete` specifically can be built by parsing that line, deliberately, rather than
by this record silently repurposing an exit code the gate already treats as a hard boundary.
Nothing about this decision closes off adding a stricter gate later; it declines to make that
choice by side effect of a report change that was never asked to touch the gate.

## The Negative Control

`done_when` asks for the test that matters most: a subject the run could not judge must not
render the same as a subject that was judged clean.

Two versions exist, both passing. `check::report::tests::Test_A_Coverage_Debt_Subject_Must_Not_Render_The_Same_As_A_Clean_Run`
constructs a fabricated `Finding` carrying `Applicability::DependencyUnavailable` and asserts
its rendered `Report` output differs from an empty, clean run's — `claim: incomplete` against
`claim: complete`, and the variant's label present on one side and absent from the other.
`check::tests::Test_A_Provider_Refusal_Must_Not_Render_The_Same_As_A_Clean_Run` is the
end-to-end version: a real tree with one clean file and one file the real, registered parser
refuses, run through `Run` exactly as the shipped binary runs it, beside a tree with the clean
file alone. Both trees exit `ExitCode::Ok` — the assertion `code == clean_code` is in the test
on purpose, naming the fact this record's exit-code decision rests on — and the rendered text
still differs, which is the only place left that it can.

## What This Does Not Do

- **It does not close the one admitted debt finding.** `broken.rs` stays unparseable by
  design; this record is about that fact being visible, not about removing it.
- **It does not add a rule.** `nomos-rules` is unchanged. Every `Applicability` value it was
  already capable of producing was already reachable before this record; what changed is
  whether the report downstream could say which one arrived.
- **It does not change what makes a build fail.** `Finding::Can_Fail_A_Build` and
  `ExitCode::Violations` are untouched. A debt finding still cannot fail a build, for the
  reason `Can_Fail_A_Build`'s own doc comment gives — a rule that could not read its subject
  has reported on the analysis, not on the code.
- **It does not give the gate a stricter policy.** See above. That is a live, separate
  decision this record declines to make by accident.

## Status

Accepted, closed by `P12-COVERAGE-REPORT`. The report reads the enum: every subject that
received no judgment appears by its `Applicability` variant, in a rendering distinguishable
from a clean run by a test built for exactly that comparison, and the exit-code question the
item asked to have settled is settled — not split, and measured against this workspace's own
permanently-admitted case rather than assumed.
