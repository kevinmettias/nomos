---
id: OD-GATE-012
type: decision
title: A toolchain pinned for a command this workspace forbids running is pinned for nothing
status: closed
version: 1
authority: canonical-normative-record
tags:
  - gate
  - toolchain
  - rustfmt
  - enforcement
relations:
  - target: OD-GATE-006
    type: relates-to
---

# A toolchain pinned for a command this workspace forbids running is pinned for nothing

## Question

`rust-toolchain.toml` pinned this workspace's channel to `nightly`, and its own comment gave one
reason: `cargo fmt` needs the unstable `brace_style = "AlwaysNextLine"` option to express this
workspace's Allman brace style. `rustfmt.toml` and `README.md` both state, independently and in
their own words, that `cargo fmt` **cannot** produce this workspace's style regardless — rustfmt
has no option that puts a control-flow brace on its own line, so it rewrites the workspace's form
back to K&R every time it runs — and that `cargo fmt --check` is deliberately not a gate step,
because a gate holding two mutually contradictory checks can never be green.

So the toolchain was pinned to enable an option belonging to a command this repository already
forbids running. That is `OD-GATE-006`'s shape in a new instance: not a declaration nobody
executes, but a mechanism (`unstable_features`) that is fully wired up and genuinely inert,
because the one command that would exercise it is banned by two other files.

The cost was not confined to this file. Because every build here and in CI used nightly, no
compiler near the declared compatibility floor was ever pointed at this tree — which is exactly
how `Cargo.toml`'s `rust-version = "1.85"` came to be false while reading as checked. That gap is
`P11-MSRV-UNCHECKED`'s subject, not this record's; this record cites its measurement below without
claiming its outcome.

## What Was Measured

At `3cd2261`, under `P11-MSRV-UNCHECKED`'s own measurement, cited by this item's own why-text:
`cargo check --workspace --all-targets` on stable `1.88.0` exits `0` with zero errors. The pin is
not load-bearing for compilation. That measurement is `P11-MSRV-UNCHECKED`'s own to govern; this
record relies on the number without asserting the record that will carry it.

Searched directly for this record, and found nothing else nightly depends on:

| Checked | Result |
|---|---|
| `#![feature(...)]` anywhere in the workspace | none — the one `feature` hit in the tree (`crates/substrate/nomos-workspace/src/variant.rs`) is the `BuildVariant.features` field, an ordinary struct member, not an unstable-feature attribute |
| `cargo-features = [...]` in any `Cargo.toml` | none |
| Nightly-only Cargo resolver or workspace setting | none — `resolver = "3"` and `edition = "2024"` are both stable as of 1.85 |
| A workspace lint requiring nightly (`private_interfaces`, `private_bounds`, `unsafe_code`, the `clippy::*` group levels) | none — all stable lints |
| `clippy.toml` | one key, `allow-unwrap-in-tests`; nothing nightly-gated |
| Any `build.rs` reading a nightly-only `rustc`/`cargo` feature | none — the four build scripts (`nomos-spec-store`, `tests/integration`, `nomos-spec-project`, `nomos-cli`) read only `TARGET`, `PROFILE`, `RUSTUP_TOOLCHAIN` and `CARGO_FEATURE_*`, and record whatever string `RUSTUP_TOOLCHAIN` holds without requiring a particular one |
| `.github/workflows/gate.yml` for any other nightly dependency | none beyond the "Show toolchain" step's comment, which repeats the same spent reason this record retires (see "What This Record Does Not Decide") |

The nine `&& let` chains `OD-GATE-006` counted are not a nightly dependency either: let chains
stabilized under edition 2024 in Rust 1.88, which is a stable release and is the same floor this
record pins.

The pin was buying exactly one thing — an unstable rustfmt option — and that thing was already
unreachable by policy. Nothing else in the tree depends on a nightly compiler, a nightly-only
language feature, or nightly-only tool behavior.

## The Decision

**Drop to stable, pinned at the exact release the workspace is measured to compile on: `1.88.0`.**

Of the three answers this item named, the other two were considered and are recorded here rather
than silently passed over.

**Keep nightly for a different, real reason.** Rejected — the search above found none. A record
claiming an undisclosed reason exists would be exactly the kind of declaration `OD-GATE-006`
distrusts: unexecuted, unverifiable, and asserted rather than found.

**Keep nightly and lift the `cargo fmt` ban.** Rejected, and by a wide margin — this would mean
directly contradicting two other files' explicit, reasoned statements (`README.md`'s Conventions
section and `rustfmt.toml`'s own warning) that `cargo fmt` damages this tree's style on every run,
for a defect that is architectural (rustfmt has no control-flow-brace option at all) rather than
configurational. Lifting the ban does not fix the half-rule problem; it would reintroduce a
gate step this repository already rejected once for being unwinnable. Nothing found while
preparing this record weighs against the reasoning `README.md` and `rustfmt.toml` already state.

**Drop to stable.** Chosen. It matches what the workspace already measures true elsewhere
(`OD-GATE-008`'s floor), it costs nothing the search above could find, and it removes a pin whose
only stated purpose was already unreachable.

`rust-toolchain.toml` now reads:

```toml
[toolchain]
channel = "1.88.0"
components = ["rustfmt", "clippy"]
```

pinned to an exact release rather than to the floating name `stable`, for the same reason this
repository pins GitHub Actions to a commit rather than a tag (`OD-GATE-006`): a floating reference
changes what every build runs on a morning nobody committed anything.

## What This Costs

**What the losing side (keeping nightly) would have cost:** nothing measurable was found, which
is the point of the search above — keeping the pin was buying access to an option no permitted
command may invoke. The genuine cost of keeping it was invisible and cumulative: every build
using a channel newer than any declared floor is exactly the condition that let `rust-version`
sit at a false `1.85` unnoticed, per `OD-GATE-006` and per `P11-MSRV-UNCHECKED`'s own measurement.

**What dropping it costs:** every contributor and every CI run now compiles with a fixed
`1.88.0` toolchain instead of whatever nightly resolved to that morning. Nightly-only behavior
nobody has gone looking for could exist and would now surface as a build failure rather than
silently continuing to work — that risk is accepted rather than eliminated; the search above is
what was checked, not a proof of absence. `rustfmt.toml`'s `unstable_features = true` and
`brace_style = "AlwaysNextLine"` keys are now configuration for an option a stable toolchain's
`rustfmt` refuses outright rather than silently ignores — a louder failure than before if anyone
ever runs `cargo fmt` here, which is the intended failure mode given the ban already in place.

`crates/substrate/nomos-workspace/src/variant.rs`'s `BuildVariant.toolchain` field, captured from
`RUSTUP_TOOLCHAIN` by the `nomos-cli` and `tests/integration` build scripts, will now read a
stable-release string (e.g. `1.88.0-x86_64-pc-windows-msvc`) instead of a string naming `nightly`.
Because that field is part of a build variant's identity, any fact cached under a `nightly`
variant is simply keyed differently going forward — not corrupted, not silently reused for the
wrong build, which is exactly the property `variant.rs`'s own module comment argues the identity
exists to hold.

## What This Record Does Not Decide

It does not correct `.github/workflows/gate.yml`. That file's "Show toolchain" step carries a
comment repeating the now-spent reason ("It is nightly because rustfmt's Allman brace option is
unstable"), and this item's territory does not reach it — `.github/workflows/gate.yml` is
concurrently claimed by `P11-MSRV-UNCHECKED`, which is already amending that file to add a
compatibility-floor lane. Whichever item next touches that comment should correct it; this record
states plainly that it is now stale so the next reader does not have to rediscover that from
scratch.

It does not decide anything about lifting the `cargo fmt` ban. That question stays answered by
`README.md`'s Conventions section and `rustfmt.toml`, unchanged in substance here.

It does not decide the compatibility floor itself. `P11-MSRV-UNCHECKED` is the item measuring and
setting `Cargo.toml`'s `rust-version`, under whatever record identifier it registers; this
record's `1.88.0` toolchain pin tracks that measurement — cited above by commit and by number —
rather than re-deriving it or asserting the record that will carry it.

## What Holds It

`rust-toolchain.toml` itself, honoured automatically by `cargo` and `rustup` on every invocation
in this tree and in CI. There is no test asserting the channel is a particular value or is not
`nightly` — a later editor repinning to nightly (for a real reason or otherwise) is caught by
nothing mechanical, only by this record and by whatever the item that does it is required to
search, the same way this one was.

## Status

Closed. `P11-NIGHTLY-PURPOSE` carries it.
