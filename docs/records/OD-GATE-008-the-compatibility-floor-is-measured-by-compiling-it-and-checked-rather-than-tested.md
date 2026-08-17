---
id: OD-GATE-008
type: decision
title: The compatibility floor is measured by compiling it, and checked rather than tested
status: closed
version: 1
authority: canonical-normative-record
tags:
  - gate
  - ci
  - enforcement
  - compatibility
relations:
  - target: OD-GATE-006
    type: relates-to
  - target: OD-GATE-009
    type: relates-to
---

# The compatibility floor is measured by compiling it, and checked rather than tested

## Question

`OD-GATE-006` found `rust-version = "1.85"` in `Cargo.toml` false — nine `&& let` chains are in
this workspace and let chains stabilized in Rust 1.88, so the declared floor cannot compile the
code that declares it — and named the mechanism: every build here used `rust-toolchain.toml`'s
nightly pin, so no compiler near the declared floor was ever pointed at this tree. That record
decided the general principle and named this item to carry the instance. Its own "What This
Record Does Not Decide" section is explicit about the boundary: *"It does not decide what this
repository's supported platforms or minimum toolchain **are**. It decides that whatever they are
declared to be must be exercised. Choosing the floor belongs to `P11-MSRV-UNCHECKED`."*

Three questions follow from that, and none of them is answered by re-arguing the general
principle:

1. What is the floor, established by compiling rather than by reading release notes?
2. What does the gate lane run to keep it honest — `cargo check` or `cargo test` are different
   claims, and `rust-version` makes one specific one?
3. The number now appears in two files, `Cargo.toml` and `.github/workflows/gate.yml`. Is that a
   second authority for one fact, the shape `OD-LEDGER-003` refuses for the lint argv and
   `OD-GATE-002` refuses for the surface check?

## What Was Measured

At `875b4db`, on a machine carrying `rustup` toolchains `1.85`, `1.87` and `1.88` alongside the
workspace's own `nightly`/pinned default.

| Toolchain (`RUSTUP_TOOLCHAIN=`) | `rustc --version` | `cargo check --workspace --all-targets` |
|---|---|---|
| `1.85` | `rustc 1.85.1 (4eb161250 2025-03-15)` | **refuses**, `error[E0658]: let expressions in this position are unstable` |
| `1.87` | `rustc 1.87.0 (17067e9ac 2025-05-09)` | **refuses**, same `E0658`, same sites |
| `1.88` | `rustc 1.88.0 (6b00bc388 2025-06-23)` | exits `0`, `Finished` \`dev\` profile in 8.94s |

Both refusals land on the same class of site across ten files, `nomos-platform-std` first:
`crates/platform/nomos-platform-std/src/filesystem.rs:188`, `.../launcher.rs:241`,
`.../lock.rs:256`, `.../launcher/tests.rs:121`, and further sites in `nomos-ledger`,
`nomos-cli`, `tests/contract` and `tests/contract/tests/completeness_universes/mirrors.rs` — the
same nine call sites `grep -n '&& let'` finds workspace-wide today, confirming `OD-GATE-006`'s
count is still exact and the defect has not grown or shrunk since that record measured it. The
floor is exactly `1.88`, the release immediately below it refuses, and this is a repeat of the
prior session's measurement (`OD-GATE-006`'s "What Was Measured") reproduced independently in
this tree rather than trusted from the record.

A second measurement grounds the third question above. `Cargo.toml`'s `rust-version` was set to
`1.90` — one above the measured floor — and `cargo check -p nomos-cli` was run under
`RUSTUP_TOOLCHAIN=1.88`. Cargo itself refused before compiling a single crate:

```
error: rustc 1.88.0 is not supported by the following packages:
  nomos-analysis@0.1.0 requires rustc 1.90
  nomos-cli@0.1.0 requires rustc 1.90
  ...
```

The experiment was transient — `rust-version` was restored to its measured value immediately
after, and nothing from it is committed.

## The Decision

**The floor is `1.88`, `cargo check --workspace --all-targets` is what exercises it, and the two
files stating it are not a second authority because cargo itself keeps them in the one relation
that matters.**

### The number

`1.88`, matching `Cargo.toml`'s existing two-component style (`"1.85"` before it). Not `1.88.0`:
the third component names a patch release and `rust-version` states a language/library floor, not
an exact toolchain — the distinction `rust-toolchain.toml` needs and `Cargo.toml` does not, since
only the former selects one installable, buildable toolchain rather than describing a boundary
every equal-or-newer compiler satisfies.

Established by compiling, not by reading Rust's release notes for when `let`-chains stabilized —
the table above is the only measurement this record depends on; the changelog is corroboration
that happens to agree, not the source of the number.

### `cargo check`, not `cargo test`

`rust-version` is Cargo's own contract for what a *consumer's* compiler must be to build this
crate. A downstream consumer of a library compiles it; they do not run its test suite, and its
`dev-dependencies` are never part of what they need. `cargo check --workspace --all-targets`
matches that promise at its own grain: `--all-targets` reaches test and bench sources too —
several of the nine `&& let` sites this floor exists for are inside `tests/`, so checking library
code alone would under-measure the same defect `OD-GATE-006` found — while stopping at "does this
compile", which is exactly as far as the promise `rust-version` makes extends.

`cargo test` on the floor is a different, larger claim: that this workspace's own test behavior,
and everything its `dev-dependencies` need, holds on a five-releases-old compiler. Nothing in
`rust-version`'s contract asks for that, and asserting it here would be inventing an obligation
this declaration was never understood to carry — the same shape of error `OD-GATE-006` names in
its own "What Was Considered And Rejected", run the narrow check and let the rest read as though
it covered more than it does, just inverted: over-claiming what passing means rather than
under-claiming what was covered.

### Two files, one relation, cargo enforces it

`Cargo.toml`'s `rust-version = "1.88"` and the gate lane's `RUSTUP_TOOLCHAIN: 1.88` are not an
independently-typed pair that a human must remember to keep matched, the shape `OD-LEDGER-003`
and `OD-GATE-002` both refuse elsewhere in this workspace. The relation that matters —
*the pinned toolchain must be no older than the declared floor* — is enforced by cargo itself,
measured above: raise `rust-version` past the lane's pinned toolchain and the very next run of
that lane refuses immediately, with cargo's own message naming every package and the version it
now requires, before a line of workspace code is even reached. That is a louder and faster
failure than a human-maintained cross-reference would produce, and it fires from the side that
can drift silently — a declared floor rising without the lane being told.

The other direction — the lane's compiler failing to actually support code the declaration
claims it does — is not something cargo checks by comparing two numbers; it is what running the
lane *is*. That was `OD-GATE-006`'s whole finding: a floor nothing compiles reads as checked
while being false. The lane's `cargo check` at the pinned version is the mechanism for that
direction, the same way `nightly` failed to be one only because nothing that old was ever
invoked.

So the number is written twice because it is two different kinds of statement — a promise to
consumers, and an instruction to a CI runner — and both directions in which they could go out of
sync are caught by something that runs rather than by a rule that must be remembered. That is the
test `OD-LEDGER-003` applies to the lint argv and `OD-GATE-002` applies to the surface snapshot,
passed here for a different pair of files.

### The lane

Two steps, `ubuntu-latest` only — matching `Lint` and `Rules` above, and for the same measured
reason `OD-GATE-009` gives them: this workspace has no `cfg(windows)`, `cfg(unix)` or
`cfg(target_os = ...)` anywhere, so a compilability question has no host in it and a second run on
Windows would recheck an identical token stream at full cost.

```yaml
- name: Install compatibility floor toolchain
  run: rustup toolchain install 1.88.0 --profile minimal --no-self-update

- name: Compatibility floor
  env:
    RUSTUP_TOOLCHAIN: 1.88.0
  run: cargo check --workspace --all-targets
```

`RUSTUP_TOOLCHAIN`, not `cargo +1.88.0`. `rust-toolchain.toml` pins a channel, and only
`RUSTUP_TOOLCHAIN` overrides that pin — a `+toolchain` argument selects among installed
toolchains but does not out-rank a channel file present in the working directory, so it would not
reliably beat whatever that file names on a given day. Pinned to the full patch release the floor
was measured on, `1.88.0`, rather than `1.88` or `stable`, so the lane checks the exact compiler
this record's table names and not whatever "1.88-something" or "stable" resolves to that morning
— the same reasoning this workflow already applies to `actions/checkout` and `cargo-deny`.

## What This Record Does Not Decide

It does not decide `rust-toolchain.toml`'s channel. That file is outside this item's territory;
whatever it names, the lane above pins its own toolchain explicitly rather than trusting the
ambient default, which is what makes the lane's claim independent of that file's future edits.

It does not decide the drift direction cargo does not catch — a declared floor that is *lower*
than what the code truly needs, with the gate's own pinned toolchain masking it because the pin
and the false-low declaration happen to agree. `P11-FLOOR-AGREEMENT` carries that; this record's
"cargo enforces it" claim above is scoped to the one direction measured, a floor stated *higher*
than the toolchain in use.

It does not decide whether this workspace should track stable as it advances, or drop the
declaration. Both were live answers to "what should `rust-version` say" and both are rejected
below rather than left unconsidered.

## What Was Considered And Rejected

**Track stable rather than pin a number.** Rejected. `rust-version` with no gate exercising it is
exactly the false-declaration shape `OD-GATE-006` found; a floor that moves with whatever stable
happens to be on release day is not compiled against by anything either, unless the lane
re-resolves "stable" every run — which reintroduces the floating-reference problem this workflow
already refuses for `actions/checkout` and `cargo-deny`, for the same reason: a lane whose meaning
changes on a morning nobody committed anything is not a lane anyone can reason about from the
diff.

**Drop the declaration.** Rejected. `rust-version` is not merely unwanted metadata here; the
measurement above gives it a true, checked value at negligible cost, and Cargo's own dependency
resolution consults it (measured directly: raising it past the running toolchain made cargo
refuse before compiling anything). Deleting a claim that can be made true and cheaply enforced is
not the same move as deleting `deny.toml`'s ban would have been — the case `OD-GATE-006`
describes as "the right answer only where the claim is not wanted".

**Pin a number.** Chosen, at `1.88`, for the reasons above.

**Run the compatibility lane on both operating systems.** Rejected on the same measured basis
`OD-GATE-009` used for `Lint` and `Rules`: no `cfg` anywhere in the workspace means a second run
checks an identical token stream, and this workflow already declines to pay that cost where it
buys no additional coverage.

## What Holds It

The lane itself, by running rather than by asserting, on `ubuntu-latest` on every push to `main`
and `dev` and on every pull request. `Test_The_Workflow_Should_Not_Appear_Empty` and
`Test_No_Step_In_The_Gate_Should_Excuse_Itself` hold the step the same way they hold every other
step in this file — it cannot be emptied or given `continue-on-error` without a mechanical
refusal.

What does **not** hold it is worth stating, the way `OD-GATE-009` states it for the matrix: no
test asserts `Cargo.toml`'s `rust-version` and the lane's `RUSTUP_TOOLCHAIN` name the same value.
The one direction that matters is cargo's own refusal, measured above; a `RUSTUP_TOOLCHAIN` typo
that pins something *newer* than the declared floor would not be caught by cargo at all, since a
newer compiler than the floor is exactly the permitted case — it would simply mean the lane was
quietly checking a higher floor than the one it publishes, drifting back toward the condition
this record exists to end. Nothing here forecloses that; it is named so the next reader does not
have to rediscover it.

## Status

Closed. `P11-MSRV-UNCHECKED` carries it and discharges the third of `OD-GATE-006`'s four
instances — `rust-version` now states `1.88`, established by compiling rather than by reading
release notes, and the `Compatibility floor` lane exercises it on every run.
