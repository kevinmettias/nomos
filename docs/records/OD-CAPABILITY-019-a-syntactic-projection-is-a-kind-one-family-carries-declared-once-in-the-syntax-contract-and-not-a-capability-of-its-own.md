---
id: OD-CAPABILITY-019
type: decision
title: A syntactic projection is a kind one family carries, declared once in the syntax contract, and not a capability of its own
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - capability
  - syntax
  - rules
  - measurement
relations:
  - target: ARC-CONFORMANCE-003
    type: relates-to
  - target: OD-CAPABILITY-011
    type: relates-to
  - target: OD-CAPABILITY-006
    type: relates-to
  - target: OD-RULES-034
    type: relates-to
  - target: OD-RULES-035
    type: relates-to
---

# A syntactic projection is a kind one family carries, declared once in the syntax contract, and not a capability of its own

## Question

`ARC-CONFORMANCE-003` gave its third milestone as "a lexical rule is a declaration:
`DeclaredTextRule` carries the detectors, parameters, citation and text surface the corpus's
95 text engines need". Its tasks were to be authored when it opened, with territory grepped
then. Opening it meant measuring what those 95 engines need from the form, and that
measurement found the population the milestone was written for does not exist in the shape the
row assumed.

The question is therefore not what `DeclaredTextRule` must grow. It is what a code-standards
check actually is once it is read down to the point where it touches source, and what porting
one costs this workspace.

## What Was Measured

Every figure below was taken from code-standards at `f0d820729` and this workspace at the
revision this record lands on.

**A code-standards check is three layers, and extraction is the bottom one.** A check's
`main.go` is a thin declaration: 239 of the 320 hand the driver a judgment value, 188 of them
a `driver.FileJudgment` or a `driver.LanguageJudgment`, and `check-concurrency`'s is typical —
a tool name, a capability, the engine's analysis function and a report function. The shared engines under `rules/**/shared/` are the second layer, and 89 of
the 108 register a capability (`capabilities.Register_Capability`) and declare an analyzer
interface. The third layer is the language kernel, `language-kernels/programming/<lang>/<lang>lang`,
whose methods implement those interfaces. The engine then judges the typed records the kernel
returns: `presumption.Presumptions_For` resolves the file's language and asks it through the
interface, and `presumption.go` imports nothing but `nomos.dev/kernel`.

**So the "95 text engines" counted the judgment layer, and the judgment layer does not read
source.** `ARC-CONFORMANCE-003` counted 95 engines using `strings.` and zero importing
`go/ast`, and concluded the corpus was 88 per cent text. Both counts are accurate. The
`strings.` calls judge a record a kernel already extracted, such as whether a presumption's
reason is a placeholder, and no engine imports a kernel because every one reaches its kernel
through an interface registered at start-up. The quantity that decides the port is how the
kernel method extracts, and that record did not measure it.

**Method.** A grep cannot answer this, and trying to answer it with one was the first thing
this measurement established: counting tree references over each Rust-specific check's own
files and the packages it imports scored `borrowing`, `lifetime-discipline` and `proc-macro`,
all line scanners already ported here as text rules, at the same 1,607 as `allman-braces`,
which walks a tree, because every check imports its whole kernel. So each of the 108 engines
and each of the 135 language-specific checks outside `run-all` was read along its judgment's
own call path until it reached a parse or a line loop, for Rust, Go and C# separately where an
engine is answered per language. Eight readers took a batch each. Thirteen items were
controls whose answer had been established independently before their batch reported —
eleven before any batch was dispatched, two C# checks read while their batch was running —
and all thirteen came back as expected. Every classification carries the line of the parse call or loop, and all 358
of those lines were opened afterwards: 354 contain the construct their class names, and the
other four were read and confirmed.

**Engines.** Of 108:

- 73 extract through a syntax tree wherever they are answered, counting `confinedcalls`, which
  reuses `renderpath`'s extraction.
- 8 extract the same projection through a tree in one language and through lines in another:
  `concurrency`, `errortype`, `imports`, `integration`, `shippedtests`, `tuplereturn`,
  `unboundedloop` and `wrapping`.
- 12 read text only, and three of them read program source: `todo` scans comments line by
  line, `variantshape` scans Rust enums with brace depth, and `commandflags` matches Go flag
  definitions by pattern. Of the rest, `loopreason` is a comment look-back helper;
  `commanddocs`, `docref`, `featuredocs` and `standardsdoc` read documentation; `scripting`
  reads shell scripts; `promptquality` reads prompts; and `conformance` and `claimtruth` hand
  their text to a model.
- 9 read a manifest, a ledger or the file system: `bugledger`, `dependencies`, `flakeledger`,
  `fuzzledger`, `gatetier`, `goals`, `layout`, `mutationledger` and `symmetry`.
- 3 are answered only by kernels for other languages (`declarations`, `fieldimmutability`,
  `shape`), and 3 extract nothing at all (`naming`, `vocabulary`, `duplicationsettings`).

**Language-specific checks.** Of 135, 101 parse a tree, Dockerfiles, CSS, HTML, XAML and YAML
included; 11 relay a compiler or linter (Roslyn, `tsc`, `go vet`, `cargo check`); 12 judge
one line at a time; 4 carry state from line to line; and the remaining 7 split between
whole-text scans, a manifest reader, the file system and cross-file readers. Of the 17 in
those three text classes, this workspace already holds the rule identifiers of 8 in whole or
in part — seven Rust checks and C#'s trailing whitespace. The other 9 are five Razor checks,
one XAML check, a scan of `.golangci.yml`, a scan of `Cargo.toml`, and `move-out-of-mut`,
which cannot fire on an Allman-braced tree at all.

**The corpus keeps a projection apart from how it is extracted.** The 8 mixed engines above
are one projection each, answered by a tree in one kernel and by a line scan in another:
`wrapping`'s imports are parsed in Go and matched line by line in Rust and C#. What an engine
judges is the record, and the method belongs to the kernel.

**A projection costs this workspace a capability family.** `RequiredFact` names 19 families
and each is its own crate under `crates/capabilities/`. The metric fact
`P125-A-RULE-THAT-NEEDS-A-METRIC-FACT-3` added is the recent precedent: outside its two new
crates, which hold 28 files, it edited the root manifest, the lock file, `README.md`,
`nomos-architecture.json`, the composer, ten files of check orchestration, four surface
snapshots and five files of the determinism harness. The reachability fact cost 28 files
across 10 areas, and the two compiler-backed facts arrived as a crate each. At that price the
73 tree engines and 101 tree checks are the cost of this campaign, and the narrowness of
`DeclaredTextRule` is not.

**The syntax agreement already exists and every language provider already names it.**
`nomos-lang-rust`, `nomos-lang-go` and `nomos-lang-csharp` each depend on `nomos-cap-syntax`
and answer `nomos.cap.syntax.items`. `OD-CAPABILITY-011` kept that payload a flat index of
items with three closed extensions, refused a general typed tree, and declined walking into
function bodies as "real, separate, larger work", which is exactly where most of the corpus's
projections live: presumptions, float comparisons, nested calls, loops, conditions. The walking
itself is not new here: `nomos-lang-rust` already builds `syn` with `full` and `visit`, and the
complexity provider already visits expressions inside bodies, in a crate of its own.

**Re-deciding a tree check as a text rule has been the workaround, and it is lossy.** This
workspace holds the rule identifiers of 30 corpus checks whose Go original parses a tree, many
of them ported as text rules. Two of those re-decisions are measured: a text prototype of
`allman-brace-placement` agreed with the Go tool on fifteen hand-written probes and then
reported 13 false positives in 15 findings over `crates/`, in six structural classes a parser
separates and a scanner cannot, and `unsafe-justification`'s substring detector read a string
literal spelling `unsafe impl` as a declaration (`P66-UNSAFE-JUSTIFICATION-STRING-BLINDNESS`).
The owner asked that checks and repairs be reproduced the way code-standards makes them,
because they are lexical rather than semantic or architectural. Measured, that is true in the
sense that matters and false in the one `ARC-CONFORMANCE-003` assumed: code-standards makes
most of its checks from a syntax tree, not from text. Its Rust and C# kernels are tree-sitter
parsers, which resolve no name and no type; its Go kernel type-checks in 14 of its 91 analyzer
files, because Go's standard library makes that cheap; and eleven checks relay a compiler.
Syntax is the level a port has to reach, and for most of the corpus it is the whole of it.

## The Decision

1. **The corpus's port unit is a projection, and the third milestone is re-stated around it.**
   Of `ARC-CONFORMANCE-003`'s four port forms, a native function over a fact (P3) is the
   dominant one and a declared text rule (P2) is the minority, which reverses what that record
   concluded from the judgment layer. The milestone becomes: a syntactic projection is a
   declaration, and porting a corpus engine costs the projection's declaration, one extractor
   per language that offers it, and the rule. `DeclaredTextRule` stays exactly as
   `OD-RULES-034` decided it, and it grows a field when a rule needs one rather than because a
   milestone says so: what the corpus still has in its shape lies mostly in languages this
   workspace does not read.

2. **A projection is a kind, not a family.** One capability, `nomos.cap.syntax.sites`, carries
   every projection. Its payload is one record per located construct, and a record states its
   kind, its line and the fields its kind declares. This is `OD-RULES-035`'s first decision one
   layer down: that record made an axis a key in a row-shaped family rather than a family of
   its own, and this one makes a projection a kind in one family rather than a capability of
   its own.

3. **A kind is declared once, in the syntax contract.** Its name, its fields and the construct
   it projects are declared in `nomos-cap-syntax`, beside `nomos.cap.syntax.items`, because
   that crate is already the agreement every language provider names and a kind is an
   agreement between those providers and the rules that read them. No provider and no rule
   restates a kind's fields, and a payload record whose kind is not declared is a payload the
   family's reader refuses, not a record it passes on.

4. **A kind is flat.** A record holds a line and named values, each text, an integer, a truth
   value or a list of text, and it never holds another record. Of the 815 fields the structs
   in the 108 engines declare, 717 are already one of those; 43 hold a list of another struct
   — a signature's parameters, an enum's variants, a switch's arms — and such a list becomes a
   kind of its own whose records name the record they belong to. `OD-CAPABILITY-011`'s refusal
   of a general tree stands: the provider walks the tree and the payload carries what the walk
   found. Not every engine receives records today — `bindingscope`'s `Collect_Bindings` is
   generic over the kernel's node type and compares two scope stacks — and a scope stack is
   stated flat as a list of scope identifiers. A projection that cannot be stated that way is
   the observation that reopens this decision.

5. **A kind says what is projected, never how.** Each provider declares which kinds it offers.
   Whether it answers one through a parser or through lines is its own business and its own
   `Guarantee`'s, the way the corpus answers `wrapping` both ways, so `nomos-lang-rust-scan`
   may offer a kind it can honestly produce without a parser, under the fallback the registry
   already has.

6. **A kind a language does not offer is reported, never clean, and a kind it declines says
   why.** The payload for a file states which kinds its provider offers there and which it
   declines, so an offered kind with no records is a clean file and the other two are not. A
   provider declines a kind only with a stated reason that the construct does not exist in its
   language, and a rule reads that as `Applicability::NotApplicable`, the one variant that is a
   positive statement about an absent judgment. A kind neither offered nor declined is a gap,
   and a rule reads it as `Applicability::MissingCapability` for that subject, which is what
   `cross-language-correspondence` already does when the side it needs is absent
   (`OD-CAPABILITY-006`). The corpus draws exactly this line: run over this workspace,
   `check-presumption` reports 25 files whose language "declines this surface deliberately"
   and says why ("JSON has no null-forgiving assertion"), apart from 75 whose language "offers
   no such surface, AND HAS DECLARED NO REASON — this is an unwritten gap, not a decision",
   and its C# kernel declines `labeledloop` because "C# cannot name a loop".

7. **The bound.** The family is paid for once: the capability, one `RequiredFact` variant, its
   materialization in check orchestration, one offer and one fact production per provider that
   offers any kind — composed both in check orchestration's provider table and in
   `nomos-composer-providers`, which lists every offer again — one determinism golden per such
   provider, and the `README.md` rows that describe the syntax contract and those providers.
   After that, adding a kind edits `nomos-cap-syntax` and its
   surface snapshot, the providers that offer it, the rules that read it, and the golden of a
   provider whose golden fixture contains the construct — and nothing else. It adds no crate,
   no manifest, no lock entry, no `README.md` row, no `nomos-architecture.json` entry, no
   composition wiring and no `RequiredFact` variant. Offering an existing kind in another
   language edits that language's provider alone.

## What This Does Not Do

It builds nothing and authors no code change.

It does not decide a declared judgment over a kind. A rule over a kind is a Rust function, as
every rule over a fact is today. Whether the judgments share a shape a declaration could
state, the way forty-four text rules shared one for `OD-RULES-034`, is measured once enough
of them exist to count.

It does not move the complexity, clone-on-copy, nested-lock or reachability facts into the
family. Each is a projection-shaped capability that paid the family price, and whether any of
them should become a kind is a question for an item that holds its crate, measured against its
own consumers.

It does not decide demand below the family. A run that selects any rule reading a kind
materializes the family, so every offered kind is computed for every file of that language.
If that cost is ever measured to matter, which kinds to compute is that item's decision.

It does not reach the other milestones' populations. The 11 relays belong to M4, the 27
language-specific repairs and 64 inline markers to M5, and the languages this workspace does
not read to M7.

It does not amend `OD-CAPABILITY-011` or `OD-RULES-034`. Both stand as written:
`OD-CAPABILITY-011`'s items payload is unchanged, and the body descent it declined is taken up
here as a separate family rather than as a change to that payload.

## The Items That Build It

`P174-A-SYNTACTIC-PROJECTION-IS-A-KIND-ONE-FAMILY-CARRIES-2` lands this record.
`P175-ARC-CONFORMANCE-003-RE-STATES-M3-AROUND-OD-CAPABILITY-019` lands the amendment to
`ARC-CONFORMANCE-003` that withdraws its text-dominance conclusion and carries the new M3 row,
together with the citation version the standards-corpus rule keeps of that record; it is a
separate item only because the rules crate that citation lives in was reserved by another item
when this one landed. The build increments are authored when this record lands and their
territory is grepped then, in three steps: the family with its first kind,
offered in Rust and read by one rule, proven site for site against the corpus kernel's own
tests for that projection; the same kind offered in Go and C#, proven the same way; and a
second kind whose own territory is the bound in the seventh decision, so that a widening of
that item is the evidence the bound is wrong.

## Status

Accepted. Revisit if the second kind cannot be added inside the bound, or if a projection the
corpus needs cannot be stated as a flat record.
