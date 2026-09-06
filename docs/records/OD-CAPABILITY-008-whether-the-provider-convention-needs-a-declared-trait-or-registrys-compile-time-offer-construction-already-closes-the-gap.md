---
id: OD-CAPABILITY-008
type: decision
title: Whether the provider convention needs a declared trait, or Registry's compile-time offer construction already closes the gap nomos-proto's structural typing opened
status: open
version: 3
authority: canonical-normative-record
tags:
  - capability
  - packages
  - architecture
relations:
  - target: OD-CAPABILITY-001
    type: relates-to
  - target: OD-CAPABILITY-007
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
---

# Whether the provider convention needs a declared trait, or Registry's compile-time offer construction already closes the gap nomos-proto's structural typing opened

## Question

`code-standards`/`nomos-proto`, the Go-era predecessor this workspace rebuilds, once let a
check-local package hand-roll its own copy of the Rust language type. Go's structural typing
let the copy silently satisfy less of the real interface than it meant to; the resolver fell
back to a weaker default with no build error, and the check quietly stopped being enforced in
every shipped binary while its own tests, which linked only the kernel, kept passing. The fix
was manual: a file of nothing but `var _ analyzer.Interface = RustLanguage{}` compile-time
assertions, hand-added per surface, to recover a guarantee Go's structural typing does not
give for free.

`crates/languages/nomos-lang-rust/src/provider.rs` and
`crates/languages/nomos-lang-rust-scan/src/provider.rs` both implement the identical
four-part shape today — a `PROVIDER` identity constant, `Declared_Guarantee()`, `Materialize`,
and an identically-shaped `FactContext` struct each redeclares — as free functions and
constants, by convention, with no shared trait naming the shape. Two real, live
implementations of the same convention exist, so a trait covering it would not be designed
from zero instances the way `OD-PACKAGE-006` refused to design `KNOWN_PROVIDERS` from a
population of two.

The open question: does formalizing `PROVIDER`/`Declared_Guarantee`/`Materialize` as a real
trait close a Rust-shaped version of the Go bug above, or does
`crates/orchestration/nomos-check-orchestration/src/composition.rs`'s existing shape —
`registry.Offer(nomos_lang_rust::Provider_Offer())`, calling each provider's function by its
real, statically-resolved name — already make the bug unreachable here for a reason specific
to Rust, so a trait would add a vocabulary with nothing left for it to prevent?

## Current Position

The Go bug's mechanism was structural: a type can satisfy an interface it was never declared
against, so a second, drifted implementation can silently *replace* the real one at a call
site that never named either by type. Rust has no structural typing of that kind. `Registered()`
in `composition.rs` calls `nomos_lang_rust::Provider_Offer()` and
`nomos_lang_rust_scan::Provider_Offer()` by their real paths — a rename, a signature change,
or a dropped function is a compile error at that call site, not a silent fallback to a weaker
default nobody asked for. `OD-CAPABILITY-001`'s registry ranks by `Guarantee` among offers that
were already, individually, constructed by code the composition root explicitly calls; nothing
there resembles Go's runtime interface satisfaction, where the resolver chose among candidates
the source never named together.

That argues the bug class is already closed here, for a reason this workspace already chose
independently before either provider existed: the composition root is a single, explicit,
hand-maintained list, the same shape a companion research pass separately found Go's own
blank-import registry converged on and called weaker than Rust's, because a missing
registration here is a missing function call a reviewer can see rather than a missing blank
import whose absence is invisible until the check silently stops running.

What a trait would add instead, if anything, is not safety against silent drift — the four
functions already cannot drift silently, being named and typed at their one call site — but a
single place stating the shape the convention requires. Today nothing except reading both
`provider.rs` files side by side confirms `Materialize`'s signature, `Declared_Guarantee`'s
return type, and `PROVIDER`'s type actually agree between the two implementations, or would
catch a *third* provider that got one of the four subtly wrong in a way that still compiles on
its own (for instance a `Materialize` taking its context by a different shape than the other
two share) — the two existing providers were checked to match by inspection this session, not
by anything the compiler enforces across them.

A trait would also interact with, and should not be read as license to remove, this crate's
already-reasoned decision to duplicate `Encode_Payload` deliberately —
`nomos-lang-rust/src/provider.rs`: "a shared writer would make two providers of one capability
agree by construction and prove nothing"; `nomos-lang-rust-scan/src/provider.rs`: "the
duplication is the interface." Whether a trait over `Materialize`/`Declared_Guarantee` (the
registration surface) can be added without touching `Encode_Payload` (the payload-construction
surface those comments defend) is itself unverified — it looks separable on inspection, since
`Encode_Payload` sits outside the convention's four parts, but this record does not assume the
boundary holds. A design pass would need to confirm the trait's method set does not end up
pulling `Encode_Payload` along with it, which would then be reopening a duplication this
workspace already reasoned about and kept on purpose.

## A Third Provider Joins: What It Reveals

`crates/languages/nomos-lang-rust-cargo` was built and composed into `Registered()` after this
record was written (`P13-DEPENDENCY-EDGES-2`, `P13-DEPENDENCY-WIRE-1`), and this record's own
named trigger — "a third provider joins" — has fired in the literal sense. What it reveals is
not what "the trigger fires, so build the trait" would suggest.

Checked field by field against the real code, not assumed: `PROVIDER` (`&str`),
`Declared_Guarantee() -> Guarantee` (identical `pub const fn` signature, verified across all
three `guarantee.rs` files), and `FactContext` (the identical four fields — `snapshot`,
`variant`, `configuration`, `generation`, the same types — verified across all three
`provider.rs` files) agree exactly across all three providers now. A trait over just those
three parts would be sound today and would now have three real instances to check a shape
against instead of two, closing part of what this record's "Current Position" section named
as the actual gap — nothing except reading files side by side confirms agreement.

`Materialize` does not agree, and this record's own speculation about what a third provider
might reveal — "a `Materialize` taking its context by a different shape than the other two
share" — is exactly what happened, in a larger way than that sentence anticipated. Even
between the first two providers, `Materialize`'s return type already varied with each
provider's own fallibility (`nomos_lang_rust::Materialize` returns `Materialization`, an enum
of `Materialized`/`Unparseable`, because syntax parsing can fail per file;
`nomos_lang_rust_scan::Materialize` returns `MaterializedFact` directly, because an
approximate scan cannot). `nomos_lang_rust_cargo::Materialize_Workspace` diverges further, by
design rather than by drift: it takes `root: &Path` instead of `subject: SubjectId, source:
&str` — no `SubjectId` exists yet when it is called, because it discovers every workspace
member's subject itself — and it returns `Result<Vec<PackageFact>, MetadataError>`, one call
producing many facts, rather than the one-call-one-fact shape the first two share. This is not
a bug: `nomos_lang_rust_cargo` is a whole-workspace capability (Cargo resolves a manifest
graph, not a single file), where the first two are per-file capabilities called once per
`SubjectId` a caller already holds. The difference in `Materialize`'s shape tracks a genuine
difference in what each provider's capability is *of*, not a subtly-wrong copy of a shape that
should have matched.

## What Would Decide It

A third provider was named as the natural trigger, the same role it plays in
`OD-PACKAGE-006`: at that point the question of whether the convention's four parts genuinely
agree stops being answerable by reading two files side by side, and a shape mismatch a trait
would have caught at the second provider becomes one only a trait catches at the third. The
third provider has now answered that question directly, for three of the four parts, in the
affirmative — `PROVIDER`, `Declared_Guarantee` and `FactContext` agree across all three, and a
trait spanning exactly those three has real, checked agreement to be built against rather than
an assumption. `Materialize` answers it in the negative: the third instance's divergence is
principled, tracking a real difference in capability granularity, so a single trait method
covering all three `Materialize`s would have to be shaped generically enough to admit both "one
fact from one named subject" and "many facts from one discovered root" — which is not "the
convention's shape" so much as a new, more general shape invented to paper over two capability
kinds that are not actually the same kind. Building that trait now would repeat the mistake
this record already named for `Encode_Payload`: making two providers of different things agree
by construction, which proves nothing and forecloses a third capability kind (a fourth
provider, whole-repository rather than whole-workspace or per-file) from needing its own
`Materialize` shape without the trait's signature growing again to admit it.

A second, independent trigger from the original record is unchanged by this: if a consumer is
ever built that needs to hold providers polymorphically — iterating over an unknown-length
list of them, rather than the composition root's current shape of naming each by import —
Rust's static call-site checking stops applying for that consumer specifically, and something
closer to Go's dynamic dispatch reappears by design. That consumer does not exist yet.

## Both Named Triggers Have Fired, Checked Directly Rather Than Assumed

**The whole-repository class this record's own "what would decide it" section named as the
next test now has six real members, not the fourth this record speculated about.** Every
`pub fn Materialize_Workspace` in the workspace was read, not sampled: `nomos-repo-policy`'s
five domains (`goals`, `limits`, `naming`, `scripting`, `words`) and `nomos-lang-rust-deny`
all carry the identical shape `Materialize_Workspace<Port>(root: &Path, context:
FactContext, port: &Port) -> Result<PolicyFact, ErrorType>` — one call, one `PolicyFact`,
differing only in `Port` (`FileSystem` for the five repo-policy domains, `ProcessLauncher`
for `nomos-lang-rust-deny`) and the error type. This is not the "third, genuinely distinct
capability granularity" the Status section below asked a fourth provider to test: it is six
further instances of the *same* one-call-one-fact granularity `nomos-lang-rust-deny` already
established alone, now checked against five more real bodies rather than inferred from one.
The other whole-workspace shape this record did not separately name — `nomos-lang-rust-
cargo`, `nomos-lang-rust-clippy` and `nomos-lang-go-modules`, each `Result<Vec<Fact>,
Error>`, one call producing many facts — stays the genuinely different granularity this
record already reasoned would not unify with the per-file shape, untouched by this finding.

Six real, checked instances of one exact shape license a second, narrower trait this record
did not previously have grounds to name: `PROVIDER`/`Declared_Guarantee`/`FactContext`
across all providers, *and* `Materialize_Workspace` specifically within the `Result<PolicyFact,
Error>` family, generic over an injected port type and an associated error type. Unifying
`Materialize` across *that* six-member family would not repeat the mistake this record named
for the wider population — cargo, clippy and go-modules stay outside it, because their
return shape is a different capability kind, not a subtly-drifted copy of this one.

**The second trigger — a consumer holding providers polymorphically — has not fired, and the
item that would fire it needs its own correction first, not this record's.** `P41-RUN-
PLANNER`, named here as the consumer, is `stranded`: its `depends_on` still names
`P41-RULE-DECLARES-ITS-REQUIREMENTS`, which was declined, while the capability that
prerequisite was for was separately delivered under `P41-RULE-DECLARES-ITS-REQUIREMENTS-5`,
now `Done`. `P41-RUN-PLANNER` itself was never built and its stale dependency link is why —
not a design question this record answers, and not evidence the polymorphic-consumer trigger
has fired. A claimant of `P41-RUN-PLANNER` needs to know two things this record states
plainly rather than leaves to be re-derived: its own `depends_on` entry is stale and should
be re-pointed at (or re-verified against) `P41-RULE-DECLARES-ITS-REQUIREMENTS-5` before the
item is claimable again, and once unblocked, that consumer is the second trigger this record
has been waiting on — building it is what would make the polymorphic-provider question real
rather than hypothetical.

## Status

Open, both named triggers addressed rather than left standing. The whole-repository class
has fired and decides, not merely narrows, the first half: `PROVIDER`/`Declared_Guarantee`/
`FactContext` across every provider, plus `Materialize_Workspace` within the six-member
`Result<PolicyFact, Error>` family specifically, are each buildable as a trait on real,
checked agreement — build either when a caller needs it enforced rather than inspected, the
same "wait for a need, not a wish" discipline `D-135` already names elsewhere, since nothing
today consumes providers in a way inspection does not already cover. The wider `Materialize`
unification this record originally worried about stays declined: cargo, clippy and
go-modules' `Result<Vec<Fact>, Error>` shape is a different capability kind, not a fourth
granularity needing the trait to grow again. The second trigger has not fired — `P41-RUN-
PLANNER` does not exist as a built consumer, is `stranded` on a stale dependency reference to
a since-declined item rather than blocked on a real design question, and is named here as
exactly what a claimant needs to know before that trigger can ever fire for real.
