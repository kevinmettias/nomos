# Nomos

A persistent software-engineering intelligence and control plane.

Nomos builds and maintains a canonical, multi-resolution model of software, and uses it
to keep intent, architecture, implementation, runtime evidence, automated
transformations and agent work aligned.

This repository is at **Phase 0**. What exists is the coordination substrate every later
phase needs — the protocol vocabulary, the canonical model kernel, the platform port,
the work ledger, and the boundary tests that keep the architecture from eroding. There
is no analysis engine yet.

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

## Coordinating concurrent work

Several agents can work this repository at once. The ledger is what keeps them from
overwriting each other: two items are concurrently claimable exactly when their
territories are provably disjoint, and an unanswerable overlap question refuses the
claim rather than granting it.

```
nomos work list [--state ready|claimed|blocked|done|declined]
nomos work claim   --item <id> --holder <name> [--lease 2h]
nomos work renew   --item <id> --holder <name> [--lease 2h]
nomos work abandon --item <id> --holder <name> --reason <text>
nomos work validate
nomos work audit
```

Exit codes are a contract, because agents branch on them rather than parsing output:

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | validation error |
| 2 | usage |
| 3 | claim unavailable — **retryable**, try another item |
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
