---
id: OD-RULES-018
type: decision
title: env, doc and docs are resolved by a descriptive rename, not a words-policy addition
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - naming
relations:
  - target: OD-RULES-017
    type: relates-to
---

# env, doc and docs are resolved by a descriptive rename, not a words-policy addition

## Question

`OD-RULES-017` resolved twenty-five of the thirty `Check_Abbreviations` findings it measured
and left five open at their five sites, reasoning that spelling `env`/`doc`/`docs` out —
its own examples were `Environment_Shebang` and `Documentation_Own_Unmarked_Example` — would
misdescribe what the code is about, since none of the five abbreviates "environment" or
"document" in the ordinary sense the ban exists to catch. It declined to decide them on that
evidence and named what would resolve them: a scoped-approval extension to
`nomos.cap.words.policy`, or a structural exemption in `Check_Abbreviations` itself.

This record takes a third path neither considered: a rename that describes what each site is
about differently, rather than substituting a synonym for the banned word.

## What Was Measured

Read against each of the five sites, `OD-RULES-017`'s objection holds against the literal
substitution it evaluated but not against every rename:

- `claims.rs`'s `Has_Misplaced_Inner_Doc`/`Line_Is_Misplaced_Inner_Doc` name what `//!`/`#![...]`
  is called. "Doc comment" and "documentation comment" are the same term — the Rust Reference
  uses both — so `Inner_Documentation_Comment` loses nothing a reader would have gotten from
  `Inner_Doc`.
- `supersession.rs`'s `Test_Adr_Doc_001_Should_Carry_An_Explicit_Supersession_Edge` names a
  fixed node id, `ADR-DOC-001`, but the id itself is what needs to stay precise — and it does,
  quoted verbatim four times in the test body's SQL and assertions. The function name only has
  to say what is tested, and the very next test in the same file already does this without the
  word: `Test_The_Superseded_Record_Should_Be_A_Visible_Placeholder`. The flagged test now reads
  `Test_The_Superseded_Record_Should_Carry_An_Explicit_Supersession_Edge`, matching a convention
  the file had already established rather than inventing one.
- `concurrency_text.rs`'s `..._Should_Reject_The_Docs_Own_Unmarked_Example` means "the rule's
  own doc comment's example" — `Its_Own` says exactly that without the word, since "its" already
  refers back to the rule under test named earlier in the same identifier.
- `script_discipline.rs`'s `..._Should_Accept_Env_Shebang` sits beside a sibling,
  `..._Should_Report_A_Hardcoded_Shebang`, that already contrasts the hardcoded and portable
  forms without naming either program. `..._Should_Accept_A_Portable_Shebang` completes that
  contrast; the assertion's own source fixture still spells `#!/usr/bin/env bash` exactly.

Every rename above changes an identifier that names the *relationship* the test or function
states, never a string literal, node id, or shebang line — the four places precision actually
lives are untouched.

## The Decision

**`env`, `doc` and `docs` are not added to `standards.json`.** `OD-RULES-017`'s reasoning for
declining that stands: both are common enough as lazy shorthand for "environment" and
"document" elsewhere that a workspace-wide approval risks masking a real abbreviation this rule
exists to catch, and neither this record nor `OD-RULES-017` has evidence beyond five isolated
sites to justify that trade.

**The five sites are renamed to describe what they are about instead of quoting the banned
word**, per the mapping above. `Check_Abbreviations` itself is unchanged, and so is
`DEFAULT_BANNED_WORDS` — this is not the structural exemption `OD-RULES-017` also named as a
live option, because no new mechanism was needed once the rename itself stopped being read as a
literal word substitution.

## What This Record Does Not Do

It does not build the scoped-approval channel `OD-RULES-017`'s "What This Record Does Not Do"
section named. That gap in `nomos.cap.words.policy` is real and still open; this record simply
found that these five findings did not need it.

It does not claim every future `env`/`doc`/`docs` finding will have an equally clean rename.
Some future site may genuinely have no way to describe itself without the word, in which case
`OD-RULES-017`'s two named options — scoped approval or a structural exemption — are still the
ones to reach for.

## Status

Accepted. All five sites are renamed, `cargo test -p nomos-lang-rust -p nomos-spec-store
-p nomos-rules` is green, and the workspace self-check reports zero blocking findings.
