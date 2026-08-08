# Nomos

A persistent software-engineering intelligence and control plane.

Nomos builds and maintains a canonical, multi-resolution model of software, and uses it
to keep intent, architecture, implementation, runtime evidence, automated
transformations and agent work aligned.

This repository is at **Phase 2 complete**. Two things exist.

The coordination substrate every later phase needs — the protocol vocabulary, the
canonical model kernel, the platform port, the work ledger, and the boundary tests that
keep the architecture from eroding.

And the specification system: the corpus lives in a database behind a preservation ledger
that makes silent content loss fail rather than pass. It exists because the previous
specification revision destroyed 282 markdown table rows, all 6 code blocks and 132
sections of narrative, and the mechanism that would have caught it was present and never
ran. See `docs/records/ARC-SPECDB-001-the-specification-is-a-database.md`.

There is no analysis engine yet.

## Layout

Crates are ordered into bands. A crate may depend only on crates in a strictly lower
band, and `tests/contract` asserts it.

| Band | Crate | Owns |
|---|---|---|
| 0 | `nomos-contracts` | Protocol truth. Depends on `serde` and nothing else. |
| 10 | `nomos-model` | Subjects, composite identity, evidence, and the `SubjectSet` exclusion primitive. |
| 15 | `nomos-platform` | Port traits: clock, filesystem, cross-process lock, process launcher. |
| 16 | `nomos-platform-std` | The std implementation of those traits. |
| 20 | `nomos-ledger` | Territory-based mutual exclusion over `work/ledger.json`. |
| 90 | `nomos-cli` | The `nomos` binary. |

The specification system sits beside the kernel rather than above it. It reaches the
product only through a knowledge capability, so nothing in the product may name it.

| Band | Crate | Owns |
|---|---|---|
| 11 | `nomos-spec-model` | The canonical normalizer and hashing. The single authority on what content hashes to. |
| 12 | `nomos-spec-store` | Schema, migrations, and this repository's own governing records. |
| 13 | `nomos-spec-bundle` | Deterministic JSONL export and import — the portable authority committed to git. |
| 13 | `nomos-spec-ingest` | Parsers and the manifest gate against the real v14 corpus. |
| 14 | `nomos-spec-validate` | The `NSV-PRESERVE-*` rules and the run that fails closed. |

The normalizer was **recovered from the corpus, not chosen**: v14's hash generator does
not ship, so the algorithm was reconstructed and verified against all 2,533 recorded
block hashes and all 493 recorded statement hashes. A disagreement in `nomos-spec-model`
makes the entire preservation ledger measure nothing, which is why it is gated by
`crates/spec/nomos-spec-model/tests/normalizer_gate.rs` before anything downstream is
trusted.

## Coordinating concurrent work

Several agents can work this repository at once. The ledger is what keeps them from
overwriting each other: two items are concurrently claimable exactly when their
territories are provably disjoint, and an unanswerable overlap question refuses the
claim rather than granting it.

```
nomos work list [--state ready|claimed|blocked|done|declined]
nomos work add     --item <id> --title <text> --why <text> --done-when <text>
                   --territory <path> [--territory <path> …]
                   [--territory-pattern <glob> …] [--depends-on <id> …]
                   [-- <program> <args…>]
nomos work claim   --item <id> --holder <name> [--lease 2h]
nomos work renew   --item <id> --holder <name> [--lease 2h]
nomos work finish  --item <id> --holder <name>
nomos work abandon --item <id> --holder <name> --reason <text>
nomos work validate
nomos work audit
```

**Territory** is written as repository paths, not identifiers, because the ledger is
committed and reviewed in a `git diff` — a diff of digests is a diff nobody reads. Paths
are compared after normalization, so `./crates\A\src\Lib.rs` and `crates/a/src/lib.rs`
are one subject, and a directory contains the files beneath it. An item that reserves
nothing is refused: it would exclude nobody while looking like work.

**Finishing runs something.** `done_when` is prose for a human; everything after `--` is
an argument vector that gets executed, with no shell between what was written and what
runs. `finish` records the item done only if that exits zero, and it keeps three answers
apart — the predicate failed (the work is not done), the predicate could not be started
or timed out (nobody found out), and there is no predicate at all (nothing was checked,
which must never read like everything checked out).

Exit codes are a contract, because agents branch on them rather than parsing output:

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | validation error |
| 2 | usage |
| 3 | claim unavailable, or a dependency is unfinished — **retryable**, try another item |
| 4 | conflict — a human has to resolve it |
| 5 | the ledger or its lock could not be used at all |

The distinction that earns its own code is 3 versus 5. An agent told the item is taken
should pick up something else; an agent told the ledger is broken should stop and fetch
a person. Collapsing those into "non-zero" makes the first indistinguishable from the
second.

## Conventions

Function names are `Pascal_Snake_Case` and control flow uses explicit `return` and
Allman braces, matching the sibling `xvpe` workspace. Types stay `UpperCamelCase`.

`cargo fmt` **cannot** produce this style and damages the tree when run — rustfmt has no
option that puts a control-flow brace on its own line, so it rewrites the workspace's
form back to K&R every time. `cargo fmt --check` is deliberately not a gate step: a gate
containing two contradictory checks is one that can never be green, and a gate that can
never be green is one everybody learns to ignore. See `rustfmt.toml`.

`unwrap_used`, `indexing_slicing`, `arithmetic_side_effects` and `float_cmp` are denied.
A panic on the analysis path is a determinism defect rather than merely a crash: a replay
must reach the same panic at the same step.

## Running the gate

```
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
