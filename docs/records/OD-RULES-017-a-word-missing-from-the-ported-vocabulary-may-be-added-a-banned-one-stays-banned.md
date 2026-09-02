---
id: OD-RULES-017
type: decision
title: A word missing from the ported vocabulary may be added for this repository; a word it bans stays banned everywhere
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - naming
relations:
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-016
    type: relates-to
---

# A word missing from the ported vocabulary may be added for this repository; a word it bans stays banned everywhere

## Question

Composing `Check_Abbreviations` into a real run left 30 blocking findings, all in code this
workspace itself wrote, none of them a name-nobody-chose in the sense the rule's two existing
exemptions (a trait-fixed method name, a use-binding naming another crate's declaration)
already cover — every one is a freely-authored identifier that happens to contain a banned or
vowelless word because that word is what the code is about. Six shapes, by inspection:

- `CHK_003_SITE_FILES`/`Chk_003_Entry`/`Test_CHK_003_...` (3, `tests/contract/tests/
  requirement_trace/committed.rs`): quote `CHK-003`, a requirement id this repository already
  minted and cited by `docs/records/OD-CONTRACTS-002` and a committed `.assessment` file.
- `Check_Eager_Vs_Lazy_Context`/`EAGER_VS_LAZY_CONTEXT` and five derived test names (7,
  `error_text.rs`): quote `eager-vs-lazy-context`, code-standards' own rule id, ported
  verbatim — every rule id constant in this crate carries the doc comment "matching the
  code-standards rule id" for exactly this reason.
- `Check_No_Mod_Rs_Files`/`NO_MOD_RS_FILES`/`Is_Disallowed_Mod_Rs` and two derived test names
  (5, `structure.rs`): quote `no-mod-rs-files`, code-standards' own rule id, and the literal
  `mod.rs` filename the rule judges.
- `SENSITIVE_URL_PARAMS` and three derived test names (4, `security_text.rs`): `param`/
  `params` is banned, and spelling it out costs nothing — these are ordinary identifiers, not
  quotes of anything fixed.
- `Rc`/`RefCell` (`rust_text.rs`), `cmp::Ordering` (`concurrency_text.rs`), an env shebang
  (`script_discipline.rs`), `Lf`/`Utf8` (`nomos-spec-project/tests/projections/
  determinism.rs`), `duration_ms` (`agent_execution_outcome.rs`), `Nth`
  (`nomos-spec-store/src/submission/tests.rs`), and a `Nomos_Platform_Stds_Launcher` typo
  (`nomos-lang-rust-deny/tests/integration_seams.rs`) — 7, one file each: each names a Rust
  standard type/module, a POSIX convention, a line-ending term, an established `_ms`
  millisecond-suffix convention, an ordinary English ordinal, or misspells an already-approved
  word.
- `Has_Misplaced_Inner_Doc`/`Line_Is_Misplaced_Inner_Doc` (`nomos-lang-rust/tests/corpus/
  claims.rs`), `Test_Adr_Doc_001_...` (`nomos-spec-store/tests/governing_records_are_present/
  supersession.rs`), `Test_..._Docs_Own_Unmarked_Example` (`concurrency_text.rs`) — 4: `doc`/
  `docs` is banned, and each site names Rust's own "doc comment" term of art.

Whether the workspace's own `standards.json` — the channel `nomos.cap.words.policy` already
built for exactly this, holding `arc`/`std`/`repo` and 21 others beyond the ported list — may
answer some of these, and whether a banned word may ever be added through it, is the question
a mechanical extension of `OD-RULES-015`/`016`'s precedent does not answer, because none of
these thirty is a structurally-nested name the way a trait member or a use-binding is.

## What Was Measured

**None of the thirty is banned *and* missing.** Cross-referencing all thirty flagged words
against `DEFAULT_BANNED_WORDS` in `abbreviations.rs` splits them cleanly: `chk`, `vs`, `rs`,
`rc`, `cmp`, `lf`, `ms`, `nth` are absent from it — code-standards simply never listed them,
the same gap `std`/`repo`/`arc` already found and this repository's own `standards.json`
already closed for those three. `param`/`params`, `env`, `doc`/`docs` are on it by name.
`stds` is neither: it is not a word this rule's vocabulary judges at all, because it is a
misspelling of `std`, which is already approved.

**`standards.json` already overrides a banned word once.** `repo` is in `DEFAULT_BANNED_WORDS`
and also in `standards.json`'s own `words.approved_abbreviations` — this repository is called
`nomos` and reasons about repositories constantly, and the override already stands, tested,
and has not turned any of this workspace's other abbreviation findings into a `repo`-shaped
false negative. So the channel *can* carry an override; the open question was never mechanical
capability, it was whether these particular words are worth the same trade.

**A missing word costs nothing to add.** `arc`, `utf`, `ascii`, `min`, `max` are already
approved for the identical reason `rc`, `cmp`, `lf`, `ms` are missing: each names a Rust
standard type/module, a protocol acronym, or an established numeric-suffix convention, narrow
enough that a future vague use of the same two-or-three letters is not a realistic risk this
rule exists to catch.

**A banned word costs its ban everywhere it is added, not just at the cited site.** Unlike a
missing word, `param`/`env`/`doc` were excluded on purpose — presumably because each is also a
common, lazy shorthand for an ordinary English word ("parameter", "environment", "document")
the standard wants spelled out. Adding one to `standards.json` does not scope the exemption to
the finding that prompted it; it silences the ban for every future name in this workspace.
`param`/`params` costs nothing to spell out here, so the ban is not actually in tension with
these four sites and a rename settles it outright. `env` and `doc`/`docs` are different: the
sites that use them are not abbreviating "environment" or "document" at all — one quotes the
literal POSIX `env` command, the others Rust's own "doc comment" term of art — and spelling
either out (`Environment_Shebang`, `Documentation_Own_Unmarked_Example`) would misdescribe what
the code is about rather than merely lengthen it. Whether that is worth trading the ban's
protection for is a judgment this record declines to make on five isolated sites' evidence.

## The Decision

**A word absent from `DEFAULT_BANNED_WORDS` — missing from the ported vocabulary rather than
excluded from it — is added to `standards.json`'s `words.approved_abbreviations` when it names
a fixed external identifier (a code-standards rule id, or a requirement id this repository has
already minted and cited elsewhere) or an established, narrow technical term (a Rust standard
library type, module or protocol acronym, a numeric-suffix convention already established for
a sibling word).** `chk`, `vs`, `rs`, `rc`, `cmp`, `lf`, `ms`, `nth` qualify. Adding them is
this record's whole point but not this item's own act — this item's territory reserves only
the record, not `standards.json` — so a follow-up item, reserving that file, carries it out.

**A word `DEFAULT_BANNED_WORDS` names is not added through that channel merely because one
site's use is legitimate**, because the channel has no way to scope an addition to a site — it
silences the ban everywhere. Where the banned word can be spelled out with no loss of meaning,
the identifier is renamed instead: `SENSITIVE_URL_PARAMS` and its three derived test names are
`param`/`params`-shaped and cost nothing to spell out as `SENSITIVE_URL_PARAMETERS`/
`Parameter`, and `Nomos_Platform_Stds_Launcher` is a plain misspelling of the already-approved
`std`, corrected to `Std`. Both renames are named here for the same follow-up item.

**`env` and `doc`/`docs` are left open.** Both are banned, and at their five sites neither
abbreviates the English word the ban exists to catch — but both are common enough words that
approving them for this workspace risks quietly disabling the ban's real catches elsewhere,
and this record has evidence from five isolated sites, not a survey of that risk. A future
record narrowing the channel itself — an addition scoped to a site or a file rather than the
whole workspace — would resolve these without that cost; building it is not this record's job.

## What This Record Does Not Do

It does not touch `standards.json`, any of the thirty finding sites, or `Check_Abbreviations`
itself. This item's own territory reserves only the record; a follow-up item, correctly scoped
to the files this decision names, carries out the eight additions and the two renames.

It does not build a site-scoped or file-scoped override channel for `env`/`doc`/`docs`. It
declines to decide those five on the evidence at hand, which is not the same as deciding they
must stay reported forever — a future record with a real proposal for scoping is free to
revisit them.

It does not touch `DEFAULT_BANNED_WORDS` or `DEFAULT_APPROVED_WORDS` in `abbreviations.rs`.
Both are code-standards' own list, ported verbatim, and not this repository's to edit —
`standards.json` is the only channel this record uses or recommends.

## Status

Accepted. The reasoning splits all thirty findings across missing-vs-banned and, within
banned, spellable-vs-not: twenty are named for a follow-up item as `standards.json` additions
(`chk`, `vs`, `rs`, `rc`, `cmp`, `lf`, `ms`, `nth`, covering the `CHK_003`, `Vs`, `Rs` and
scattered-technical-term shapes save `env` and the `Stds` typo), five as renames (`param`/
`params`, `Stds`), and five (`env`, `doc`/`docs`) stay open, explicitly, with the reason
recorded above rather than left silently unequal to the other twenty-five.

Revisit if a future addition through `standards.json` turns out to mask a real abbreviation
this rule should have caught — that would be the missing-word trade this record makes turning
out to cost more than the `repo` precedent suggested, and the answer then is to remove the
addition and rename the sites that prompted it instead.
