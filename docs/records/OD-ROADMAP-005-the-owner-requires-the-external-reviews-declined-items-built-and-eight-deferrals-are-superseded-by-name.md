---
id: OD-ROADMAP-005
type: decision
title: The owner requires the external review's declined items built, so eight deferrals are superseded by name and their measurements stand
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - roadmap
  - layering
  - project
relations:
  - target: OD-ROADMAP-001
    type: relates-to
  - target: OD-CAPABILITY-008
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
  - target: OD-PACKAGE-014
    type: relates-to
  - target: OD-EXECUTOR-004
    type: relates-to
  - target: OD-EXECUTOR-005
    type: relates-to
  - target: OD-PACKAGE-016
    type: relates-to
  - target: OD-HOST-011
    type: relates-to
  - target: OD-HOST-012
    type: relates-to
  - target: OD-PLATFORM-003
    type: relates-to
  - target: OD-CAPABILITY-017
    type: relates-to
  - target: OD-AGENT-004
    type: relates-to
  - target: OD-LEDGER-041
    type: relates-to
---

# The owner requires the external review's declined items built, so eight deferrals are superseded by name and their measurements stand

## Question

An external architecture review of `dev` at `bc0aaac` was reconciled against `docs/records/`
on 2026-09-21, claim by claim. Three of its thirteen points produced code changes, three
produced new decision records, one was already built, and six were declined because an accepted
record had already measured the same argument and named a trigger that had not fired. Two
further points had no disposition: the increment `OD-LEDGER-041` names as owed, and the rule the
review's own thirteenth section asks for.

The owner then required the declined set built anyway.

That is a sequencing decision, and it is the owner's to make rather than a session's. What it
needs from this record is not permission but **bounds**: which deferral each piece supersedes,
what stays true, and what the override does not reach. Without them, six implementations would
each read as an agent building against an accepted record that says not to, and a later session
reconciling the same review would find six records saying wait and a tree saying otherwise with
nothing to say which is current.

## What Is Not Being Claimed

**None of the six declines was wrong on its own evidence, and this record does not say
otherwise.** Each measured a real population at a real commit, and those measurements are cited
by the items that now supersede them rather than deleted. `OD-CAPABILITY-008` really did find
that three of the provider convention's four parts agree and the fourth diverges for a principled
reason. `OD-EXECUTOR-005` really did find that Ollama is a `ModelBackend` and not a second
`AgentExecutor`. `OD-AGENT-004` really did find that the two artifacts a review wanted compacted
were the two that had not gone stale.

**So the override is about *when*, not about *whether the reasoning held*.** This repository's
own precedent is `OD-ROADMAP-001`, which retired a population-of-zero caution for a named cluster
on exactly this footing: build the shape now, from the requirement text and the types that exist,
rather than wait for a second consumer to justify it. That record has been cited by items and
declined against reviews six times since, which is what a bounded override looks like when it is
working.

## The Decision

**Build the eight pieces below. Each supersedes the named clause of the named record, at the
version stated, and nothing else in that record moves.**

1. **The analysis providers are composed rather than named by the check service.**
   Supersedes `OD-CAPABILITY-008` v3's second trigger, that no consumer holding providers
   polymorphically exists yet; `OD-HOST-004` v2's conclusion that a registry never needs a
   selection mechanism because composition is not choice; and `OD-PACKAGE-014`'s foreclosure of
   activation driving composition. What is authorized is that
   `nomos-check-orchestration` stops naming the capability, language and repository provider
   crates directly and receives them from a composition root.

2. **A port stands between the generic agent path and its concrete backends.**
   Supersedes `OD-EXECUTOR-004` v1 and `OD-EXECUTOR-005` v2's shared trait trigger, that no
   second real `AgentExecutor` exists, and `OD-PACKAGE-016` v1's first decision section, which
   placed the resolver in `nomos-agent-orchestration` because it is the one crate reaching both
   backend crates. What is authorized is that the generic path names the port and a composition
   root supplies `nomos-agent-executor-claude-code` and `nomos-model-backend-ollama`.

3. **An application operation surface exists and the hosts project it.**
   Supersedes `OD-HOST-011` v1's refusal of a contracts crate on `OD-PACKAGE-015`'s third clause
   and `OD-HOST-012` v1's decision that repo-tooling handlers keep their present home until a
   second host wants the product half alone. What is authorized is one operation surface the
   hosts depend on in place of assembling the product themselves.

4. **`nomos-platform` compiles without XVPE.**
   Supersedes `OD-PLATFORM-003` v1 only where that record's retirement of `AGT-006`'s
   no-dependency clause reaches `nomos-platform` itself. The crossing stays adopted and stays
   pinned, `nomos-platform-xvpe` stays the adapter, and what moves is the clock type the ports
   crate re-exports. This does not restore the clause for the workspace, and it does not make
   XVPE a peer again: Nomos remains an application over it.

5. **The review-finding capability contract is its own crate.**
   Supersedes `OD-CAPABILITY-017` v1's closing decision that neither bundled crate is split by
   that record, and `OD-CAPABILITY-002`'s licence to keep a contract beside its single provider
   until a second provider contends for it. The vendor connector keeps the provider; the contract
   a rule reads moves out from under the vendor's name.

6. **The architecture prose is generated or checked against one model.**
   Supersedes `OD-AGENT-004` v2's decline of generated README tables and compacted manifest
   commentary. The measurement that decline rested on stays true and is the reason the increment
   is a projection or a check rather than a rewrite: what is authorized is that a fact
   `nomos-architecture.json` holds stops being restated in prose that nothing compares.

7. **The board listing defaults to the live board.** Supersedes nothing. `OD-LEDGER-041` named
   this as owed when it refused the archive, and it is listed here so the set is complete.

8. **A rule states what a host may know of the orchestration graph.** Supersedes nothing, and is
   the one piece whose output is a record rather than code. The review's thirteenth section asked
   for it, both hosts name eight orchestration crates today, and no record states the rule.

## What This Does Not Do

**It does not license scope beyond those eight.** A piece that turns out to need a ninth change
is a new item and, if it reaches another record's decision, a new question for the owner.

**It does not let an item skip its record amendment.** Each superseded record is amended by the
item that lands its change, in the same commit or the one after, so that a reader arriving at
`OD-HOST-011` finds the override rather than a contradiction. An implementation that lands
without its amendment leaves the record set lying, which is worse than the deferral it replaced.

**It does not grant a zone permission by fiat.** Several of these pieces move an edge — a
composition root reaching providers, a host reaching one surface instead of eight. Every such
edge is declared in `nomos-architecture.json` by the item that needs it and judged by the same
rule and the same contract assertions as any other edge. This record changes no permission.

**It does not retire a trigger in any record not named above**, and it does not reopen the three
decisions this review already produced: `OD-CONTRACTS-006`, `OD-LEDGER-041` and
`OD-PROJECT-004` v2 stand as written.

**It does not promise an order.** The pieces contend for the same files and this tree is shared
with live sessions, so which lands first is a coordination outcome rather than a decision. An
item blocked on a peer's claim waits; it does not reach in.

## Consequences

Eight items are authored `--origin required` citing this record, each carrying its own territory,
its own verification predicate and the amendment its superseded record needs. This record moves
no code and amends nothing by itself.

## Status

Accepted. The bound is the enumeration above: eight pieces, each against a named clause of a
named record at a stated version, with every measurement those records made left standing.
