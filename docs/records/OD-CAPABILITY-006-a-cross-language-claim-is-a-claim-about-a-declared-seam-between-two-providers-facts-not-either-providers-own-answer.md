---
id: OD-CAPABILITY-006
type: decision
title: A cross-language claim is a claim about a declared seam between two providers' facts, not either provider's own answer
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - cross-language
  - analysis
  - boundaries
relations:
  - target: ARC-CONFORMANCE-001
    type: relates-to
  - target: OD-ANALYSIS-004
    type: relates-to
  - target: OD-ANALYSIS-006
    type: relates-to
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
  - target: OD-CAPABILITY-004
    type: relates-to
  - target: OD-RULES-003
    type: relates-to
---

# A cross-language claim is a claim about a declared seam between two providers' facts, not either provider's own answer

## Question

`crates/languages` holds two providers, `nomos-lang-rust` and `nomos-lang-rust-scan`, and
`tests/contract/tests/boundaries/bands.rs` puts them at the same band deliberately, forbidding
an edge between them: "two providers of one capability... this suite's downward rule forbids
an edge between crates at one band, which is what stops the second answer from being derived
from the first. Two providers that shared a parser could not disagree." `P8-SECOND-PROVIDER`
is why that rule has ever been tested at all — it built the second Rust provider so
`Registry::Resolve` had more than one offer to choose between, and `OD-CAPABILITY-001` settled
which of several usable offers wins.

That is a rule about two answers to *one* question: one capability, one subject key space, one
provider's offer weighed against another's, resolved by selection or fallback. It says nothing
about two providers answering two *different* languages' questions about two different
subjects, joined into one claim about how those subjects relate to each other — an FFI call, a
message one side serializes and the other deserializes, a protocol one side speaks and the
other answers, a build pairing shipped from two separate toolchains. This repository has the
mechanism a claim like that would need — `FactVariant`, `Guarantee`, `EvidenceClass`,
`Applicability`, `Observation`, all provider-neutral by construction — and no second language
to test it with. Whether two providers' facts actually compose into a claim spanning both, or
merely sit next to each other in the same store, is unproven, and `OD-ANALYSIS-004` and
`OD-ANALYSIS-006` already did the equivalent groundwork for two other never-yet-produced
resolution levels rather than waiting for a producer to discover the shape by trial. This
record does the same for the cross-language seam, honestly, before a second language exists to
tempt an implementer into deciding it ad hoc.

## What A Cross-Language Claim Is

**It is not a same-capability disagreement.** `P8-SECOND-PROVIDER` and `OD-CAPABILITY-001`
cover the case where two offers answer the *same* question about the *same* subject — the
`bands.rs` rule exists exactly so that case stays confined to one band, decided by
`Selection::Weaker` and fallback, never by one provider deriving from the other. A
cross-language claim is not that case wearing a different label. There is no ranking between a
Rust ownership fact and a Python ownership fact the way there is between a parser's answer and
a scanner's — neither is a weaker or stronger offer of the same fact, because they are not
offers of the same fact. Nothing in `Registry::Resolve` or `Selection` applies, and nothing
here reopens that they don't.

**It is a claim about the seam, not a claim either side's tooling holds.** `ARC-CONFORMANCE-001`
already generalized the shape: a Nomos conformance claim is "composed from architecture...,
requirements..., history..., runtime evidence..., and policy... checked against each other
rather than against a language's grammar or type system," and the record's structural test is
that Nomos writes native analysis "only where the fact a claim needs is not obtainable from any
existing provider at all." A cross-language claim qualifies on exactly that test, for a reason
that is structural rather than a gap in today's tooling: a Rust language server has no model of
a Python process, a Python type checker has no model of a C library's ownership contract, and
neither omission is either tool being wrong — the claim was never theirs to make, because
composing two providers' facts about two different subjects into one claim about how those
subjects relate is a question only the party that can see both sides can even ask. That party
is Nomos, exactly as it is the only party that can check the README's band table against the
workspace or a projection's freshness against its own render history — two of `ARC-CONFORMANCE-001`'s
four worked examples, cited here because a cross-language seam is the same shape of claim over
a different pair of inputs.

## What Has To Be True Of Two Providers For Their Facts To Be Comparable At All

Two providers each filing facts under `FactVariant`, `Guarantee`, `EvidenceClass`,
`Applicability` and `Observation` is necessary and not sufficient. That shared vocabulary
states how strongly each provider's own fact is known — its resolution level, its soundness
and completeness, how it was come by, whether a judgment was reached. It states nothing about
whether the Rust fact and the Python fact are facts *about the same thing*. Filing both under
the same five-question vocabulary is what lets them be compared once there is something to
compare; it is not itself the something.

**A cross-language claim is well-formed only when a correspondence between the two subjects is
itself declared.** Not inferred by either provider — a Rust provider has no way to discover
that `alpha::Handle` is the same boundary as Python's `ctypes.c_void_p` binding in `beta.py`,
because that correspondence is not a fact about either subject's own text, syntax, or resolved
model; it is a fact about how this repository's own architecture wires the two together, an FFI
declaration, a shared IDL or schema file, a matched build pairing. `OD-RULES-003` already states
the parallel case for a different kind of edge: "a declared architecture is data, the observed
graph is a capability's fact" — a dependency edge is not derived by watching the code, it is
authored and then checked against what the code does. A cross-language correspondence is the
same kind of thing: declared, not discovered, and a cross-language capability checks the two
providers' facts against a correspondence it was handed rather than one it worked out.

Without that declared correspondence, two syntactic facts about two different-language subjects
are two unrelated answers, not evidence about a seam — proximity in the store is not
composition. `OD-CAPABILITY-004` already refused the adjacent version of this mistake when it
kept an absent optional knowledge packet out of `Applicability`'s vocabulary rather than let an
empty collection read as a verdict: a fact sitting where a claim was expected is not the claim,
and a cross-language check that composed two providers' facts merely because both existed would
be the same conflation one level up — reporting a seam checked when only two unrelated subjects
were ever looked at.

So the criterion has two parts, and a cross-language check that skips either is not making the
claim it appears to make:

1. **The correspondence is declared**, by this repository's own architecture or configuration,
   naming which subject on each side is claimed to be the same seam — the join a cross-language
   capability checks against, never one it infers, for the structural reason above: neither
   side's provider can see the other's subject to infer it from.
2. **Each side's fact meets the resolution floor the specific claim being made requires** — the
   next section states which floor each family needs, because "both sides produced *a* fact"
   is not "both sides produced the fact this claim needs."

## The Claim Families, And What They Are Reachable With

The WHY names five families. Each is classified against the two resolution levels
`OD-ANALYSIS-004` and `OD-ANALYSIS-006` already fixed the meaning of, and against `Syntactic`,
which already has producers today.

**1. FFI ownership compatibility** — which side owns, and which must free, a value crossing the
boundary. `OD-ANALYSIS-004` already places the shape this needs at `SemanticallyResolved`: "a
claim of this family's shape is that a value's ownership, once resolved, crosses a boundary
`bands.rs` declares closed" — the same shape, applied to a declared FFI boundary instead of a
band boundary, needs a provider that has actually resolved ownership, not one that pattern-matches
`unsafe` blocks or raw-pointer syntax. Confirming the foreign side actually honors what the
Rust side declares — that a pointer handed across is not also freed on the far side, at an
actual call rather than in the declared contract — is `RuntimeObserved`'s territory per
`OD-ANALYSIS-006`'s split between static and executed facts. **Not reachable today on either
level.** No `SemanticallyResolved` producer and no `RuntimeObserved` producer exists anywhere
in this workspace; this family waits on both.

**2. Producer/consumer serialization format agreement** — whether both sides read and write the
same wire shape for one message type: field names, order, declared types, optionality. This is
the one family with real reach at `Syntactic` today, and only partly. Comparing two sides'
*spelled* declarations — side A declares four fields, side B declares three, one name differs —
needs no name resolution and no execution; it is a parse-tree-level comparison of two
declarations against a declared correspondence, exactly the kind of fact `nomos-cap-syntax`
already produces on one side today. Reachability stops the moment the comparison must be about
the two sides' *resolved* types rather than their spellings — a type alias, a generic
parameter, or an import under a different local name defeats a syntactic comparison, because
two different spellings naming the same underlying type, or one spelling hiding two different
underlying types, is exactly what `SemanticallyResolved` exists to settle and `Syntactic`
cannot. **Partly reachable now** (name, arity and literal-type agreement, syntactically); **the
harder half needs `SemanticallyResolved` on both sides**, which nothing in this workspace
produces yet.

**3. Cross-protocol error propagation** — whether an error raised on one side surfaces correctly
on the other, in the shape the other side expects. The static half of this — do the two sides'
declared error vocabularies even have a corresponding case for every case on the other side, an
exhaustiveness question over two declared enums checked against a declared mapping — is
reachable the same way family 2's easy half is, once a syntactic cross-language facility exists
to read both declared vocabularies and the mapping between them. Whether an error actually
propagates and arrives in the declared shape, rather than merely being declared to, is a claim
about an execution: `OD-ANALYSIS-006` is explicit that facts "established by running the
program and watching what it actually does" are `RuntimeObserved`'s shape and no other level's.
**The mapping-completeness half is reachable once a syntactic cross-language facility exists**
(none does yet); **the propagation-actually-happens half needs `RuntimeObserved`**, which has no
producer.

**4. Cross-side threading assumptions** — an assumption that holds on one side (single-threaded
access, a GIL-serialized call) does not automatically hold on the other (a Rust thread pool
calling in from several threads at once). `OD-ANALYSIS-006` places actual concurrency
interleaving and actual synchronization order at `RuntimeObserved` exclusively, and
`OD-ANALYSIS-004` places the async/concurrency-structure family — the closest static analogue,
a declared `Send`/`Sync` bound or a documented threading contract — at `SemanticallyResolved`,
itself without a producer. Neither level has a producer in this workspace, and the family's own
real claim (that the intended contract is honored across an actual cross-language call
sequence, not merely declared) is the `RuntimeObserved` half regardless. **Not reachable at any
level today** — the least reachable of the five, because even its static half waits on a
producer that does not exist.

**5. Separately-built version compatibility** — whether two independently-built components,
compiled at different times from different commits, agree on the interface they share. This
family differs in kind from the other four: it is not a claim about program semantics at all,
but about build and history provenance. `FactKey`'s existing `BuildVariantId` and
`ConfigurationId` components — named in `OD-ANALYSIS-006` as already sufficient to identify
"which optimization level, target and feature set the executed binary carried" and "the digest
of a fully resolved effective policy" — already carry what this family needs to compare. It
needs no new `FactVariant` level and no language-semantic producer on either side, only a
producer that reads and compares the two sides' existing build and version metadata against a
declared correspondence, which sits closer to `ARC-CONFORMANCE-001`'s "history" and "runtime
evidence" inputs than to a language fact at all. **This is the one family that does not wait on
a resolution-level producer this record has to name as missing** — the gap here, if any, is a
producer for the comparison itself, not a resolution level nothing in this workspace can reach.

## What A Cross-Language Check Reports When One Side Has No Provider

The common real case is a stack where exactly one language is covered — a Rust-only workspace
naming a correspondence to a Python client nothing here parses. `done_when` requires
`Applicability`, not silence, and the specific value matters because two nearby ones read
similarly and mean different things.

**A cross-language rule binds a subject *pair*** — both halves of a declared correspondence,
per the criterion above. When one half's language has no installed provider at all, the rule
cannot evaluate the pair even partially: there is no fact on that side to compose, at any
`FactVariant`, so there is nothing to check the covered side's fact against. That is exactly
what `Applicability::MissingCapability` is defined to mean — "no installed provider offers a
capability the rule requires" — and the report names which side is missing.

**This rules out two answers that read like reasonable alternatives and are not:**

- Reporting the covered side's own fact as though the cross-language claim itself were
  satisfied. A claim that requires two facts joined at a declared seam is not the same claim as
  one side's fact alone, and reporting it as satisfied is exactly the silent-pass failure mode
  `Applicability` exists to prevent — the seam was never checked, only one provider's ordinary
  business was.
- `Applicability::NotApplicable`. That variant is "the only variant that is a *positive*
  statement about the absence of a judgment" — it says the rule does not bind this subject. A
  cross-language rule over a declared correspondence does bind the subject; the correspondence
  says so. What is missing is a provider to evaluate one half with, which is `MissingCapability`'s
  case exactly, not the rule's non-applicability.

**Contrast with `Applicability::PartiallySupported`**, which is for a rule "evaluated over part
of its subject only" when the subject is itself a set — some files in a corpus covered, others
not. A cross-language pair is a single subject with two required halves; one half missing
entirely is not partial coverage of many subjects, it is the whole join's precondition failing
before evaluation starts. `MissingCapability` is the value whose own definition matches that
failure; `PartiallySupported` would understate it, the same way `OD-CAPABILITY-004` kept a
similar pair of near-identical-looking reports apart by definition rather than by convenience.

**If a provider is installed on both sides but one could not run for this particular pair** —
crashed, timed out, refused the specific subject — that is `Applicability::ProviderUnavailable`
("a provider that would satisfy the requirement is installed but could not run"), distinct from
`MissingCapability` for the reason `Applicability`'s own documentation already gives: the remedy
differs, and a reader who is told to install something when something is already installed and
merely failed is told the wrong thing to do next.

## What This Record Does Not Do

It does not build a cross-language capability, a correspondence-declaration type, or any
producer for either resolution level named above. The first cross-language capability is
separate work, reserving its own territory, and this record is what that work is measured
against — the relationship `OD-ANALYSIS-004` and `OD-ANALYSIS-006` each already hold to the
first producer in their own domains.

It does not add a sixth `FactVariant` level, a new `Applicability` variant, or change
`Guarantee`, `EvidenceClass` or `Observation` in any way. Every type this record names is
unchanged by it, and the two reports it fixes — `MissingCapability` for a wholly missing side,
`ProviderUnavailable` for an installed side that could not run — are both variants that already
exist for exactly this reason.

It does not decide which crate a cross-language capability lives in, or whether it is one crate
or several. `OD-CAPABILITY-002`'s criterion governs that question when a second language and a
correspondence declaration both actually exist, unchanged by anything here.

It does not define a correspondence-declaration type as Rust code, a schema format, or an IDL.
It states that a declared correspondence is what makes two providers' facts comparable at all;
defining its shape is the first cross-language capability's own work, judged against this
record the way a program-semantics payload is judged against `OD-ANALYSIS-004`.

It does not enumerate every cross-language claim a future capability may make. The five families
above are the ones the item that reserved this record named, classified against the resolution
vocabulary that already exists — illustration of the reachability test, not a closed list,
exactly as `OD-ANALYSIS-004`'s five worked shapes and `OD-ANALYSIS-006`'s three worked examples
are illustration and not the definition.

It does not reopen `bands.rs`'s same-band rule for `nomos-lang-rust` and `nomos-lang-rust-scan`,
or `P8-SECOND-PROVIDER`'s finding. That rule governs two providers of one capability; this
record governs a different case, two providers of two different languages joined by a declared
seam, and neither changes the other.

## Status

Closed by `P12-CROSS-LANGUAGE`.
