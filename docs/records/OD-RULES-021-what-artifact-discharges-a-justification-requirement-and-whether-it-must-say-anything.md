---
id: OD-RULES-021
type: decision
title: What artifact discharges a justification requirement, and whether it must say anything
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
relations:
  - target: OD-RULES-014
    type: relates-to
---

# What artifact discharges a justification requirement, and whether it must say anything

## Question

`Check_Unsafe_Justification` was measured on 2026-09-05 against `aho-corasick` (32 files,
20,937 lines, a crate that documents its safety contracts carefully) and produced 219
blocking findings there. Checked against the source directly rather than counted:
`packed/ext.rs` line 8 declares an `unsafe fn` with a rustdoc `# Safety` section on the three
lines directly above it, and `automaton.rs` line 198 declares an `unsafe trait` with the
identical convention — both reported as unjustified. Separately, a bare `// SAFETY:` marker
with nothing after it satisfies the rule today, unlike its siblings. Neither half is fixable
by editing the matcher alone: both are the same question, what artifact discharges a
justification requirement, and whether it has to say anything — and the answer shapes at
least four rules that each ask for one.

## What Was Measured

**Nomos's own self-check has never exercised this rule against a real `unsafe` declaration.**
Fifty-seven crates in this workspace declare `#![forbid(unsafe_code)]`, and a direct search
of every real `.rs` file under `crates/` found exactly zero genuine `unsafe fn`, `unsafe
impl`, `unsafe trait` or `unsafe { }` construct — the only two files matching a text search
are `rust_text.rs` itself (this rule's own hand-written test fixtures) and
`nomos-lang-rust-scan/src/item_kind.rs` (a string-literal syntax table, not real unsafe code).
The defect this record answers was invisible to this workspace's own gate for the same
reason `P45-RULES-CALIBRATED-AGAINST-CODE-THEY-WERE-NOT-TUNED-ON` names: a rule tuned only
against fixtures and a codebase that forbids the construct it judges has never been checked
against a real population of it.

**`Has_Unsafe_Construct` treats a block and a declaration as one shape, and they are not.**
`crates/rules/nomos-rules/src/checks/rust_text.rs` matches `unsafe {`, `unsafe fn`, `unsafe
impl` and `unsafe trait` identically, then checks all four for the same artifact:
`Comment_Has_Safety_Reason`, which requires a `//`-prefixed line (checked via `Comment_Text_
Of`, which also accepts `///` and `//!` as comment markers generically) whose trimmed text
`starts_with("safety:")`. A rustdoc `# Safety` heading — the convention `rustc` and `clippy::
missing_safety_doc` already ask for on an `unsafe fn` — is a markdown heading, `# Safety`, not
a line starting with the literal text `safety:`. It is scanned as a candidate comment line by
the same `Previous_Comment_Block_Has` walk that would find a `// SAFETY:` line, and still
fails the check, because the predicate it is tested against looks for the wrong prefix. An
`unsafe {}` block has no declaration site of its own to carry a doc section; a `#[must_use]`-
style item declaration does, and `unsafe fn`/`unsafe trait`/`unsafe impl` are declarations.

**A third artifact shape already exists in this workspace, correctly, and is not the one
`unsafe` needs.** `Check_Suppression_Directives_Carry_A_Reason` (`go_text.rs`) judges Go's
`//nolint`/`//nolint:linter` directive, checked via `Nolint_Has_Reason` against trailing text
on the identical line as the directive — a same-line marker, because a suppression directive
in Go carries no separate declaration to attach a doc section to and is not naturally split
across two lines the way a preceding comment is. This shape is already right for what it
judges and is not proposed as a substitute for either of the other two.

**The vacuous-marker gap is real and is not shared by `unsafe`'s own siblings.** `Has_Local_
Allow_Justification`, `Has_Local_Inline_Always_Justification` and `Has_Local_Ignore_
Justification` all check `Comment_Is_Non_Empty` — real, non-whitespace text beyond the
comment marker. `Comment_Has_Safety_Reason` checks only a fixed prefix, so `// SAFETY:` alone,
or `// SAFETY: TODO`, satisfies it today. `allow`, `inline-always` and Go's `nolint` never had
this gap; `unsafe` is the one rule among its siblings that accepts a marker saying nothing.

## The Decision

**Three artifact shapes, matched to what the construct actually has to attach to, not one
matcher applied uniformly:**

1. **Adjacent comment** — a `//`-prefixed line, on the same line as the construct or in the
   immediately preceding comment block — for a construct with no declaration site of its own
   to document: `unsafe { }` blocks, `#[inline(always)]`, `#[allow(...)]`/`#![allow(...)]`,
   and a bare `#[ignore]`. Unchanged from what `allow`, `inline-always` and the disabled-test
   rule already do correctly.
2. **A rustdoc `# Safety` section** — a markdown heading, not a line-comment prefix — for a
   construct that is itself a declaration with its own doc-comment site: `unsafe fn`, `unsafe
   trait`, `unsafe impl`. `Check_Unsafe_Justification` needs a second detector recognizing
   this heading, checked in place of (not in addition to, for these three constructs) the
   `// SAFETY:` line-comment check that stays correct for a bare `unsafe { }` block.
3. **Same-line trailing text** — text following the directive on its own line, no adjacent
   comment involved — for a directive that is inherently one line and carries no separate
   declaration: Go's `//nolint`/`//nolint:linter`. Already correct; not touched.

**A justification must carry real text beyond its marker, in every one of the three shapes,
with no exception.** `Check_Unsafe_Justification`'s own `Comment_Has_Safety_Reason` is the
one place this workspace accepts a marker saying nothing; every sibling rule already refuses
one, and this decision closes the gap by making `unsafe` match its siblings rather than by
loosening any of them.

**Applied by name to the four rules named:**

- `Check_Unsafe_Justification`: needs both fixes. Its `unsafe {}` block case keeps the
  adjacent-comment shape, corrected to require real text after `safety:` (shape 1, vacuous
  gap closed). Its `unsafe fn`/`unsafe trait`/`unsafe impl` cases move to shape 2, a real
  `# Safety` heading with real text following it, checked in the construct's own preceding
  doc-comment block rather than a `//` comment.
- `Check_Inline_Always_Justification`: no change. `#[inline(always)]` is always an attribute
  with no declaration site of its own distinct from what it decorates; shape 1 already fits,
  and its justification is already non-vacuous.
- `Check_Every_Allow_Carries_A_Justification`: no change, for the identical reason.
- `Check_Suppression_Directives_Carry_A_Reason`: no change. Shape 3 already fits Go's
  `//nolint` convention and is already non-vacuous.

## What This Record Does Not Do

**No rule code moves here.** `Check_Unsafe_Justification`'s new `# Safety`-heading detector,
its retained (and corrected) block-level `// SAFETY:` detector, and the test fixtures proving
both — including a real case shaped like `aho-corasick`'s own `packed/ext.rs` — are a
follow-up item's own territory: `crates/rules/nomos-rules/src/checks/rust_text.rs` alone, no
new capability and no new fact, since this is the identical text-scanning shape every rule in
that file already uses.

It does not touch `Check_A_Test_Does_Not_Retry_Until_Green`, `Check_Panics_Are_Justified_
Documented_And_Validated` or `Check_Shared_Interior_Mutability_Says_Why`. Those were not
named in the item this record answers, and each judges a construct (a retried assertion, a
panic macro, an `Rc<RefCell<_>>`) with no declaration-vs-block duality of its own to
reconsider — they are not silently assumed correct, they are simply outside this record's
own scope.

It does not decide anything about a doc-comment convention for a language other than Rust.
Go's `//nolint` is already shape 3 and untouched; no other language in this workspace has an
`unsafe`-shaped construct today.

## Status

Accepted. Three artifact shapes — an adjacent comment, a rustdoc `# Safety` section, and
same-line trailing text — matched to whether a construct has its own declaration site to
document, not one shape applied to all four. Every shape must carry real text beyond its
marker; `Check_Unsafe_Justification`'s block case already can be fixed to require this, and
its declaration cases need the new rustdoc-heading detector this record names but does not
build. `inline-always`, `allow` and Go's `nolint` are already correct and unchanged.
