---
id: OD-GATE-018
type: decision
title: Adopting code-standards' waiver mechanism for a nine-phase remediation campaign
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - governance
  - external-tooling
relations:
  - target: OD-CONTRACTS-001
    type: relates-to
---

# Adopting code-standards' waiver mechanism for a nine-phase remediation campaign

## Question

A user-directed campaign runs `kevinmettias/code-standards`' `formatting`, `numerics`,
`placement`, `clean-file`, `code-shape`, `concurrency-and-unsafe`, `declare-types`,
`decompose-file` and `errors` rust gate phases against this workspace and drives each to
zero reported findings, with suppressions minimized. Two things stand in the way of doing
that honestly: `check-visibility-scope` refuses to judge this repository at all without a
`standards.json` at the root, and a large share of the raw findings across every phase are
not defects — they are test code the repository already governs differently, or two
confirmed gaps in the checker's own front end. Neither can be resolved by editing Rust.
This record is the one-time governance decision both need, so ninety-one per-check
judgment calls do not each re-litigate it.

## What Was Measured

**The raw counts are dominated by test code.** A finding was classified as test code when
its line falls inside a `#[cfg(test)] mod … { }` span (found by a stateful scan that blanks
string/char literal contents first — a naive per-line brace count misreads the JSON
fixtures embedded as backslash-continued Rust string literals in, for example,
`crates/host/nomos-api/src/work.rs`, the same class of misread this repository's own
`docs/records` already knows `check-allman-braces`/`check-closing-brace` suffer from).
Measured against the default (non-`--include-tests`) run over `crates/` and `tests/`:
`check-presumption` drops from 159 raw findings to zero once its 159 `#[cfg(test)]`
findings are set aside; `check-literals` drops from 290 to 9 once 257 `#[cfg(test)]`
findings and 24 enum-discriminant misreads (below) are set aside. The same pattern recurs,
at smaller volume, in `check-interfile-duplication`, `check-vertical-organization`,
`check-tuple-return`, `check-error-info`, `check-transposable-parameters`,
`check-responsibility-extraction`, `check-glob-imports`, `check-allman-braces` and
`check-closing-brace`. `clippy.toml` already permits `unwrap()`/`expect()` in test code
repository-wide, and `P12-NAMED-LITERALS`'s own `done_when` already excludes `#[cfg(test)]
mod tests` blocks from what a named-literal defect counts as — this record generalizes
that same governance choice across every check in the nine phases rather than deciding it
once per check.

**Two front-end gaps recur, unchanged from prior measurement.** `check-literals` still
misjudges an enum variant's discriminant (`Usage = 2,`) as a magic number in logic —
confirmed present in the ExitCode-shaped enums (`nomos-cli`'s `agent`, `check`, `gate`,
`request`, `spec`, `work` modules, and `nomos-surface-provenance`'s `exit_code.rs`) that
prior work already found this gap against. Naming a constant per discriminant is what the
checker's own source comment warns against, so these are adopted as findings this
repository does not act on, not fixed.

**`check-visibility-scope` requires the whole repository as its argument, not `crates/` and
`tests/`, and this working tree currently holds four other agents' worktrees under
`.claude/worktrees/` with real, un-merged commits of their own.** A repository-rooted run
during this campaign is measured against a corpus that includes those branches' code, which
this record does not treat as ground truth for this check. `check-visibility-scope`'s real
remediation is deferred to when the shared tree does not carry live, divergent worktrees —
noted here so a future session does not read a repository-root run's count as settled.

## Decision

**Adopt the mechanism, minimally.** `standards.json` is added at the repository root with
only `suppression.max_horizon_days` set (180, the shipped default, stated explicitly rather
than left to imply no opinion was formed); `suppression.markers` is left at its default
(`safety-only`), so the thirteen safety-critical checks — `check-panic-primitives` and
`check-atomic-ordering` among the findings this campaign touches — continue to be argued
with an in-code `// check: allow <reason>` marker beside the code, which is the mechanism
this tool's own design intends for a reader who meets the code without the justification.
Every other exemption this campaign needs routes through `suppressions.json`, populated with
`check waive --adopt` against a filtered, keep-going run so that one categorical decision
(this record) produces the waiver ledger's many mechanical entries, rather than a person
arguing each fingerprint by hand.

**Two categories are pre-decided by this record, not re-argued per finding:**

1. A finding inside `#[cfg(test)]`-gated code, for a check whose concern is test-only
   ergonomics this repository's own `clippy.toml` already permits (`check-presumption`,
   `check-literals`, and the same class in `check-interfile-duplication`,
   `check-vertical-organization`, `check-tuple-return`, `check-error-info`,
   `check-transposable-parameters`, `check-responsibility-extraction`,
   `check-glob-imports`, `check-allman-braces`, `check-closing-brace`) is adopted via
   `check waive --adopt --reason "test code; see OD-GATE-018" --owner @kmettias`, not
   hand-fixed.
2. `check-literals`' enum-discriminant misreads and any array-length-in-type-position
   misread matching the `Digest128`-style public-surface case this repository's own prior
   work already found unfixable are individually waived, citing this record, not fixed.

**What is not pre-decided:** every other finding across the nine phases — the genuine
literals, the real `.expect()` presumption calls in production code, every formatting,
placement, decompose-file, declare-types, concurrency and errors finding not matching one
of the two categories above — is triaged and fixed (or, if a specific case turns out to be
its own structural exception, waived individually with its own stated reason) phase by
phase, in the order: `clean-file`, `decompose-file`, everything else, `placement` last.

## What This Record Does Not Do

It does not waive every finding a check reports; only the two named, general categories
above are pre-decided. Every other finding is a real candidate for a code fix and is
triaged on its own merits in the item that closes its phase.

It does not change `suppression.markers` from the shipped safety-only default, and does not
grant any check outside the shipped thirteen an in-code marker.

It does not resolve `check-visibility-scope` against this repository; that check's real
count is deferred until the working tree is not sharing live, divergent worktrees, per the
measurement above.

It does not adopt any of `standards.json`'s other configuration blocks (naming overrides,
duplication thresholds, language-specific calibration, a standards-tree policy, and so on).
Every field left absent takes the shipped default, which this record deliberately does not
re-argue.

## Status

Accepted, and the campaign it authorized has run to the end. The mechanism landed as
described — a root `standards.json` setting only `suppression.max_horizon_days`, and a
`suppressions.json` now carrying 2224 adopted waivers, the two pre-decided categories
dominating it (`check-presumption` and `check-literals` together account for roughly two
thirds). All nine phases closed: `clean-file` (`P14-CLEANFILE-PHASE-REMEDIATION`, then
`P17-CLEANFILE-*` against its regressions), `decompose-file` (`P15-DECOMPOSEFILE-*`, one item
per crate directory), everything else (`P16-EVERYTHINGELSE-*`, ending at
`P16-EVERYTHINGELSE-FINALSWEEP`'s full 76-check sweep), and `placement` last, as this record
ordered it (`P20-PLACEMENT-*`, with `P22-PLACEMENT-DUP-REGRESSION` closing the two
duplication findings a later fix introduced).

One deferral this record made itself still stands, and stands for the reason it gave:
`check-visibility-scope` is unmeasured against this repository because it requires the whole
repository as its argument and this working tree still carries live, divergent worktrees under
`.claude/worktrees/` and elsewhere. That condition has not changed, so a repository-root run's
count is still not ground truth. Revisit when the shared tree does not carry them.
