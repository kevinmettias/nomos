---
name: nomos-add-plugin
description: Add a second (or later) language provider, syntax capability, package manifest crate, or rule to this workspace - which convention functions a provider crate exports, what a rule's function signature must be, where each gets composed in for real, and what territory to reserve. Use before scaffolding a new crate under crates/languages, crates/packages, or crates/rules, or before adding a second rule function to nomos-rules.
---

# Adding a language provider, package format, or rule

This is the mechanical recipe `OD-HOST-004` and the two real precedents in this workspace
(`nomos-lang-rust-scan` as a second provider, `nomos-package`/`OD-PACKAGE-007` as the
language-agnostic manifest split) already establish. It does not re-argue any of their
reasoning — read the cited record if you want the why. This skill exists so the four or
five mechanical steps do not have to be reverse-engineered from a git diff each time.

Follow `nomos-task`'s claim/implement/verify/finish/commit loop for the ledger item this
work needs; this skill only covers what that item's territory and `done_when` should
contain.

## 1. A second provider of a capability that already exists

The real precedent is `nomos-lang-rust-scan` (`crates/languages/nomos-lang-rust-scan`), the
second offer against `nomos.cap.syntax.items`.

A provider crate exports exactly four things, by convention — there is no `Provider`
trait, so nothing enforces this beyond the composition root failing to compile without it:

- `pub const PROVIDER: &str` — this provider's own `ProviderId` string, owned here and
  pulled by reference everywhere else it is needed (a package's `KNOWN_PROVIDERS`, a
  composition root's `Registry::Offer`) rather than retyped.
- `pub const fn Declared_Guarantee() -> Guarantee` — the four-axis promise
  (`FactVariant`, soundness `Assurance`, completeness `Assurance`,
  `IncrementalGranularity`) this provider's method actually supports. State it honestly on
  every axis your method is weaker on; `Assurance::Unsound` (established false) and
  `Assurance::Unknown` (nobody has established it) are not interchangeable —
  `crates/languages/nomos-lang-rust-scan/src/guarantee.rs`'s own tests are the worked
  example of proving a guarantee is neither vacuously satisfying nor vacuously failing.
- `pub fn Provider_Offer() -> ProviderOffer` — `{ provider: ProviderId::New(PROVIDER),
  capability: Capability(), version: CONTRACT_VERSION, guarantee: Declared_Guarantee() }`,
  where `Capability()` and `CONTRACT_VERSION` come from the capability contract crate
  (`nomos-cap-syntax` for syntax), never redeclared here — a capability contract is owned
  by the crate below every provider that offers against it, not by any one provider.
- `pub fn Materialize(...) -> MaterializedFact` (or whatever the capability's own fact
  shape is) — the actual work: turning a subject into a fact this provider's guarantee
  describes.

**Band**: the same band as the peer it contends with (25, for a second syntax provider) —
never a band that could let one provider name the other. `tests/contract/tests/boundaries/
graph.rs`'s downward-only rule forbids a same-band edge, which is what stops a second
provider's answer from being derived from the first's.

**Composition**: `Registry::Offer` (or `Declare_And_Offer` for a genuinely new capability —
see §3) is one more hand-written line in `crates/orchestration/nomos-check-orchestration/
src/composition.rs::Registered()`. `OD-HOST-004` already decided this needs no selection
mechanism at any count: the registry ranks, the caller states a `Requirement`, one more
`Offer` call is composition, not choice. Check `Registered()` as it stands before adding
your line — as of commit `fb46438`, both `nomos_lang_rust::Provider_Offer()` and
`nomos_lang_rust_scan::Provider_Offer()` reach the production composition root, so a third
offer follows an already-proven pattern rather than an untested one. Confirm this yourself
against `crates/orchestration/nomos-check-orchestration/src/composition.rs` rather than
trusting this sentence — it goes stale the moment a new provider lands and nobody re-reads
it, which is exactly how it went stale once already.

**Package registration**: if this provider is meant to be selectable through a
`LanguagePackage`-shaped manifest, its `KNOWN_PROVIDERS` array (`nomos-lang-package`'s
today) needs your `PROVIDER` constant added — see §4 for whether that means editing an
existing package crate or writing a new one.

**Reserve in the ledger item's territory**: the new provider crate's directory,
`Cargo.toml` (workspace members and `[workspace.dependencies]`), `README.md`'s band table,
`tests/contract/tests/boundaries/bands.rs`, the composition root file you edit, and
`tests/contract/surface` (a new snapshot for the crate — `NOMOS_SURFACE_BLESS=<crate>
cargo test -p nomos-contract-tests --test public_surface`).

## 2. A capability contract that does not exist yet

If the fact your provider produces is not shaped like any existing capability
(`nomos.cap.syntax.items`, `nomos.cap.module.surface` are the only two in this workspace
today), the contract itself needs a home first — its own crate, below every provider that
will offer against it, the way `nomos-cap-syntax` sits below `nomos-lang-rust` and
`nomos-lang-rust-scan`. Do not define a capability contract inside the first provider that
needs it: `crates/languages/nomos-lang-rust-scan/src/guarantee.rs`'s module doc is the
worked cautionary tale of what happens when a contract starts out living with one party to
it (`nomos-cap-syntax` was
extracted from `nomos-lang-rust` for exactly this reason, once a second provider needed to
agree with it independently). Pull the contract out at the moment a second party is
expected to offer against it, not only once one actually has.

## 3. A second rule

Much smaller than a provider. `crates/rules/nomos-rules` holds three rules today,
`Check_Completeness_Mirrors`, `Check_Naming_Convention` and `Check_Dependency_Direction` —
all `fn(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>` — a fourth rule
is a fourth function of that same shape inside the same crate, not a new crate or a new
band. There is no `Rule` trait; match the signature.

**Wiring it in**: `crates/orchestration/nomos-check-orchestration/src/run.rs`'s `Run`
function calls every rule it knows about unconditionally:

```rust
let mut findings = Check_Completeness_Mirrors(sources, &mut reader);
findings.extend(Check_Naming_Convention(sources, &mut reader));
```

`OD-HOST-004` decided `Run()` stays hand-written, and one more unconditional call beside the
existing ones is composition, not the accretion the record warns about — add your own
`findings.extend(Your_Rule(sources, &mut reader));` and fold the result into the
`CheckOutcome::Judged` `Run` returns. **This stops being true the moment your rule is meant
to run only for some invocations** (a per-language rule, an opt-in, a subset) — at that
point read `OD-HOST-004` in full before writing an `if`; it names the declared selection
mechanism that case needs instead.

**State your own floor.** `nomos_rules::Syntax_Requirement` is `Check_Completeness_Mirrors`'s
own stated `Requirement` against the capability it reads — not `Registered()`'s and not
`Run()`'s. Your rule states its own the same way; do not let the composition root decide
what your rule needs.

**Reserve in the ledger item's territory**: `crates/rules/nomos-rules` (your new function
and its own test module), `crates/orchestration/nomos-check-orchestration/src/run.rs` (the
new call site), and `tests/contract/surface/nomos-rules.txt` if the crate's public surface
grows a new export.

## 4. A second language's package manifest

Depend on `nomos-package` (band 24) directly — never on `nomos-lang-package`, which is the
Rust-specific wrapper, not a generic base a second language extends. `OD-PACKAGE-007` is
why the split exists and what it does and does not provide: `nomos_package::PackageManifest`
carries `language_versions: Vec<String>` as raw, unresolved labels (there is no typed
version-domain type to reuse — invent your own, the way `nomos-lang-package::RustEdition`
is Rust's own), and `nomos_package::Parse_Manifest`/`Read_Manifest` take a `known_providers:
&[&str]` parameter rather than a hardcoded list. Your package crate supplies that list (its
own providers' `PROVIDER` constants, pulled by reference the same way
`nomos-lang-package::KNOWN_PROVIDERS` pulls Rust's) and resolves each raw
`language_versions` label against whatever typed domain your language's own version scheme
actually needs, reusing `nomos-lang-package`'s `reader.rs::Resolved_Editions` as the shape
to follow rather than as code to call.

**Band**: above your language's own provider crates (so it can name them), the way
`nomos-lang-package` sits at 26, above `nomos-lang-rust`/`nomos-lang-rust-scan` at 25. It
must never need to be below `nomos-package` (24) or above it in a way that would create a
same-band or upward edge.

**Reserve in the ledger item's territory**: the new package crate's directory, `Cargo.toml`,
`README.md`, `tests/contract/tests/boundaries/bands.rs`, and `tests/contract/surface`.

## What this skill does not cover

`OD-PACKAGE-006` is accepted, not open: `nomos_package::KnownProviders`
(`crates/packages/nomos-package/src/known_providers.rs`) is the generic
provider-registration base type the resolution decided, and
`crates/packages/nomos-lang-package/src/known_providers.rs` is its first real consumer, building
`KNOWN_PROVIDERS` through `KnownProviders::New(...).As_Slice()` rather than as a bespoke
array of its own. §1's "Package registration" step and §4 both describe that shape as it
stands today.

What the resolution explicitly does not reach is `OD-PACKAGE-006`'s own original
trigger: a third same-language (Rust) provider crate joining this workspace has still not
happened, and whether `KnownProviders` needs anything beyond "one more line, pulled by
reference" at that point is a question the record's own text says this resolution does not
answer. Read `OD-PACKAGE-006` directly before assuming that question is settled — the
generic base type resolves how a package crate builds its allowlist, not what a third
same-language provider would demand of it.
