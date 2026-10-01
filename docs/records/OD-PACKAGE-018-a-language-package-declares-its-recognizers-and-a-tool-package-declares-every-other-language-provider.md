---
id: OD-PACKAGE-018
type: decision
title: A language package declares its recognizers, and a tool package declares every other language provider
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - package
  - capability
  - conformance
relations:
  - target: OD-PACKAGE-014
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
  - target: OD-CAPABILITY-013
    type: relates-to
  - target: OD-ROADMAP-006
    type: relates-to
---

# A language package declares its recognizers, and a tool package declares every other language provider

## Question

`OD-PACKAGE-014` decided a package's activation semantics are a conformance claim checked in both
directions: a manifest declaring a provider the registry never offers, and "a registered provider no
installed package claims", are each a finding. `nomos-check-orchestration`'s
`Check_Package_Conformance` is that check, and its test reads this workspace's three
`LanguagePackage` manifests against the registry a real run holds. It reports seven registered
providers that no manifest claims, and its doc calls them a gap a future item closes by adding them
to the manifests.

The Go and C# language packages say the opposite. Each one's `KNOWN_PROVIDERS` doc says a
`LanguagePackage` registers what recognizes the language and deliberately leaves out its ecosystem's
other providers. Each package's reader refuses a manifest naming a provider its list does not admit,
so the gap the test describes cannot be closed the way its doc says.

Both readings stand in the tree, and each new provider meets them. `P171` met them when it composed
the C# conditional-compilation provider. `P128-GO-HAS-A-PARSER-AND-A-MANIFEST-READER-AND-NO-TOOL-SPEAKS-FOR-IT`
requires the Go language package's list to gain the Go lint provider, which the second reading
forbids. Which providers does a language package declare, and what declares the rest?

## What Was Measured

All of it on 2026-09-28, at `2a42e117`.

**A manifest's provider list has exactly two readers, and neither composes anything.** Each package
crate's `Read_Manifest` admits a registration only if its `KNOWN_PROVIDERS` names it and refuses the
manifest otherwise. `Check_Package_Conformance` compares the admitted lists against
`Registry::Offers`. Grepped across every `.rs` file: no host, service or composition root reads a
manifest, and the conformance check's only caller is
`nomos-check-orchestration/tests/capability_composition.rs`. So the list's whole effect today is
which manifests parse and what the conformance claim reports. It changes no run, as `OD-PACKAGE-014`
required.

**Every provider the three language packages admit offers `nomos.cap.syntax.items`, and no other
provider is admitted.**

| Package | `KNOWN_PROVIDERS` | Every one offers |
|---|---|---|
| `nomos-lang-rust-package` | `nomos.lang.rust.syn`, `nomos.lang.rust.scan` | `nomos.cap.syntax.items` |
| `nomos-lang-go-package` | `nomos.lang.go.tree-sitter` | `nomos.cap.syntax.items` |
| `nomos-lang-csharp-package` | `nomos.lang.csharp.tree-sitter` | `nomos.cap.syntax.items` |

`nomos.lang.rust.syn` also offers `nomos.cap.controlflow.reachability`. A package declares a
provider identity, and every offer that identity makes comes with it.

**No record decided that scope.** The Rust crate's doc states no scope at all; its list simply holds
the two syntax providers. The Go crate's doc states the scope and cites the Rust list as its
precedent, and the C# crate's doc repeats the Go one. A precedent was read off a list's contents and
then written down as a rule, and this is the first record to hold the rule.

**The seven unclaimed providers, with the family `OD-CAPABILITY-013` classified each under, or would
classify each under by its own table's reasoning.**

| Provider | Offers | Family | Delivery |
|---|---|---|---|
| `nomos.lang.rust.cargo` | `nomos.cap.dependency.edges` | PACKAGE_MANAGER | external, `cargo metadata` |
| `nomos.lang.rust.clippy` | `nomos.cap.lint.diagnostics` | LINTER | external, `cargo clippy` |
| `nomos.lang.rust.deny` | `nomos.cap.dependency.policy` | PACKAGE_MANAGER | external, `cargo deny` |
| `nomos.lang.rust.compiler` | `nomos.cap.rust.copy_clones`, `nomos.cap.rust.nested_locks` | SEMANTIC_MODEL | in-process, `ra_ap_hir` |
| `nomos.lang.rust.complexity` | `nomos.cap.metric.complexity` | PARSER | in-process, `syn` |
| `nomos.lang.go.modules` | `nomos.cap.dependency.edges` | PACKAGE_MANAGER | in-process, reads `go.mod` |
| `nomos.lang.csharp.msbuild` | `nomos.cap.csharp.conditional_compilation` | SEMANTIC_MODEL | external, `dotnet msbuild` |

`OD-CAPABILITY-013`'s own table gives cargo, clippy, deny, go-modules and the compiler their
families. Complexity is PARSER for the reason its own guarantee is `Syntactic`: it is a
reading of the parse tree, the same kind of answer the syntax providers give. MSBuild is
SEMANTIC_MODEL for the reason the compiler is: it resolves what a build means for the code, here which
branches are compiled.

**A package kind whose manifest carries a family per provider already exists, and nothing writes
one.** `nomos-tool-package`, built by `P47-TOOLPROVIDER-HAS-NO-PACKAGE-2`, reads a
`PackageKind::ToolProvider` manifest whose every registration carries an `OD-CAPABILITY-013` family.
Its `KNOWN_PROVIDERS` admits clippy and deny, two of the seven. No manifest of that kind exists under
`packages/`, no crate depends on `nomos-tool-package`, and the conformance test reads no manifest of
its kind. So the conformance check reports clippy and deny as claimed by no installed package,
while a package kind built to claim them sits unused.

**The check's unclaimed direction is scoped by a name prefix, on a ground that fails for these
seven.** `Check_Package_Conformance` reports a registered provider as unclaimed only when its identity
starts `nomos.lang.`. Its doc gives the ground: the registry also offers `nomos.repo.*` policy
providers, a review connector and the requirement-trace provider, "and none of those belong to a
`LanguagePackage` manifest". That ground is sound for those, and it is exactly the ground on which
the seven also do not belong to a `LanguagePackage` manifest -- yet they carry the prefix. The prefix
names where a provider's crate lives, `crates/languages`, and not which package claims it.

**`OD-CAPABILITY-013` already refused the other split available.** One could put the providers that
launch an external tool in a `ToolProvider` package and leave the in-process ones with the language.
That record measured `nomos-lang-rust-cargo` against `nomos-lang-go-modules` -- the same family, the
same capability, and different delivery -- and found delivery "the axis that does not matter". A split
on delivery would separate the two most alike providers in the table.

## The Decision

**1. A `LanguagePackage` declares exactly the providers that recognize its language: the provider
identities that offer `nomos.cap.syntax.items` for it.** This is what all three lists already hold.
From this record on, it is a rule a list is checked against, not a precedent a doc infers. Every
other offer a recognizing identity makes comes with it, as `nomos.lang.rust.syn`'s reachability
offer does.

**2. Every other language provider is declared by a `ToolProvider` package, under its
`OD-CAPABILITY-013` family, whatever its delivery.** "Language provider" means a `nomos.lang.*`
identity, the population the conformance check already scopes to. This workspace ships one
`ToolProvider` manifest per language it serves:

- `packages/nomos.tool.rust.json`: cargo, clippy, deny, the compiler and complexity;
- `packages/nomos.tool.go.json`: go-modules;
- `packages/nomos.tool.csharp.json`: MSBuild.

`nomos-tool-package`'s `KNOWN_PROVIDERS` admits all seven. It takes each provider's own `PROVIDER`
constant, for `OD-PACKAGE-006`'s reason. It names each one's crate in `nomos-architecture.json`'s
same-zone permits, where its first two already stand.

**3. The conformance check reads every package manifest of both kinds.** A registered `nomos.lang.*`
provider is unclaimed when no manifest of either kind declares it. `OD-PACKAGE-014`'s claim is
unchanged. What changes is which manifests count as the installed packages it compares, and that
used to be one kind of two.

**4. So neither reading was simply wrong.** The conformance test was right that the seven were
unclaimed, and the language packages were right not to claim them. What was missing was the
manifests of the kind that does. A new language provider now costs one place: its `PROVIDER` in
exactly one package crate's `KNOWN_PROVIDERS` -- the language package's if it recognizes, the tool
package's otherwise -- plus one manifest line. A provider added to neither is a conformance finding,
by the test that reads the real manifests.

**5. `P128-GO`'s Go lint provider goes to `nomos-tool-package` and `packages/nomos.tool.go.json`, and
never to the Go language package.** That item's known-provider clause is re-authored to say so.

## What This Record Does Not Do

It does not wire the conformance check into a host. It remains reached only by its test, as
`OD-PACKAGE-014` left it, and where an installed target repository's manifests would be found is
still `D-091`'s installation half.

It does not reopen `OD-CAPABILITY-013`'s vocabulary or any family its table assigned. It assigns two
families that table did not reach, complexity and MSBuild, and says why each.

It does not make the family a routing key. A rule still names a capability and never a family or a
tool, as `OD-CAPABILITY-013` decided. The family classifies a registration and nothing more.

It does not decide how many `ToolProvider` packages a repository other than this one ships. One per
language is this workspace's choice, because each language's providers version and install together
here. The rule is only that the providers are declared by that kind.

## Status

Accepted, with its build in the same change:
`P176-THE-PACKAGE-CONFORMANCE-TEST-CALLS-SEVEN-PROVIDERS-A-GAP-AND-THREE-LANGUAGE-PACKAGES-CALL-THEM-A-DESIGN`.
