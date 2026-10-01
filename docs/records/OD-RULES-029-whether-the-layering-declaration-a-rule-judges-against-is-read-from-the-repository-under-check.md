---
id: OD-RULES-029
type: decision
title: Whether the layering declaration a rule judges against is read from the repository under check
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - rules
  - architecture
  - layering
relations:
  - target: OD-RULES-003
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-RULES-020
    type: relates-to
---

# Whether the layering declaration a rule judges against is read from the repository under check

## Question

`Check_Dependency_Direction` and `Check_Dependency_Completeness` judge every edge against
`ZONES` and `Permits` in `crates/rules/nomos-rules/src/checks/dependency/zones.rs`, where
`ZONES` is a Rust slice naming this workspace's own crates as string literals and `Permits`
is a hand-written match. The item that raised this asked whether that declaration becomes an
`OD-RULES-011` sixth family — a capability fact read from the repository under check — or
stays compiled, and said no record argues either way.

**A record does.** `OD-RULES-003` decided it, in those words: "a declared architecture is
expressed as data, not as a bespoke Rust test." So the question this record can honestly
answer is not the one it was asked. It is narrower and more useful: which seam that data
arrives through, how much of `OD-RULES-003`'s mechanism is already built, and what the one
missing piece is.

## What Was Measured

Measured 2026-09-13, against this workspace.

**The declaration, and the gap it leaves elsewhere.** `ZONES` holds 66 entries, one per
workspace member, as string literals. On any other repository no member matches, so
`violations::Violations_In` takes its early exit and returns nothing, while
`completeness::Violations_In` returns one `dependency-completeness` finding per member. A
foreign workspace therefore gets a finding per crate saying it declared nothing, and silence
on the layering question the rule exists to answer.

**Two of `OD-RULES-003`'s three prerequisites are built.** That record named three things
that "do not exist yet" and left the migration unowned. Today:

| `OD-RULES-003` named | today |
|---|---|
| a capability establishing the observed graph as a fact at a stated `Guarantee` | **built** — `nomos-cap-dependency`, at `FactVariant::SemanticallyResolved`, the exact level that record specified and argued for over `Syntactic` |
| a rule consuming both through `OD-RULES-001`'s seam | **built** — `Check_Dependency_Direction` reads `DependencyPayload` through the `FactReader` |
| a place to author a declared architecture as data rather than a Rust `const` table | **not built** — `ZONES` is still the `const` table |

So the mechanism is two-thirds complete, matching its own specification, and the missing third
is exactly what this item asked about.

**The second copy.** `standards.json` declares a `tiers` array: 29 groups, 50 paths, in the
`band-N` vocabulary `OD-RULES-020` retired for named zones. 16 of this workspace's 66 members
have no entry at all — `nomos-platform-xvpe`, `nomos-composer-std`, `nomos-lang-rust-compiler`,
`nomos-repo-policy`, the five `nomos-cap-*-policy` contracts, `nomos-connector-coderabbit`,
`nomos-cap-requirement-trace`, `nomos-tool-package`, `nomos-workspace-discovery`,
`nomos-correction-orchestration`, `nomos-agent-orchestration` and `nomos-lsp`. Grepped across
every `.rs`, `.py`, `.json`, `.yml`, `.toml` and `.sh` in the tree: **nothing reads it.** It
was authored by `P26-DEPENDENCY-TIER-POLICY` so that code-standards' own
`check-dependency-direction` could read this repository's band table.

**What a sixth family would cost.** Less than the item assumed. `OD-PACKAGE-015` collapsed
the five `OD-RULES-011` families from six crates into one, so a sixth is a module in
`nomos-repo-policy`, not a new crate, and `OD-RULES-019`'s `standards_document` already holds
the shared read step.

## The Decision

**The declaration is read from the repository under check — `OD-RULES-003` decided that and
this record does not re-decide it. It is not an `OD-RULES-011` sixth family. The two seams
carry different things, and that difference is the whole answer.**

### Why not a sixth family

An `OD-RULES-011` family is a repository's **policy parameters** — which words are banned, how
many parameters a function may take. It parameterizes a judgment the rule keeps making.

`OD-RULES-003`'s declared architecture is a **triple**: a finite set of named components, an
order over them, and named exceptions the order alone cannot express. `ZONES` is only the
first of the three. `Permits` is the other two.

Shipping the membership map as a sixth family and leaving `Permits` compiled would move
one-third of the triple and fix nothing for the repository it was moved for: a foreign
workspace would author `crate → zone` in *this* workspace's twelve-zone vocabulary and still
be judged by *this* workspace's permitted-edge matrix. `OD-RULES-003` made the order and the
exceptions part of the declaration precisely so a second party could hold a different one,
which is the seam `OD-CAPABILITY-002` drew and that record cites: an agreement does not belong
to the party enforcing it.

That is also why the vocabulary has to travel with the data rather than be assumed. A zone
lattice with reasons behind each cell — `OD-RULES-020`, `OD-RULES-028` and `OD-CAPABILITY-015`
each argue one — is this workspace's architecture. It is not every repository's, and a
declaration format that cannot express a different one has not externalized the declaration,
only moved it.

### What is actually blocked, and what is not

Not the fact side: `nomos-cap-dependency` establishes the observed graph, at the resolution
`OD-RULES-003` specified. Not the rule side: the comparison already runs through the reader.

What is missing is a format for the triple and a provider that reads it — the third
prerequisite, unowned since `OD-RULES-003` was accepted because that record deliberately
declined to schedule the migration. It is now the only thing between two built thirds and a
rule that answers for any repository.

### The `dependency-completeness` behaviour is a defect against an accepted record, not an open question

`OD-RULES-003` says what a repository declaring no architecture gets:
`Applicability::NotApplicable`, chosen over `MissingCapability` and `ConfigurationDisabled`
with a reason given for each. What it gets today is a `dependency-completeness` finding per
member.

Inside this workspace that is right — a member absent from a declaration this repository *did*
author has genuinely declared nothing. The defect is that the compiled table cannot tell that
case from the other one: **"this repository declared an architecture and this member is missing
from it" and "this repository declared no architecture at all" are indistinguishable when the
declaration is a `const` in the rule's own crate.** Externalizing the declaration is what makes
them distinguishable, and until then the rule cannot report `NotApplicable` honestly, because
it has no way to know that it should.

That is filed as its own item rather than argued further here.

### `standards.json`'s `tiers` array is a second authority, and it is to be deleted

Not kept as the declaration a provider would read, and not treated as code-standards' input
this workspace does not own — the file is this repository's own, and its five `OD-RULES-011`
families read from it.

It is deleted because every property that would make it the future declaration is false. It
carries one-third of the triple, in a vocabulary `OD-RULES-020` retired, 16 of 66 members
short, and read by nothing in this tree. Reviving it once the format exists would import that
16-member gap into the mechanism's first real consumer, and re-expressing it in zone
vocabulary is authoring it fresh rather than reusing it.

Deleting it is also what makes the reconciliation owned. A stale declaration nobody reads has
no failing test and no owner; the gap is invisible precisely because nothing depends on it.
Removing it means the next declaration is authored once, against the 66 members that exist, by
the item that builds the format — rather than inherited half-done from a table written for a
different tool.

## What This Record Does Not Do

It does not build the format, a contract crate, or a provider. It does not change
`nomos-rules`: `ZONES` and `Permits` stay exactly as they are until the third prerequisite
exists.

It does not edit `standards.json`. The deletion it decides is filed as its own item, because
this record holds only its own two files.

It does not reopen `OD-RULES-003`, `OD-RULES-011` or `OD-RULES-020`. It applies the first,
declines to stretch the second, and keeps the third's vocabulary.

It does not schedule the migration. `OD-RULES-003` declined to, deliberately, and nothing
measured here changes that — except that the cost is now two-thirds lower than when that
record declined it, which is worth knowing the next time somebody asks.

## Amendment: The `tiers` Array Has A Reader, Outside This Repository

Version 1 disposed of `standards.json`'s `tiers` array as "a second authority to delete", on
the measurement that nothing in this tree reads it. **The measurement was right and the test
was wrong.** Nothing in this tree reads it because the reader is not in this tree.

`code-standards` deserializes the array directly, at `kernel/config/limits/limits.go` line 183
— `Tiers []dependencyTier` with the JSON tag `tiers` — and that package's own documentation
states the contract it belongs to: "A workspace whose shape is a layering declares tiers",
which its `check-dependency-direction` then judges. That is exactly what
`P26-DEPENDENCY-TIER-POLICY` authored the array for, in those words.

Two **Done** items in this repository's own ledger have verified conditions depending on it:

| item | its verified condition |
|---|---|
| `P26-DEPENDENCY-TIER-POLICY` | "`standards.json` declares the README crate bands as dependency tiers and `check.exe dependency-direction` passes for the repository" |
| `P36-STANDARDS-JSON-SCHEMA-DRIFT` | "`check doctor .` reports no BROKEN line for `standards.json`, and the file still declares … `tiers` …" |

Deleting the array would have silently falsified a finished item's own stated condition, which
is a worse defect than the stale one version 1 set out to remove.

### The distinction version 1 missed

That version considered the disposition that actually fits — "code-standards' own input this
workspace does not own" — and rejected it, because "the file is this repository's own, and its
five `OD-RULES-011` families read from it."

**That conflates the file with the key.** `standards.json` is a shared configuration file: of
its twelve top-level keys, most are `code-standards`' and five are `nomos-repo-policy`'s. A
shared file's disposition is decided **per key, not per file**, and "does this repository own
the file" answers a different question from "does this repository own this key".

So the corrected disposition: **`tiers` is another tool's declared input, which this repository
holds and does not own.** It stays. That it is in the `band-N` vocabulary `OD-RULES-020`
retired, and 16 of 66 members short, are real observations about it — and they are that tool's
concern to act on, not a licence for this one to delete it.

### What that changes about the third prerequisite

Nothing about the decision above, and one thing about the migration.

Version 1 reasoned that reviving `tiers` when the format exists would import its 16-member gap
into the mechanism's first consumer. That still holds, and is now load-bearing for a second
reason: **the future declaration cannot re-use this key even if it wanted to**, because another
tool already reads it under its own schema, and two readers with different expectations of one
key is the shape this repository files records about. Whatever `OD-RULES-003`'s third
prerequisite authors, it authors somewhere `tiers` is not.

### Why this record made the mistake

It grepped this repository and concluded from silence. A shared file's other readers are
invisible to that method by construction — and the ledger already held the evidence, in two
Done items naming the tool and the check outright.

## Status

Accepted, version 2. The declaration is data read from the repository under check, per `OD-RULES-003`; it
is that record's triple and not an `OD-RULES-011` family, because a membership map without its
lattice externalizes nothing; two of the three prerequisites are built to specification and the
third is unowned; the foreign-repository behaviour contradicts `OD-RULES-003`'s `NotApplicable`
and is filed as a defect; and `standards.json`'s `tiers` array is **not** deleted — the amendment
above corrects that, naming the reader outside this repository and the two Done items whose
verified conditions depend on it. It is another tool's declared input, and the future
declaration is authored somewhere it is not.
