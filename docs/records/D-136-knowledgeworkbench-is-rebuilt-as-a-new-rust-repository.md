---
id: D-136
type: decision
title: KnowledgeWorkbench is rebuilt as a new Rust repository, and the existing .NET repository becomes its prototype
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - ecosystem
  - kwb
  - bootstrap
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: D-135
    type: relates-to
---

# KnowledgeWorkbench is rebuilt as a new Rust repository, and the existing .NET repository becomes its prototype

## Decision

KnowledgeWorkbench (KWB) is rebuilt from scratch as a new Rust repository, sibling to this
one and to XVPE. The existing .NET repository is not ported. It becomes the new
repository's prototype corpus, in the same relationship this repository already has to
`code-standards`: read for its domain model, its documented data-loss incidents, and its
own accepted decisions, and answered requirement by requirement — met, diverges, deferred,
or declined — as the new repository is built, rather than carried over by default.

## Rationale

KWB's own governing decision, `docs/adr/0001-implementation-language.md` in the .NET
repository, rejected Rust "for now": most of its code was judged to be EF Core and
expression-tree schema mapping, migrations, and dependency injection that does not survive
translation, with the choice explicitly named revisitable "if the domain's invariant
density grows faster than its plumbing." That decision was made without the context this
repository now supplies — a working records-driven governance skeleton, a territory-based
work ledger, a content-addressed store, and a fact-oriented shared-analysis substrate,
proven across a domain just as provenance-heavy and invariant-dense as KWB's own epistemic
kernel already is. Building KWB in Rust from the outset, on Nomos's bootstrap pattern and
toward XVPE's platform substrate, fits the ecosystem shape `ARC-ECOSYSTEM-001` already
adopted better than continuing an implementation that cannot consume either.

This repository's own relationship to `code-standards` is the direct precedent for how to
do this without carrying the old codebase's structure over wholesale: `code-standards` was
never ported, only read and reconciled for content loss, with generic scaffolding rebuilt
from first principles once its shape was understood. The same applies here. The .NET
repository's Postgres/pgvector storage, EF Core migrations, and C# domain classes are not
translated. Its domain model is read: the universal epistemic kernel that domain-specific
packs specialize, the content-derived claim and concept identity that is the actual dedup
mechanism across sources, the `CoverageOutcome` and derivation-ledger primitives that
distinguish "ran and found nothing" from "never ran," and the five recorded data-loss
incidents (`D17`-`D21` in the .NET repository's own `AGENTS.md`) that name the specific
failure classes the new repository's types must make unrepresentable rather than merely
avoided by convention.

## Consequences

The new KWB repository is not built inside this one; it is a new sibling repository under
its own governance, and this record does not create it — it fixes the relationship between
the two. From this point, the .NET repository is read-only prototype material: its own
`docs/adr/0001-implementation-language.md` is superseded for the new repository's own
records to state formally when that repository exists, and a future claim that KWB "is a
.NET product" cites a repository this decision has already deprecated.

Per `D-135`, subsystems in the new KWB repository that carry no knowledge-domain semantics
are proposed in XVPE rather than reimplemented a third time. Subsystems that are KWB's own
reasoning surface — claims, concepts, argumentation, evidence, the epistemic kernel — remain
KWB's, exactly as `ARC-ECOSYSTEM-001` already states.

## Alternatives Considered

Continuing the .NET implementation and integrating it with Nomos and XVPE through
inter-process contracts was rejected: it would leave KWB permanently unable to consume
XVPE's Rust platform substrate or Nomos's store and ledger patterns except across a language
boundary, for a domain whose own governing decision already named increasing invariant
density as the trigger for revisiting the language choice.

Porting the .NET codebase mechanically — translating classes rather than redesigning around
the domain model — was rejected for the same reason `code-standards` was read rather than
ported: a mechanical translation carries an old shape's assumptions into a language whose
type system can make the old failures structurally impossible only if the design starts
from the invariants the old system learned by losing data, not from the old classes that
allowed the loss.
