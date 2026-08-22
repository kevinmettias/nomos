---
id: OD-PACKAGE-012
type: decision
title: Model and agent backend routing integrates through the existing capability system, not a privileged layer, when it is built
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - package
  - model
  - agent
  - capability
  - roadmap
relations:
  - target: OD-PACKAGE-010
    type: relates-to
  - target: OD-PACKAGE-011
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
---

# Model and agent backend routing integrates through the existing capability system, not a privileged layer, when it is built

## Question

The user handed over four external documents to check against this track's work for
anything not already rejected, superseded, or incoherent with this workspace's design:
`nomos arch review.txt`, `nomos plan.txt`, `nomos packages.txt`, and
`nomos_spec_and_work_ledger_arch.txt`. Three of the four, checked directly rather than
assumed irrelevant, name nothing in `ModelBackendPackage`/`AgentExecutorPackage`'s
territory at all: `nomos plan.txt` is a rule-registry-wiring survey, `nomos packages.txt`
is a study of `nomos-proto`'s (the Go predecessor's) package architecture, and
`nomos_spec_and_work_ledger_arch.txt` is a specification-format and work-ledger-format
brainstorm. Grepped for `model backend`, `agent executor`, `model rout*`, `ModelSelection`,
and `MODEL-ROUTE` across all four: zero matches outside `nomos arch review.txt`.

That document's section 18, "Model backends should similarly be providers, not privileged
architecture," names a design principle this workspace's governing records do not
currently state anywhere. This record checks whether it is already covered, already
rejected, or in conflict with anything decided -- and finds none of the three, which is
what licenses recording it.

## What Was Measured

The claim: "A model is another way of satisfying a capability... the architecture should
avoid a giant parallel 'AI layer'... `ModelBackend`/`AgentExecutor` [should] integrate
through existing capability/evidence/work interfaces," rather than the eventual routing
system inventing its own resolution mechanism from nothing.

Checked against `ARC-ROADMAP-001` and `ARC-ECOSYSTEM-001` directly (grep, not memory):
neither states this. `ARC-ROADMAP-001` lists "language/provider/capability system" and
"model backend + agent executor infrastructure" as two separate near-term-tier bullets,
without saying whether the second binds to the first.

Checked against `OD-PACKAGE-010` directly: it states only the negative fact that "a model
backend does not register a `nomos_capability` provider the way `nomos_lang_rust` does" --
the reason `nomos-package`'s `providers` field (`PKG-007`'s fourth domain, shaped for a
language's tool registrations) was not reused for `ModelSelection`. That is a statement
about what `nomos-model-package`'s *manifest* does not do today, not a decision about
whether the eventual routing/execution system (`MODEL-ROUTE-038` through `049`) should
bind to `nomos_capability` once one exists. The two questions are different: a
`ModelBackendPackage` manifest is a distribution/versioning artifact, parallel to how a
`LanguagePackage` manifest is one -- neither wires directly into `nomos_capability::
Registry` by itself; a language's actual capability registration happens separately, in
composition (`registry.Offer(nomos_lang_rust::Provider_Offer())`). So `OD-PACKAGE-010`'s
statement is consistent with the principle, not a decision against it, and leaves the
routing-layer question genuinely open.

Checked against `OD-PACKAGE-011`, landed earlier this session: it confirms no real
implementation of `MODEL-ROUTE-038` through `049` exists anywhere in this workspace to
check any shape against -- including this one. So the principle is not yet actionable, and
this record does not make it so.

Checked against `OD-CAPABILITY-002`: this workspace already applied the identical
principle once, for language providers -- "a capability contract is not a provider's
property," resolved through `nomos_capability`'s registry rather than a provider claiming
its own terms, with the criterion for when a shared concept earns dedicated machinery
being contention between real parties, not anticipation of one. The external review's
point 18 is the same shape applied to a different pair of parties (a model/agent backend
and the thing that would route to it) that this workspace has not yet reached.

## The Finding

**The principle is real, not already stated, not contradicted, and not yet actionable.**
Naming it now, precisely, means the session that eventually builds `MODEL-ROUTE-038`
through `049`'s first real increment does not have to independently rediscover it or
build a routing mechanism that later has to be reconciled with `nomos_capability`'s
existing provider/evidence split after the fact.

**When `MODEL-ROUTE-038` through `049`'s first real increment is eventually built** --
the same trigger `OD-PACKAGE-011` already names, unchanged here -- it should be checked
against `nomos_capability::Registry`/`ProviderOffer` before inventing a separate
resolution mechanism: does a model or agent executor's selection genuinely fit the
existing capability/provider/evidence shape (a capability demand, a set of provider
offers, a resolution bounded by a ceiling), or does it need its own? `OD-CAPABILITY-002`'s
criterion applies unchanged: shared machinery earns its place when a second real party
contends for it, not by resemblance alone. This record does not answer that question in
advance of the real body that would decide it -- it names the question so it is asked
rather than skipped.

## What This Does Not Do

It does not touch `crates/packages/nomos-model-package` or any other code. It does not
build any part of `MODEL-ROUTE-038` through `049`, resolve `MODEL-ROUTE-037`'s
catalog-entry shape, or decide that model/agent routing *will* use `nomos_capability` --
only that the question must be asked against that system before an alternative is
invented, when a real case to check it against finally exists. It does not reopen
`OD-PACKAGE-010`, which stands exactly as it left the boundary, and does not restate or
amend `OD-PACKAGE-011`'s finding that no such case exists yet. It does not act on
`nomos plan.txt`, `nomos packages.txt`, or `nomos_spec_and_work_ledger_arch.txt` -- checked
and confirmed to name nothing in this track's territory, so a future session does not have
to re-check them for this purpose.

## Status

Accepted. Names a design constraint surfaced by external review, checked against this
workspace's own records and found neither duplicated nor contradicted, for whichever
future increment `OD-PACKAGE-011` already licenses. Schedules no work of its own.
