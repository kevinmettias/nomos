---
id: OD-HOST-001
type: decision
title: Choosing a platform, running a verb and rendering its outcome are three crates, not one
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - host
  - cli
  - orchestration
  - platform
  - architecture
relations:
  - target: OD-RULES-001
    type: relates-to
---

# Choosing a platform, running a verb and rendering its outcome are three crates, not one

`nomos-cli`'s `work.rs` imported `FileLock`, `StdFileSystem`, `StdProcessLauncher` and
`SystemClock` from `nomos-platform-std` directly, and its dispatch functions took
`&mut FileLedger<StdFileSystem, SystemClock, FileLock>` by name. Choosing a platform,
running a `nomos work` verb and rendering its outcome were one body of code with no seam a
second adapter could depend on without taking all three — the defect `P10-SERVICE-SEAM`
named, from the README's own plan to put IPC, gRPC and MCP adapters around the same
headless core the CLI already sits on.

Naming a concrete provider in a composition root was never the defect: `OD-RULES-001`
already permits that for `nomos check`. The defect was that nothing separated composition
from orchestration, so a second adapter wanting the work group's claim, lease and finish
sequence would have had to re-derive it, and the two copies would drift with nothing able
to notice.

## The decision

A seam exists. `nomos-work-orchestration` (band 40, `crates/orchestration/nomos-work-
orchestration`) is the middle of the three: it owns the request vocabulary
(`WorkCommand`, `ClaimRequest`, `EndingRequest` — moved from `nomos-cli::work` verbatim)
and one function, `Run`, generic over the four traits `nomos-platform` declares
(`FileSystem`, `Clock`, `CrossProcessLock`, `ProcessLauncher`) rather than over
`nomos-platform-std`'s implementations of them. `Run` takes a command and an
already-constructed, caller-owned `FileLedger<F, C, L>` and process launcher, and hands
back `WorkOutcome` — a typed value carrying exactly what `nomos-ledger`'s own API already
returns (`Reservation`, `ClaimRefusal`, `VerificationRecord`, `FinishRefusal`,
`AddRefusal`, `LedgerDocument`, `LedgerError`) or a small bundle of the board and the
moment it was read (`BoardView`, `ShowView`). Nothing in `nomos-work-orchestration` writes
a line of output, picks an exit code, or names a concrete platform type.

`nomos-cli`'s `work.rs` is now a composition root and a renderer and nothing else: it
builds the platform (`StdFileSystem`, `SystemClock`, `FileLock`, `StdProcessLauncher`),
calls `nomos_work_orchestration::Run` once, and matches the returned `WorkOutcome` against
the `WorkCommand` it already had in hand to produce text and an `ExitCode` — the same
`report.rs` and `listing.rs` functions the module already had, now fed pre-computed values
instead of computing them inline. The work group is the demonstration `P10-SERVICE-SEAM`
asked for: every verb (`list`, `show`, `add`, `finish`, `claim`, `renew`, `takeover`,
`abandon`, `decline`, `validate`, `audit`) is dispatched through `Run`, and a second
composition root gets the same guarantee by supplying its own `F`, `C`, `L` and `P` — or
reusing `nomos-platform-std`'s — and rendering `WorkOutcome` however its own transport
wants to (JSON over gRPC, a UI model over IPC), without vendoring the claim/lease/finish
sequence a second time.

This is not a module boundary. `nomos-work-orchestration` is a workspace member `nomos-cli`
depends on; a second adapter crate depends on it exactly the same way, and neither of them
is `nomos-cli`. The property `P10-SERVICE-SEAM`'s `done_when` asked for — a second adapter
can call the middle one without taking the other two — holds because `nomos-work-
orchestration` names neither `nomos-platform-std` (composition) nor any rendering type
(`ExitCode` stayed in `nomos-cli`, and so did every `writeln!`).

## What stayed out, and why

**Argument parsing** stayed in `nomos-cli::work::parse`. A second adapter is not expected
to parse `argv`; it constructs a `WorkCommand` from whatever its own transport carries.
Moving parsing into the orchestration crate would have made every future adapter's request
format go through a CLI-shaped intermediate for no reason — the vocabulary `WorkCommand`
already *is* the transport-independent shape.

**The directory walk that finds this repository's already-published records** — what
`WorkCommand::Add` needs to decide whether a declared amendment is honest — stayed in
`nomos-cli::work::Published_Records`. `nomos_platform::FileSystem` is read,
atomically-replace and exists; it has no directory-listing operation, so this cannot be
expressed generically without widening a port that has exactly one other consumer for a
question that is this repository's own convention rather than the platform's.
`nomos_ledger::FileLedger::Add`'s own documentation already draws this line for the same
reason. `Run`'s `published: impl FnOnce() -> Territory` parameter is where that value
crosses from the composition root into the orchestration crate — lazily, so it is never
walked for the ten verbs that are not `add`.

**Exit codes** stayed in `nomos-cli::work::ExitCode`. They are a process convention, not a
verb outcome — a JSON adapter has no exit code to pick, and the mapping from a
`ClaimRefusal` to a retryable-versus-fatal number is exactly the rendering step this record
keeps out of the orchestration crate.

## What this costs

One more crate in the workspace, one more row in `README.md`'s band table and
`tests/contract/tests/boundaries/bands.rs`'s `BANDS`, and one more public-surface snapshot
under `tests/contract/surface/`. `nomos-cli::work`'s own module count went down by three
files (`command.rs`, `claim_request.rs`, `ending_request.rs` moved out) and its two
remaining ledger-facing functions (`Render`, `Published_Records`) are shorter than the
`Run` they replaced, because they no longer also decide which ledger call a command means.

The tuple match in `nomos-cli::work::Render` — `match (command, outcome) { ... _ =>
unreachable!() }` — is the one place this record accepts a runtime invariant Rust's type
system does not state on its own: `nomos_work_orchestration::Run` always returns the
`WorkOutcome` variant naming the `WorkCommand` variant it was given, and nothing in either
enum's shape lets the compiler see that without the unreachable arm. A future verb added to
one enum and not the other is caught immediately — the match becomes non-exhaustive or the
new arm falls into `unreachable!()` the first time it runs — rather than silently.

## What would make this wrong

If a second adapter never arrives, this is a seam nobody crossed — a real cost, paid once,
for an option this record cannot prove will be exercised. What would make the seam itself
wrong, rather than merely unused, is `nomos-work-orchestration` growing a second concern:
a rendering decision leaking into `WorkOutcome`'s shape (a variant that exists to be
printed rather than to report what happened), or a platform-specific type leaking into its
signature (an `impl` bound that only `nomos-platform-std` satisfies). Either would be the
same defect this record fixed, one layer up.
