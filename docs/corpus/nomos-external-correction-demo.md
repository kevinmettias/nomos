# External-source correction through MCP and real Cargo validation

Measured 2026-10-04 using the complete published `cfg-if` 1.0.4 source package,
independently authored at https://github.com/rust-lang/cfg-if. Its packaged VCS
metadata names commit `3510ca6abea34cbbc702509a4e50ea9709925eda`.
The local source population has 15 files totaling 25,728 bytes, including its
original manifests, tests, documentation and licenses. The upstream library
SHA-256 is `c09723e0890d15810374009e96b20bf0eb2f65f383006516f34db36240835c85`.
No upstream source was trimmed or vendored into Nomos.

This is a complete published package source population, not its full Git history
or a claim about every file excluded from the package by its publisher. The
acceptance harness copies every source file, refuses links, and omits only VCS
state and build products. It works solely in unique temporary copies and checks
that the upstream library bytes remain identical afterward.

## Reproduce

From Nomos, with a complete independently authored Rust package available locally:

```powershell
$env:NOMOS_RELEASE_EXTERNAL_REPO = 'C:\Users\kmett\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\cfg-if-1.0.4'
cargo test --locked --offline -p nomos-mcp --test release_workflow -- --include-ignored --nocapture --test-threads=1
```

The external test is ignored in ordinary runs because it needs an explicitly
supplied corpus. Including it without a readable, nonempty corpus is a failure.
The command also runs the existing real-pipe MCP workflow and malformed-request
regressions. It uses the built MCP binary, actual stdin/stdout, real Cargo
processes, no credentials and no network. Each compiler process is bounded at
120 seconds and each MCP answer at 30 seconds. Temporary target directories avoid
nested Cargo target locks. The harness uses the existing platform launcher through test-only dependencies, including its timeout and process-tree cleanup contract; it adds no shipped process mechanism.

## A short narrated demonstration

1. Inspect the copied source and judge it through MCP: the selected mirror rule
   has no blocking finding in the untouched package.
2. Add a deliberately false enforcement declaration to the actual library file.
   Cargo still passes all four upstream tests (two unit, one integration and one
   doctest). Nomos now identifies the missing check, its source location,
   applicability and derived evidence. Compiler validation alone misses this
   engineering defect.
3. Ask MCP for its default correction. It stages a preview and leaves the active
   source unchanged. Take the proposed after-bytes from that actual preview and
   verify their exact before-bytes against the observed library.
4. In an independent verification tree, deliberately introduce a Rust syntax
   error. Cargo fails with exit 101 and compiler diagnostics. Withhold commit;
   the active source and its Nomos finding remain unchanged.
5. Put the exact proposed bytes in a clean verification tree. All four tests pass.
   Only then send explicit commit through MCP. The committed bytes match the
   externally validated proposal, the workspace snapshot changes, the blocking
   mirror finding disappears, and all four tests pass again.

The first successful measured run took **4,557 ms** for this external conversation,
excluding Nomos's own compilation. Its real Cargo observations were 1,359 ms for
the imperfect baseline, 325 ms for the expected compiler failure, 1,440 ms for
the valid staged tree, and 1,180 ms after committing. These are individual local
observations, not a performance benchmark or a speed claim for larger repositories.

The controlled worked comparison is specific: compiler/tests alone pass while
one false enforcement claim remains; the MCP client finds and removes that
claim, with the same upstream tests passing afterward. This establishes a useful
agent operation, not a general productivity effect or a trial against human
engineers. Narrating the five decisions above gives a two-to-five-minute demo;
the automated execution itself is much shorter.

A separate real-package negative control compiles successfully but executes zero
tests. The validation acceptance check refuses it, preventing a vacuous build
from passing as relevant test evidence. A raw-output test covers the same rule.

## Limits

The defect is deliberately injected into the independent source package; it is
not an allegation that upstream shipped that false claim. This demonstration
selects one policy rule and one existing mechanical correction family. It does
not certify the release's complete useful-rule set, an architectural dependency
refactor, or large-corpus performance.

Compiler validation is orchestrated by the client over an independent copy.
Nomos's model stage/validate operations do not themselves run those tests or
consume the validation result, and the explicit commit re-derives the correction.
The harness confirms exact byte agreement, but the engine still lacks a portable
external-validation receipt tied to a frozen candidate and atomic concurrent
compare-and-replace. Those limits remain part of the 0.1 acceptance work.
