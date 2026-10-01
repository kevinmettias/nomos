---
id: OD-CAPABILITY-005
type: decision
title: A required naming refuses rather than substitutes, because the caller said so
status: closed
version: 1
authority: canonical-normative-record
tags:
  - capability
  - resolution
  - provenance
relations:
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-CAPABILITY-003
    type: relates-to
  - target: OD-CONTRACTS-002
    type: relates-to
---

# A required naming refuses rather than substitutes, because the caller said so

## Question

`Requirement` carries `preferred: Option<ProviderId>` and one builder that sets it,
`Preferring`. There is one strength of naming, and `registry.rs`'s own comment gives the
reason the gap is not obvious: when the preference is not honoured, `Registry::Resolve`
still answers, through `Resolution::Satisfied` and `Applicability::SupportedWithFallback`.
The judgment still stands and its provenance is not what was asked for — refusing outright
would deny a caller who would have accepted the fallback, which is the mistake
`OD-CAPABILITY-001` avoided when it declined to let the registry decide how far down a
caller spends.

That reasoning is sound for a caller that named a provider because it would rather have it.
It is not sound for a caller that named a provider because that provider *is* the policy —
the strongest reader for a subject that matters, the one implementation whose output is
admissible downstream. That caller is served a different provider, told so in a field
(`Applicability::SupportedWithFallback`), and nothing makes it look: the field satisfies
`Is_Evaluated`, so the run reports a judgment made by somebody other than the one required,
and reports it as evaluated, which is true and is not what was asked. `OD-CAPABILITY-003`
made this worse rather than better — per-subject fallback means a capability can be answered
by two providers under two addresses within one run, and a caller that required one of them
has more places to not notice, not fewer.

## What Was Measured

`registry.rs`'s `Honoured` function is the entire mechanism: if `requirement.preferred` is
set and does not match the offer that answered, the result is `Satisfied` with
`SupportedWithFallback`. There is no path from an unhonoured naming to `Unsatisfied`, and
nothing on `Requirement` or `Resolution` lets a caller ask for one. A convention — "check
`Applicability` after every `Resolve`" — is the only thing standing between a required
naming and a silently substituted answer, and nothing enforces the convention.

## The Decision

### Two functions, not one field and a convention

`Registry` gains `Resolve_Requiring(&self, requirement: &Requirement, required: &ProviderId)
-> RequiredResolution`, beside the unchanged `Resolve`. It does not add a second field to
`Requirement` — a `required: Option<ProviderId>` beside `preferred: Option<ProviderId>` would
be exactly the boolean-beside-a-field shape this item was opened to refuse: nothing stops a
call site from setting one and not the other, and a reviewer has to read the value to know
which strength a given call site meant. Two methods with two return types mean a call site's
strength is legible at the call, not at the value.

`Resolve_Requiring` forces `required` into the ranking exactly the way `Preferring` already
does — `Selection::Over` already picks a named provider over a stronger unnamed one, so
nothing new is invented there — and then checks whether the offer that was forced to the
head is actually the one that answered. If it is, the answer is
`RequiredResolution::Satisfied`, indistinguishable in shape from what `Resolve` would have
produced with `Applicability::Supported`, because an honoured requirement and an honoured
preference are the same event. If it is not — because the required provider is absent,
below the floor, or unreadable at this version — the answer is
`RequiredResolution::Unsatisfied`, naming `required` (echoed back, since the caller already
knows it) and, when somebody else was usable, `answered`: who would have answered, had this
been a preference instead.

### Why refusing is right here though `OD-CAPABILITY-001` declined to let the registry refuse on the caller's behalf

`OD-CAPABILITY-001` is about the same registry, the same fallback machinery, and it went the
other way: a lowered floor bought reachable coverage, and refusing the weaker offer instead
of serving it would have spent the caller's own widened floor against it. The difference is
what the caller said. There, the caller said "I will accept as low as this floor" and named
nobody — the registry choosing the strongest usable offer is applying the caller's own
stated tolerance, not substituting its judgment for the caller's. Here, the caller says "only
this provider", through `Resolve_Requiring` rather than `Resolve`, and an answer from anybody
else is not what was tolerated — it is what `Resolve` already offers, one call away. Refusing
under `Resolve_Requiring` does not deny a caller anything it did not already have access to;
it makes the caller's naming mean what it says at the one call site the caller chose to
invoke.

### `RequiredUnmet` reuses `Unmet`, once

`Unmet` already distinguishes "nobody declared this", "declared and nobody offers it",
"offered at a version I cannot read", and "offered, and nothing reaches the floor" — none of
those four is wrong when the required provider specifically is what is absent, unreadable, or
below floor, because in every one of those cases nobody at all could have answered, required
or not. `RequiredUnmet::Unavailable(Unmet)` reuses that vocabulary unchanged rather than
inventing a parallel four-way split. The one thing `Unmet` cannot say — that somebody usable
*did* answer and it was not who was required — is the only new variant,
`RequiredUnmet::AnsweredByOther { required, answered }`, and it exists because that is
exactly the case `Resolve` already answers and `Resolve_Requiring` must not.

## What Was Considered And Rejected

**A `required: bool` or `required: Option<ProviderId>` field beside `preferred`.** This is
what `done_when` names directly: a flag beside a field is not enforced at any call site, and
a caller — or a reviewer — has to read the value to know which strength was meant. The two
strengths need to be visible in which function was called, not in which fields happen to be
set.

**Growing `Applicability` with a third state for "required and not honoured".**
`OD-CONTRACTS-002` grew `Applicability` when a rule could not reach a judgment mechanically —
a genuinely new thing a run needed to report about a subject. This is not that: it is a
caller-side policy about how strictly to read an existing, correctly-reported
`Applicability::SupportedWithFallback`. Growing the vocabulary every non-Rust peer
reimplements to encode one caller's spending policy is the same category error
`OD-CAPABILITY-001` avoided by keeping cost off `Guarantee`.

**Reusing an existing `Unmet` variant (`NoProvider` or `BelowRequirement`) for "answered by
somebody else".** Both are false statements when somebody usable did answer: `NoProvider`
says nothing offers the capability, `BelowRequirement` says nothing reaches the guarantee.
Neither is true when the requirement was met by the wrong provider. Reusing either would
report a fabricated guarantee or coverage shortfall in place of the real reason, which is an
identity mismatch.

## What Holds It

`Test_A_Required_Naming_Should_Refuse_What_A_Preferred_Naming_Falls_Back_To` in
`crates/substrate/nomos-capability/src/registry.rs` drives the same registry and the same
unavailable provider through both `Resolve` and `Resolve_Requiring`, and asserts the first
is `Resolution::Satisfied` with `Applicability::SupportedWithFallback` while the second is
`RequiredResolution::Unsatisfied` naming both the required provider and who would have
answered — not merely a different `Applicability` read off the same `Satisfied`.

## Status

Closed by `P11-PREFERENCE-STRENGTH`. `Resolve` is unchanged; every existing caller of it
keeps today's fallback behaviour without touching a line. `Resolve_Requiring` is additive,
and no caller of it exists yet in this workspace — like `OD-CAPABILITY-004`'s knowledge
seam, the obligation is stated so the strength is right before a caller depends on it.
