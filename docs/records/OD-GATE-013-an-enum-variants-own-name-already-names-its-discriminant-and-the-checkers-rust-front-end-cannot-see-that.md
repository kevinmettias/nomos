---
id: OD-GATE-013
type: decision
title: An enum variant's own name already names its discriminant, and the checker's Rust front end cannot see that
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - lint
  - code-standards
relations:
  - target: OD-GATE-007
    type: relates-to
---

# An enum variant's own name already names its discriminant, and the checker's Rust front end cannot see that

## Question

`check-literals`, one of the checks the code-standards suite's rust `clean-file` phase runs
over this workspace, reads a bare numeric discriminant on an enum variant (`Usage = 2`) as a
magic number in logic and asks for it to be named as a constant. Its own `--help` text states
the rule's purpose: an unexplained numeral should be replaced by a name that says what the
value means. An enum variant already **is** that name — `ExitCode::Usage` says what `2` means
at every call site that matches on it — and the discriminant beside it is not a second,
unnamed value; it is the same name's numeral spelling, placed where a reader checking the
wire value can find it beside the name that explains it.

The checker's Rust front end does not see that distinction. `rust_Literal_Context`
(`language-kernels/programming/rust/rustlang/rust_literals.go` in the code-standards
repository) returns `CONTEXT_DECLARATION` for `NODE_CONST_ITEM` and `NODE_STATIC_ITEM`, and
has no case for a variant's discriminant, so `Usage = 2` inside an enum body is judged as
logic — the same bucket a numeral typed into a function body falls into. Kotlin's front end
handles the identical shape correctly (`HIGH(9)`); this is a gap specific to this one
front end, not a property of what the rule is checking for.

This workspace has four instances of one deliberate pattern the gap flags in full:
`crates/host/nomos-cli/src/spec/exit_code.rs`, `crates/host/nomos-cli/src/check/exit_code.rs`,
`crates/host/nomos-cli/src/work/exit_code.rs`, and the `ExitCode` enum embedded in
`crates/host/nomos-cli/src/request.rs`. Every one is a `#[derive(Clone, Copy, Debug,
PartialEq, Eq)] enum ExitCode` whose own doc comment states a cross-binary contract — "the
numbers are shared with every other group on this binary: an exit code means one thing per
binary rather than one thing per group" (`spec::exit_code`'s own words) — and every variant
carries a doc comment naming what that specific number means to an agent branching on it
instead of parsing output. `check-literals` currently reads thirteen findings across the
three standalone `exit_code.rs` files (six, three and four) and four more inside `request.rs`'s
embedded enum, all of the identical shape.

## Decision

**A documented, agent-facing exit-code enum's discriminants are a class this workspace's use
of `check-literals` treats as outside the finding's scope, by this record rather than by an
in-code marker or a waiver.** The variant name is the constant the checker's own `--help`
text asks for; renaming the numeral to a second, freestanding constant beside it would only
restate the variant name a second time, further from the declaration that already carries it
and the doc comment that explains it — the anti-pattern the checker's own source comment for
`CONTEXT_DECLARATION` warns a caller away from introducing.

This is deliberately not an in-code suppression and not a `suppressions.json` waiver. This
workspace has no root `standards.json`, so the default `markers: safety-only` policy rejects a
`// literal: allow <reason>` marker for `check-literals` — one of the checks not on the
thirteen-check safety-critical list, confirmed by testing the marker directly against
`check/exit_code.rs` and reverting it. A waiver needs a `standards.json` beside it and a
bounded expiry (`max_horizon_days`, default 180 days): that mechanism exists to retire
temporary debt on a clock, and this is not temporary debt. It is a considered, permanent
reading of what the rule is for, applied where the checker's own front end cannot currently
apply it. A record is the mechanism this workspace has for a decision of that shape.

## What This Costs

**`check-literals` continues to report a finding at each of these seventeen lines**, and a
fresh, unexplained run of the check over this workspace keeps listing them without this
record in view. The next reader of a raw `check-literals` run has to know to consult this
record before treating that count as a defect count.

**The exception is stated as a class, not as a closed file list, and that is itself a risk.**
It covers a documented, per-variant-explained, cross-binary-contract enum's own
discriminants — so a fifth such enum is the same decision already made and not a new one to
argue — but nothing mechanical currently prices a variant that stops carrying a doc comment,
or an enum that claims the same shape without actually being a cross-binary contract. `nomos
check --root .` and the gate's clippy pass do not enforce a doc comment on every variant, so
that boundary is held by review rather than by a check today.

## Consequences

None of the four files named above needs a code change on account of `check-literals`. Their
combined finding count — seventeen, at the time this record was written — is the expected,
permanent shape of a clean run over this workspace's `check-literals` results, not a
regression to chase toward zero.

## What Holds It

Nothing mechanical enforces the exception's boundary today — see the risk named above. The
four enums it currently covers are held by the ordinary review a change to any of them
already gets: each binary group's own tests exercise its exit behaviour, and a variant added
without a doc comment, or an enum claiming this shape without the cross-binary contract that
justifies it, is a review-visible omission before it is anything else.

## What This Record Does Not Decide

It does not except the checker's separate, unrelated gap on an array length in type
position. `pub struct Digest128([u8; 16])` in
`crates/contracts/nomos-contracts/src/identity.rs` hit the same front end for a different
reason — `NODE_CONST_ITEM` inside a type declaration, not an enum discriminant — and was
closed by naming the length, not by exception, because the number is a public const
(`Digest128::BYTE_LENGTH`) the type already exposes; a private array length in the same
position, where no exported const exists to reuse, is a plain unnamed-literal fix and not a
case this record speaks to either way. It does not build a `suppressions.json` or a
`standards.json` for this workspace. It does not fix the front end's gap in the code-standards
tool itself — that tool is a separate repository this one depends on, and its `go test ./...`
baseline is documented elsewhere as already red for unrelated reasons; a fix there is a change
to that repository's own front end, not something this record's territory reaches.
