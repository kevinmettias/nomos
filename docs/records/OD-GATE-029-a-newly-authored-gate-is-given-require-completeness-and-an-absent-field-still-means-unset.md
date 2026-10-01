---
id: OD-GATE-029
type: decision
title: A newly authored Gate is given require-completeness, and an absent field still means unset
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - coverage
  - configuration
  - compatibility
relations:
  - target: OD-GATE-016
    type: relates-to
  - target: OD-COMPLETENESS-004
    type: relates-to
  - target: OD-GATE-015
    type: relates-to
---

# A newly authored Gate is given require-completeness, and an absent field still means unset

## Question

`OD-GATE-016` built `CoveragePolicy` as two variants and made `Unset` the default, so a gate
run whose selected findings carry coverage debt reports `Passed`. That was the right call for
the question that record asked, which was whether the policy should exist at all: every
construction site predating the type had to keep behaving as it had, and
`OD-COMPLETENESS-004` had already settled that coverage debt is reported rather than gated on.

The question left over is different, and `OD-GATE-016`'s own acceptance names it as a new
question for a new record: what a Gate that nobody has configured yet *should* be. Today a
gate whose rules could not be evaluated at all reports `Passed`, and it does so because of a
field nobody wrote. That is a defensible answer for a configuration written before the field
existed. It is not a defensible answer for one authored after it.

## What Was Measured

Checked 2026-09-14 against this workspace's own gate-orchestration sources, its CLI parsing,
and the absence of a policy file at its own root.

**The type, and what its default means.** `CoveragePolicy`
(`crates/orchestration/nomos-gate-orchestration/src/policy/coverage_policy.rs`) has exactly two
variants, `Unset` carrying `#[default]` and `RequireCompleteness`.
`Test_Default_Should_Be_Unset` pins the default. `Reduced_With_Coverage`
(`gate_environment.rs`) reads it once, after `Disposition_Of_Findings` has already reduced the
blocking findings, and downgrades `Passed` to `Indeterminate` only under
`RequireCompleteness`.

**Nothing authors a non-default one.** `nomos-cli`'s own gate parsing constructs
`CoveragePolicy::default()` and says so in a comment: no flag authors a non-default policy.
Grepped across `crates/`, the only writer of a `nomos-gate.json` anywhere in this workspace is
`nomos-gate-orchestration`'s own test module, which writes fixtures into scratch trees. There
is no scaffolding verb, no API or MCP operation, and no template that produces one. This
repository's own root carries no `nomos-gate.json` either, so CI's `gate run --root .` takes
every default.

**`Unset` is carrying two jobs, and that is the measurable cost.** `Resolved_Over` on
`GatePolicyFile` merges a caller's policies over a declared file with one rule: a field left
at its default in the command takes the file's value. For coverage that rule is spelled
`if command.coverage == CoveragePolicy::default() { self.coverage } else { command.coverage }`.
So `Unset` is simultaneously a *meaning* — coverage debt does not affect disposition — and a
*sentinel* — the caller stated nothing, defer to the file. `DeclaredCoverage` in
`gate_policy_file.rs` has the same two variants with the same `#[default] Unset`, so a file
that writes `"coverage": "unset"` and a file that omits the key are also indistinguishable.

Three things follow directly, and none of them is a matter of taste:

- A caller cannot state partial operation and have it respected. Stating `Unset` over a file
  declaring `RequireCompleteness` loses, because stating `Unset` is how a caller says nothing.
- A file cannot state partial operation distinguishably from silence.
- No inspection surface can report which of the two a run is in, because the run does not
  hold the difference.

**The blast radius of redefining `Unset` instead.** 49 `GateCommand` constructions across 18
files, of which exactly four state `RequireCompleteness` and all four are tests. Every other
one inherits `Unset` through `Default`. Redefining `Unset` to require completeness would change
the disposition of every one of them without a line of any of them being edited, and would
also change what the sentinel above means at the same time.

## The Decision

**The representation default and the product-authoring default are two questions, and only the
second changes. `CoveragePolicy` gains a third variant, `AllowPartial`. `Unset` keeps exactly
the meaning it has. Anything that authors a new product Gate emits require-completeness.**

### Unset keeps its present meaning, and no existing caller is migrated

An absent `coverage` field in an existing configuration still resolves to `Unset`, and `Unset`
still means coverage debt does not affect disposition. The 49 construction sites above are
untouched, CI's own `gate run --root .` is untouched, and a historical run stays reproducible
against the configuration it was run under.

Redefining `Unset` globally would be a silent semantic migration wearing a default's clothes.
Nothing would change in any caller's source, the field would still be absent everywhere it is
absent today, and the disposition of every run in this workspace and every repository holding
an existing `nomos-gate.json` would change underneath them. A compatibility decision made that
way cannot be reviewed at any of the sites it affects, because none of them says anything.

### AllowPartial exists so a deliberate choice is inspectable rather than inferred

`AllowPartial` means the same thing to the run that `Unset` means today: coverage debt does not
affect disposition. What it adds is that somebody said so. It separates the meaning from the
sentinel, which is what makes all three consequences measured above answerable: a caller can
state partial operation and have it win over a declared file, a file can state it
distinguishably from omission, and an inspection surface can report which one a run is in.

That is the principle this record actually rests on. **Partial coverage may pass should be a
decision somebody made, not a consequence of a field nobody wrote.** The two-variant model can
express the consequence and cannot express the decision.

### A newly authored Gate is given require-completeness

Anything that authors a new product Gate — a verb that scaffolds a `nomos-gate.json`, a Gate
created through `nomos-api` or MCP, a template — emits `require-completeness` explicitly, as a
written field. None of those surfaces exists yet, which is why this record can decide their
default before any of them has one to preserve: the cheapest moment to choose an authoring
default is before there is a caller whose behaviour it would change.

A Gate authored that way is strict by default and may be relaxed by editing a field that is
already there and already visible. That is the opposite of today's arrangement, where it is
lenient by default and can be made strict only by knowing a field exists.

### Unset becomes visibly non-ideal, and is deliberately not made invalid

Wherever a policy is authored or inspected, `Unset` is named as a compatibility default rather
than shown as one choice among equals: named as such, with its effective behaviour stated, and
with `allow-partial` or `require-completeness` recommended in its place depending on what the
repository actually intends. A reader who meets `Unset` should learn that they have inherited a
decision rather than made one.

It is not made invalid, and not deprecated to the point of refusal. Doing that would force a
migration across every existing caller and every existing `nomos-gate.json` in order to improve
a default for future use, which is a cost paid by people who are not the beneficiaries. The
same reasoning `OD-GATE-016` used to make `Unset` the default in the first place applies to
leaving it valid now; what changes is that it stops being what a new Gate silently inherits.

## What This Record Does Not Do

- **It changes no code.** The third variant, the authoring surfaces that emit
  `require-completeness`, and the inspection wording that names `Unset` as a compatibility
  default are separate implementation work this record unblocks. Nothing here is built by
  authoring it.
- **It does not decide what `AllowPartial` and `Unset` render as** in any particular report,
  beyond requiring that the two be distinguishable and that `Unset` read as inherited. The
  wording belongs to the item that builds the surface.
- **It does not widen `CoveragePolicy` past the three variants.** A
  minimum-`Applicability` threshold or a per-rule coverage requirement is still what
  `OD-GATE-016` left to real evidence, and this record does not supply that evidence.
- **It does not change `nomos check`'s own exit code.** `OD-COMPLETENESS-004` settled that
  surface, and this record is about the Gate layer alone, the same division `OD-GATE-016` drew.

## Status

Accepted. `CoveragePolicy` becomes a three-variant model; `Unset` keeps its meaning and its
sentinel role and migrates nobody; `AllowPartial` makes partial operation a stated choice
rather than an inferred one; a newly authored Gate is given `require-completeness`; and `Unset`
is made visibly non-ideal in authoring and inspection output without being made invalid.
