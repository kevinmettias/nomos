---
id: OD-HOST-009
type: decision
title: A repository declares which tool answers a family, and may decline one
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - host
  - capability
  - architecture
relations:
  - target: OD-HOST-004
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-CAPABILITY-013
    type: relates-to
---

# A repository declares which tool answers a family, and may decline one

## Question

`OD-HOST-004` decided `Registry::Resolve` never needs a selection mechanism, because it
already ranks offers against whatever `Requirement` a caller states — a question about this
engine's own composition root, settled and not reopened here. A different question that
record never asked: whether the repository *being judged* may express a preference or an
exclusion among the tools this engine could run against it. Today it may not. `code-
standards`' `kernel/toolspec` names the cost of that absence directly, in its own package
doc: "the engine's answer to an absent tool is correct... but it is the engine's answer, not
the repository's decision, and the two look identical in a report." This workspace has that
defect now, and a person required a decision on whether and how to close it.

## What Was Measured

**`OD-HOST-004`'s own scope is confirmed orthogonal, not merely asserted so.** Its own text
is explicit: `Registered`'s question is whether the *registry* needs a selection mechanism
among offers a *caller* (the composition root) already names by `Requirement`. This record's
question is whether the *repository under analysis* — not the composition root, not a
caller — gets any say at all in which tool runs against it. `Registry::Resolve` ranking
offers by guarantee has nothing to say about a repository declaring "use clippy, not
`cargo check`" or "this repository has no dotnet, do not try": neither is a `Requirement` a
caller states today, and neither changes what the registry does with the offers it is
handed. The two questions do not touch.

**The mechanism this decision would reach for already exists and is proven, not
theoretical.** `OD-RULES-011` established that a repository's own configuration reaches a
rule as a capability fact rather than as ambient state a rule reads directly, and five real
capabilities already do exactly this: `nomos.cap.naming.policy`, `nomos.cap.limits.policy`,
`nomos.cap.scripting.policy`, `nomos.cap.words.policy` and `nomos.cap.goals.policy` are all
read from `standards.json` by `nomos-repo-policy`'s five domains, materialized the identical
way every other capability is, and read by a rule through the identical `FactReader`
interface. A sixth capability naming which tool a repository selects per family follows a
path this workspace has built five times already, not a new one.

**The three cases the question asks to be told apart already have distinct vocabulary, and
one of the three has never been produced by anything.** `nomos_contracts::Applicability`
already carries `MissingCapability` ("no installed provider offers a capability the rule
requires") and `ProviderUnavailable` ("a provider that would satisfy the requirement is
installed but could not run") for a tool genuinely absent or broken, and
`ConfigurationDisabled` ("policy switched the rule off for this subject — a deliberate
human choice, not a capability gap") for exactly the third case, a repository's own
decision. Checked directly: `ConfigurationDisabled` is rendered distinctly by `nomos-cli`'s
own report (`check/report.rs`) and accounted for separately by `Coverage`, but no rule in
`nomos-rules` produces it — a real word in the vocabulary with no real producer, not a gap
in the vocabulary itself. A tool that ran against real subjects and found nothing stays
`Applicability::Supported`, correctly identical to a genuinely clean tree, because it is
one.

## The Decision

**A repository may declare, per language and per family (`OD-CAPABILITY-013`'s own twelve
names), which tool answers it, or that none should run.** ~~The declaration lives in
`standards.json`, the same file the five existing policy families already read, under a new
section keyed by language and family word — the identical shape `code-standards`'
`tools.json` already proved, adapted to this workspace's one-file convention rather than a
second configuration file.~~ **Superseded at version 2: that location cannot be built, and
the declaration lives in a dedicated file at the repository root. See *The Location Was
Unimplementable* below.** The shape — keyed by language and family word — is unchanged; only
the file it is written in is.

**It travels as a capability fact, the same way the five existing policy families do — a
sixth `nomos.cap.tool.selection`-shaped capability, materialized from `standards.json`
exactly like `nomos.cap.naming.policy` is.** No new mechanism is invented: `OD-RULES-011`'s
own path is followed a sixth time, not extended or special-cased.

**A provider consults the selection before it runs, not after.** The natural point is
`nomos-check-orchestration`'s own per-capability materialization step — the same place that
already decides whether a family's provider is invoked at all for this request
(`Is_Rule_Selected`) — gaining a second, independent gate: whether the repository's own
selection permits this family to run for this language at all. A family the repository
declined never reaches its provider's subprocess; a finding carrying
`Applicability::ConfigurationDisabled` is filed in its place, `ConfigurationDisabled`'s first
real producer. A family the repository did not mention, or explicitly left to the engine,
resolves exactly as it does today.

**The three cases resolve on existing vocabulary, not new vocabulary.** A tool absent from
the machine or broken stays `MissingCapability`/`ProviderUnavailable`, unchanged. A tool the
repository declined is `ConfigurationDisabled`, now real. A tool that ran and found nothing
stays `Supported` with no findings — indistinguishable from a clean tree because that is
what it is, which is correct and is not this record's problem to solve.

## What This Record Does Not Do

**No selection mechanism is built here.** The sixth capability contract, its payload shape,
`standards.json`'s new section, and the materialization-time gate are named precisely enough
for a follow-up item's territory — a new `nomos-cap-tool-selection` crate beside the five
existing policy contracts, and one gate per subprocess-backed provider's own materialization
function in `nomos-check-orchestration` — rather than built speculatively here.

It does not reopen `OD-HOST-004`. `Registered`'s own criterion — whether a rule's or
provider's participation is unconditional or meant to vary by request — is untouched;
naming a preference at the repository level is a fact a materialization step reads, not a
change to how the registry resolves offers among the providers this engine ships.

It does not decide `OD-CAPABILITY-013`'s own open half either. That record classified
today's eight providers by family; this one uses that vocabulary as the key a repository's
selection is written against, without revisiting the classification itself.

It does not give a repository the power to select a tool this engine has not shipped a
provider for. A selection names which of the *admitted* offers for a family should run, or
that none should — it cannot conjure a ninth provider, the same way `Registry::Resolve`
cannot today.

## The Location Was Unimplementable

Added at version 2. The decision above stands in every part except where the declaration is
written, which was named without measuring the one thing that decides it: `standards.json` is
not this workspace's file.

**Another tool reads it, and reads it strictly.** Measured 2026-09-14 against the live
`code-standards` checkout. `decode_Limits` in `kernel/config/limits/config_loading.go` calls
`decoder.DisallowUnknownFields()` at line 171 and decodes the whole normalized file into one
`Limits` struct, and its own comment says why: *"DisallowUnknownFields turns a typo'd key into
a loud error rather than a no-op."* This repository's `standards.json` carries thirteen
top-level keys — `conformance_workers`, `data_format_contracts`, `dependency_budget`,
`json_key_naming`, `languages`, `naming`, `projects`, `scripting`, `standard_flags`,
`suppression`, `telemetry`, `tiers`, `words` — and every one is a field that struct names. A
fourteenth key this repository owned outright would not be an addition to a shared file. It
would make another tool fail to read a file it has read all along, loudly, by that tool's own
design.

**The five families are not a counterexample, and they are why the constraint was invisible.**
Each reads a key `code-standards` already owns and already decodes — `naming`, `scripting`,
`words`, and the rest. Nomos piggybacks on that tool's schema rather than extending it, so no
policy family has ever needed a key of its own, and "the same file the five existing policy
families already read" was true of the file and false of the act.

**So the declaration lives in a dedicated, language-neutral file at the repository root.**
That is not a new convention invented to escape the problem; this workspace has it twice
already. `nomos-gate.json` carries a repository's declared gate policy, and
`nomos-architecture.json` carries the architecture declaration `OD-RULES-024` records as
built, read by `nomos-repo-policy`'s `architecture` module at the repository root. A tool
selection is the same kind of thing: a declaration this repository owns outright, about
itself, that no other tool parses.

**Language-neutral matters for this record in particular.** The declaration is keyed *by
language*, and a carrier only one ecosystem can express — a Cargo `workspace.metadata` table,
say — would make the whole family Rust-only, which contradicts the thing being declared.

**What this does not change.** Whether a repository may declare a tool preference or an
exclusion at all, which is what this record is for and is untouched. That the declaration
travels as a capability fact the way `OD-RULES-011`'s five families do — a provider reads a
file and materializes a fact, and which file it reads was never the part that made that
pattern work. That the gate is consulted before a provider's subprocess runs, and that the
three cases resolve on existing `Applicability` vocabulary. And that **no mechanism is built
here**: this corrects a location, and the follow-up item's territory named in *What This
Record Does Not Do* is unchanged except that its new file is a repository-root declaration
rather than a `standards.json` section.

Separately, and not this record's to fix: the same measurement is why a sibling item's
architecture declaration went to `nomos-architecture.json` rather than to a `standards.json`
section. The constraint is a property of the shared file, not of either decision.

## Status

Accepted, at version 2. A repository may declare a tool preference or exclusion per language
and family, written into a dedicated declaration file at the repository root — **not**
`standards.json`, which another tool decodes with unknown fields disallowed — and read as a
capability fact the same way five existing policy families already are, checked before a
provider's own subprocess runs. The three
cases a report must tell apart already have the vocabulary: `MissingCapability`/
`ProviderUnavailable` for absent, `ConfigurationDisabled` for declined — its first real
producer — and `Supported` for ran and clean. `OD-HOST-004`'s own composition-root question
is untouched. No mechanism is built here.
