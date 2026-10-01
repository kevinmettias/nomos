---
id: OD-CAPABILITY-012
type: decision
title: Whether a language server is a provider, and of what
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - analysis
  - architecture
relations:
  - target: OD-ANALYSIS-010
    type: relates-to
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: ARC-CONFORMANCE-001
    type: relates-to
---

# Whether a language server is a provider, and of what

## Question

`P42-SEMANTIC-FACT-FAMILY` and `P40-COMPILER-BACKED-PROVIDER` both assume resolved-semantics
work means a compiler integration per language. A language server is the other candidate,
and its shape looks right on its face: a subprocess behind a capability, exactly what
`nomos-lang-rust-cargo`, `nomos-lang-rust-clippy` and `nomos-lang-rust-deny` already are, and
one protocol reaching every language this system might add rather than one integration per
language. `OD-ANALYSIS-010` declined embedding a compiler or language server for one fact,
control-flow reachability's sound tier, and was explicit that it forecloses nothing for a
future rule whose subject genuinely needs one. Whether that future has arrived for the class,
not one fact, needed checking directly.

## What Was Measured

**`P40-COMPILER-BACKED-PROVIDER` is done, and it answered the class question by the other
route.** `nomos-lang-rust-compiler` is a real, shipped provider of `nomos.cap.rust.
copy_clones`, built against a direct compiler semantic API, not a language server. The
premise that "no provider uses a real compiler semantic API" is no longer true; what remains
open is only whether an *language-server-shaped* route is also warranted beside it, not
whether resolved semantics can be had at all.

**A language server is not, in fact, the same subprocess shape its siblings are.**
`nomos_platform::ProcessLauncher`'s one method, `Run`, is documented precisely: "runs the
command to completion and captures its output." Every real provider today —
`nomos-lang-rust-cargo`'s `cargo metadata`, `nomos-lang-rust-clippy`'s `cargo clippy`,
`nomos-lang-rust-deny`'s `cargo deny` — is exactly that: spawn, run to completion, parse
stdout, exit. A language server is the opposite shape on purpose: a long-lived process
initialized once, holding an incrementally-maintained index, answering many requests over
one session through a bidirectional JSON-RPC channel that is never expected to close between
queries. Nothing in `nomos_platform` spawns a process, holds it open, and exchanges more than
one message with it. Forcing a language server into `ProcessLauncher::Run` would mean
spawning a fresh server, paying its full workspace index cold-start, for every single query —
discarding the one property that makes a language server worth using instead of a direct
compiler call, and answering the query more slowly and less honestly than
`nomos-lang-rust-compiler`'s own direct-API shape already does for Rust.

**A language server's own resolution is not sound the way a direct compiler call is, and this
workspace's `Assurance` has no variant for the difference.** `nomos_contracts::Assurance` is
exactly three values: `Sound` ("the property is claimed and the claim is backed"), `Unsound`,
and `Unknown` ("nobody has established this either way"). `rust-analyzer` and its siblings are
built for editor responsiveness over an incomplete or mid-edit tree, and are documented to
answer some queries — under heavy macro expansion, certain generic instantiations, or a
buffer mid-edit — best-effort rather than backed by a real compilation the way `rustc` itself
is. A fact sourced from a language server could not honestly claim `Assurance::Sound` on
those axes the way `nomos-lang-rust-compiler`'s direct-`rustc`-API route already can; it would
have to declare `Unknown` for whatever it cannot back, a strictly weaker guarantee than the
provider this workspace already shipped for the same class of question.

## The Decision

**A language server may be a provider in principle — the shape argument is right, one
protocol reaching every language rather than one integration per language — but it is not
buildable today, on two concrete grounds rather than a preference.**

First, `nomos_platform` has no port for a long-lived, bidirectionally-communicating process.
Building one is a real platform-layer design question of its own weight — what a session
lifecycle looks like, how many concurrent sessions a run may hold, how a session outlives or
does not outlive one `nomos check` invocation — not a detail a provider's own implementation
could improvise past `ProcessLauncher::Run`'s existing one-shot contract.

Second, whatever capability a language-server-backed fact would answer — `nomos.cap.lsp.
definition`, `nomos.cap.lsp.references`, `nomos.cap.lsp.symbol_hierarchy`, one per LSP request
shape, provider-neutral in the sense the field asked for — cannot honestly declare the
`Assurance::Sound` ceiling `nomos-lang-rust-compiler` already proved reachable for the
identical class of question (needing resolved types, in that case whether a type is `Copy`)
by calling a compiler directly. A language server is the right mechanism only for a fact
whose own honest ceiling is `Assurance::Unknown` on at least one axis, and no rule in this
workspace has yet named a question that needs exactly that shape of answer rather than a
sound one.

`IncrementalGranularity` is not decided here either, and should not be read as merely `File`
versus `Project`: a language server surviving across separate `nomos check` invocations is a
staleness question this system's own reassessment-cache model has no story for yet — who
restarts a session, on what signal, and how the cache learns the session's own index went
stale relative to disk are all open, and none is answered by picking a granularity value.

**This does not change what `OD-ANALYSIS-010` already decided, and confirms its direction
rather than reopening it.** That record covered one fact and explicitly left a genuine
compiler-frontend need open for later; this record answers the wider class question it did
not reach, and agrees: prefer a narrow, direct compiler-API integration — the shape
`nomos-lang-rust-compiler` already proved — over a language-server integration, for the
identical reason `OD-ANALYSIS-010` gave for control-flow reachability's tier 2, now
reinforced by two further, concrete grounds this record adds: the process-shape mismatch and
the `Assurance` ceiling.

**`P40-COMPILER-BACKED-PROVIDER` is satisfied by the direct-API route it already took, not by
this one.** A language-server provider, if one is ever built, would be a separate mechanism
answering a separate capability — editor-navigation-shaped facts a direct compiler API does
not expose the same way — not a substitute for what `P40` already delivered.

## What This Record Does Not Do

**No provider is built here.** It does not design the long-lived-process platform port a
language-server provider would need, and does not name that design's own territory beyond
stating that it is real, separate work.

It does not block `P42-SEMANTIC-FACT-FAMILY`. That item's own `depends_on` is already
satisfied by `P40-COMPILER-BACKED-PROVIDER`'s real delivery; a claimant may proceed by the
direct-compiler-API route that route already proved, entirely independent of this record's
own question about language servers specifically.

It does not foreclose a language-server provider forever. It names the two concrete things
that would have to be true first — a real platform capability for a long-lived,
bidirectional process, and a real rule whose own question is honestly answered at
`Assurance::Unknown` rather than needing the `Sound` ceiling a direct compiler call already
reaches — rather than leaving "maybe someday" standing in for either.

## Status

Accepted. A language server is the right shape for one protocol reaching every language, but
not a provider today: `nomos_platform` has no long-lived-process port to host one, and no
rule has yet named a question honestly answered at `Assurance::Unknown` rather than the
`Sound` ceiling a direct compiler-API provider — already shipped for Rust — can reach instead.
