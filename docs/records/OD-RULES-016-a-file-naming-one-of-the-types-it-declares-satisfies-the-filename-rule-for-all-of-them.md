---
id: OD-RULES-016
type: decision
title: A file naming one of the types it declares satisfies the filename rule for all of them
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - naming
relations:
  - target: OD-RULES-015
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
---

# A file naming one of the types it declares satisfies the filename rule for all of them

## Question

`OD-RULES-015` decided which modules `file-name-matches-declared-type` may judge. This record
decides what it may say about the ones it does.

A module holding a type and the small vocabulary that type is defined in terms of has one
name and several public types, so at most one of them can match the file stem. The rule
reported the others. Satisfying it would mean giving each companion its own file, and after
eleven of those in one directory the shape of what was being asked became clear: it is
`one-public-type-per-file`, arrived at sideways.

## What Was Measured

**Eleven of the twenty-four files in `P27-SELFCHECK-FILENAME-DRIFT-PACKAGES` are this shape,
and its census called all twenty-four ordinary drift.** Ten really were and have been renamed.
The other eleven each declare a public type whose snake case *is* the stem, plus one or two
companions:

- `budget_estimate.rs` declares `BudgetEstimate` and `CheckOrFixStage`, which is one of
  `BudgetEstimate`'s own fields. Its module doc reads "A previewed budget for one rule at one
  check-or-fix stage" — the companion is named in the sentence that names the module.
- `shadowed_profile.rs` declares `ShadowedProfile` and `ShadowClassification`;
  `validation_diagnostic.rs`, `ValidationDiagnostic` and `CandidateOutcome`;
  `telemetry_junction.rs`, `TelemetryJunction` and `PipelineStage`. Eight more of the same.

**`one-public-type-per-file` exists, sits in the same source file, and is composed into
nothing.** It is registered in `nomos-rules` and no composition root calls it, so it has never
raised a finding on this workspace. That is a standing decision not to require one public type
per file. A rule that forces the same outcome by a different route is that decision being
reversed without anybody making it.

**The rule id names the file, in the singular.** `file-name-matches-declared-type`: the file
name matches *a* declared type. The reading that reports companions is "every declared type
matches the file name", which is a different claim and would deserve a different name. The
rule's own module doc opens "Public type declarations should live in the file named for that
type", which is satisfied by a file named for the type it is about.

## The Decision

**A module satisfies `file-name-matches-declared-type` when the snake case of any public type
it declares equals the file stem.** No finding is raised on that module.

**A module whose stem matches none of its public types is reported for every one of them**,
which is unchanged and is what caught the ten real renames. Reporting all of them rather than
guessing is deliberate: when the stem names nothing in the file, nothing in the file says
which type the file was meant to be about, and a rule that picked one would be inventing the
answer it is supposed to be checking.

Together with `OD-RULES-015` this leaves the rule saying one thing, and only one: *a module of
types is named for one of the types it declares.* Which one, and how many others share the
file, are questions this rule does not answer and `one-public-type-per-file` does — if a
composition root ever asks it.

## What This Record Does Not Do

It does not compose `one-public-type-per-file` or argue against composing it. That rule asks a
real question and this record's whole point is that it is a *separate* question; if this
workspace later wants one public type per file, the honest way to get it is to compose the
rule that says so and take the findings.

It does not bless any particular companion. `CheckOrFixStage` living beside `BudgetEstimate`
is a judgment about that module which this record does not review; it only declines to make
that judgment on a filename rule's authority.

It does not weaken the ten renames it sits beside. Those files named no type they declared,
and still report.

## Status

Accepted. The rule reads it, two tests hold the two sides, the eleven companion findings are
gone, and every file whose stem names none of its types still reports.

Revisit if a file accumulates a matching type as cover — a module named for one small type
while a dozen unrelated ones shelter behind it. That would be the rule's exemption being used
to avoid a decomposition rather than to describe one, and the answer then is to compose
`one-public-type-per-file`, which is the rule that was always about how many.
