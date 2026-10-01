---
id: OD-GATE-035
type: decision
title: Every package this workspace depends on is built at opt-level 2 in the dev and test profiles, and none of its own members is
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - gate
  - build
  - performance
relations:
  - target: OD-GATE-004
    type: relates-to
  - target: OD-GATE-025
    type: relates-to
  - target: OD-GATE-027
    type: relates-to
  - target: OD-RULES-010
    type: relates-to
  - target: OD-CAPABILITY-018
    type: relates-to
---

# Every package this workspace depends on is built at opt-level 2 in the dev and test profiles, and none of its own members is

## Question

A real gate run over this repository's root takes about four minutes, and nearly all of that
time goes to the two compiler-backed rule families, `copy-clones` and `nested-locks`. Each one
links `ra_ap_hir`, loads the resolved crate graph and a discovered sysroot in-process, and
answers from that database. `OD-RULES-010` is why they are facts a native rule judges.
`OD-CAPABILITY-018` is why nested-locks reads the same kind of loaded `Semantics`.

Every caller that pays this cost builds in the dev profile or the test profile, which inherits
from it. CI's Rules step is `cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .`,
and `OD-GATE-004` made that step one whose only passing exit code is zero. The suites that
judge this repository's own root are nomos-gate-orchestration's, nomos-check-orchestration's
and nomos-api's, and `OD-GATE-025` is why a real run judges the whole tree rather than a slice
of it. Before this record the dev profile set only `debug = 1`, so `ra_ap_hir` and everything
beneath it was compiled without optimization. The analysis these two families run is a
compiler frontend's name resolution and type inference, and that is a workload optimization
changes by a large factor.

The question is whether the workspace should build the packages it depends on with
optimization in the dev and test profiles, which packages that should cover, and what doing so
costs.

## What Was Measured

The item that boarded this record, `P173-A-REAL-GATE-RUN-SPENDS-FOUR-FIFTHS-OF-ITS-TIME-IN-A-COMPILER-FRONTEND-BUILT-UNOPTIMIZED`,
carried figures measured at `269411fa` by the session holding
`P146-THE-SEVENFOLD-SELF-CHECK-COST-IS-INSIDE-EVERY-PREDICATE-THAT-RUNS-A-REAL-GATE-AND-IT-HAS-ALREADY-SERIALIZED-THE-BOARD`.
They were the reason to look and are not evidence here. Everything below was retaken on
2026-09-27 at `d0581b4e`.

### Conditions

- **Machine.** An i9-13900K with 24 cores and 32 threads, 31.8 GB of memory, the repository and
  every target directory on one SATA SSD, and toolchain 1.88.0 as `rust-toolchain.toml` pins it.
- **Tree and binaries.** A private detached worktree at `d0581b4e`, with its target directory
  at the worktree's own default `target/`. The nomos binary was built fresh from that tree for
  each side: once from `HEAD`'s `Cargo.toml`, and once from the same tree carrying only the
  stanza this record adds. Each binary was copied outside the root before it ran.
- **Root.** Spelled as the worktree's own `.`, so it carries no `..` segments and nothing is
  filtered out as lying outside it.
- **Runs.** One at a time, each under a process profiler reading the kernel's own peak-commit
  counter, `PeakPagefileUsage`. Each side had one discarded warm-up run, so that `cargo clippy`,
  which the `lint-diagnostics` provider launches, was not re-checking dependencies inside a
  measured run.
- **Load.** Between 7.2 and 9.4 GB of memory was available throughout the gate runs. No other
  toolchain process ran during the gate runs, the cold binary and test builds, the narrower
  arm, or the incremental builds, apart from two idle rust-analyzer processes. The cold clippy
  series is the exception, and its paragraph names what ran beside it.

### Files were examined

Every full run reported the same two `copy-clones` findings, in
`crates/rules/nomos-rules/tests/integration_seams.rs`. `cyclomatic-complexity` reported
measuring 14,333 functions in 2,133 of 2,134 Rust sources. A run that examined nothing would
report neither, so a fast empty run cannot be mistaken for a fast answer here.

### A real gate run

`nomos gate run --root .`, three measured runs on each side:

| Dependencies built at | Runs (s) | Median (s) | Findings | Peak commit (MB) |
|---|---|---|---|---|
| opt-level 0, as before | 249.5, 248.4, 247.8 | 248.4 | 473, 2 of which can fail a build | 2,237, 2,238, 2,237 |
| opt-level 2, this record | 52.6, 52.7, 52.4 | 52.6 | 473, 2 of which can fail a build | 2,238, 2,236, 2,238 |

That is 4.7 times faster, at the same memory. The two findings that can fail a build are in
`crates/languages/nomos-lang-csharp-compiler`, an `abbreviations` finding and a
`lifetimes-follow-the-descriptive-naming-rule` finding. They are the same on both sides and are
`HEAD`'s, not this change's; every full run on both sides exited 1 because of them.

**The finding sets are identical, line for line.** Output lines were compared in order, not
counted. The only line that differs between any two runs is the leading `run:` identifier,
which is minted per run and differs between two runs on the same side just as much. Setting
that line aside, all six measured full runs and both warm-ups produced byte-identical output.

### Per family

The same binaries, selecting rules with `--rule`, three runs each:

| Selection | opt-level 0 runs (s) | opt-level 0 median | opt-level 2 runs (s) | opt-level 2 median | Findings | Peak commit (MB), 0 then 2 |
|---|---|---|---|---|---|---|
| `copy-clones` alone | 107.2, 106.6, 107.7 | 107.2 | 17.2, 17.3, 17.2 | 17.2 | 2 | 2,142 to 2,147, then 2,126 to 2,130 |
| `nested-locks` alone | 116.6, 116.1, 116.7 | 116.6 | 18.1, 18.4, 18.3 | 18.3 | 0 | 2,179 to 2,181, then 2,132 to 2,136 |
| the other 74 rules | 24.3, 24.1, 24.1 | 24.1 | 18.2, 18.1, 18.0 | 18.1 | 471 | 118 to 119, then 118 |

The two compiler-backed families account for about 224 of the 248 seconds at opt-level 0, and
about 34 of the 53 at opt-level 2. Each family is 6.2 to 6.4 times faster. The other 74 rules
are 1.3 times faster, because they too run through dependencies such as tree-sitter and
serde. Every selection's finding set is byte-identical across the two sides. The other 74
rules' set is exactly the full set less the two `copy-clones` lines. Their 118 MB peak shows
that a run selecting neither compiler-backed rule never loads `ra_ap_hir` on either side.

### A narrower set was measured, and it is not enough

The obvious alternative is to name only the rust-analyzer packages: the 25 `ra_ap_*` packages
at opt-level 2 and everything else as before. That was built from `HEAD`'s `Cargo.toml`, with
the 25 passed as `--config` overrides. The verbose build log confirms exactly 25 dependency
units at opt-level 2 and the other 211 at 0. Its real gate runs took 133.6, 125.7 and 130.1
seconds, a median of 130.1, with the identical 473 findings and a 2,236 to 2,238 MB peak. Its
cold binary build took 55.5 seconds, one run.

So the frontend's time is not only in the packages named `ra_ap_*`. salsa, rowan, chalk,
hashbrown, triomphe and the rest of what they call carry most of what is left. Naming the 25
would save about 12 seconds of cold build and give back 77.5 seconds of every real gate run.

### Cold builds

Each run used an empty target directory, outside the worktree root, on the same disk. The
before and after runs were interleaved so that drift would fall on both sides alike. The
cargo registry and git caches were warm, so no run downloaded anything.

| Build | opt-level 0 runs (s) | Median | opt-level 2 runs (s) | Median | Difference |
|---|---|---|---|---|---|
| `cargo build -p nomos-cli --bin nomos` | 30.9, 30.8, 30.7 | 30.8 | 67.2, 67.3, 67.7 | 67.3 | +36.5 s, 2.2 times |
| `cargo test --workspace --no-run`, what CI's Test step builds | 64.6, 59.4, 59.7 | 59.7 | 91.1, 89.8, 92.4 | 91.1 | +31.4 s, 1.5 times |
| `cargo clippy --workspace --all-targets`, CI's Lint step | 25.3, 24.7, 29.5 | 25.3 | 60.2, 73.2, 69.9 | 69.9 | about +45 s |

- **The binary and test builds** each compiled the same set on both sides: 294 units for the
  binary, and 323 units producing 231 test executables for the tests.
- **The test build ran under memory pressure on the opt-level 0 side.** Available memory fell
  to between 439 and 1,370 MB, against 5.2 to 10.4 GB on the opt-level 2 side. The unoptimized
  dependencies make larger objects to link. That pressure, if anything, flattered the
  opt-level 2 side, so +31.4 s may understate the difference somewhat on a machine with more
  headroom.
- **The clippy series is the least clean figure here.** A peer session's cargo run and one of
  its test binaries were live during two of its six runs, and a rust-analyzer grew to 2.7 GB
  beside it. An earlier series, discarded because a heavier peer run overlapped four of its
  six runs, measured 25.7, 44.2 and 26.1 seconds against 67.7, 69.7 and 65.9. Both series agree
  on roughly 40 to 45 seconds. Clippy writes no code for a library it only checks, but it does
  compile every build script and procedural macro, and this setting covers those too.

### Incremental builds

This is the cost the refusal to optimize the workspace's own members is about. In the
worktree's warm target, `crates/languages/nomos-lang-rust-compiler/src/lib.rs` was touched and
the nomos binary rebuilt. That rebuilt 3 units on each side, in 3.8, 3.7 and 3.6 seconds before
and 3.0, 2.9 and 2.9 after, medians 3.7 and 2.9. An edit to a member costs no more than it did,
because members stay at opt-level 0.

### What cargo actually passes

Each profile was checked in a verbose cold build in its own empty target directory.

- **Dev profile** (`cargo build -p nomos-cli --bin nomos -v`):
  - 64 workspace-member units carry no `-C opt-level`, which means 0, and no debug-assertions
    flag, which means rustc's default at opt-level 0: on.
  - 236 dependency units and 39 dependency build-script and procedural-macro units carry
    `-C opt-level=2 -C debug-assertions=on`.
  - The workspace's own two build scripts, nomos-spec-store's and nomos-cli's, stay at 0.
- **Test profile** (`cargo test -p nomos-lang-rust-compiler --no-run -v`): the same split, with
  10 member units at 0 and 187 dependency plus 27 build-script units at 2 with
  `debug-assertions=on`. The test profile inherits the override, as it should.
- **Overflow checks** follow debug-assertions unless cargo passes an explicit flag, and it
  passed none, so they are on everywhere they were on before.
- **Debug info** is `-C debuginfo=1` on both members and dependencies, as before.

`P123`'s `Test_A_Selection_Declaring_No_Compiler_Family_Should_Never_Reach_Its_Provider`
passes built under this profile.

## Decision

**The root `Cargo.toml` carries exactly this, and nothing else changes:**

```toml
[profile.dev.package."*"]
opt-level = 2
```

Every package that is not a member of this workspace is compiled at opt-level 2 in the dev
profile, and therefore in the test profile. Every member stays at opt-level 0. Nothing else in
any profile moves. `debug`, `debug-assertions`, `overflow-checks`, `incremental`,
`codegen-units` and the whole of `[profile.release]` are left exactly as they were.

## Which packages, and why that set

`"*"` is Cargo's name for every package that is not a workspace member. At `d0581b4e` that is
265 of the 348 packages in the resolved graph, against 83 members. It includes the 25
`ra_ap_*` packages, 15 procedural-macro packages, and the 11 packages taken from the pinned XVPE
revision. Build scripts and procedural macros are covered as well, because a `package` override
outranks `build-override` in Cargo's precedence.

- **Not the workspace's own members.** A member is rebuilt on every edit to it or to anything it
  depends on, so optimizing members would move codegen time into every incremental build that
  every session runs. Nothing measured asks for it, since the cost is in the frontend's
  packages. The incremental measurement above shows members untouched by this setting.
- **Not a named list of the rust-analyzer packages.** Measured above, it leaves 77.5 seconds of
  a real gate run on the table to save 12 seconds of cold build, and it is a list someone would
  have to keep true against every `ra_ap_*` release that adds or splits a package. `"*"` needs
  no list and cannot drift.
- **The XVPE packages are covered, and that is consistent rather than incidental.** They are
  dependencies pinned at a revision, not code edited here. The one arrangement in which they
  are edited locally is the opt-in `.cargo/xvpe-local.toml` override. There they are path
  dependencies outside the workspace, still not members, so an edit in the sibling checkout
  rebuilds that crate at opt-level 2. `OD-PLATFORM-004` already says that arrangement is never
  the governing one.
- **Why 2, not 1 or 3.** 2 is what the boarded measurement used and what this record retook.
  Neither 1 nor 3 was measured, so this record claims nothing about them.

## What This Costs

- **Every cold build is longer.** On this machine the nomos binary takes 67.3 s instead of 30.8,
  the workspace's test build 91.1 s instead of 59.7, and a cold clippy about 45 s longer.
- **Every warm target directory rebuilds its dependencies once.** The first build in each one
  after this lands compiles every dependency again at opt-level 2, because the profile is part
  of each dependency's artifact identity. That is about the cold figures above, not an
  incremental few seconds. It includes the shared `F:/repos/nomos/target`, every worktree's
  target, and every private target a session keeps. The first clippy in each re-checks its
  dependencies too. The superseded opt-level 0 artifacts are not removed; they sit beside the
  new ones until somebody cleans the directory.
- **CI.** `.github/workflows/gate.yml` caches nothing, so every CI run is a cold build. On this
  machine the Linux job gains about +31 s in the Test step's build and about +45 s in the Lint
  step. The Rules step's build gains at most +36.5 s, and less where it reuses dependencies an
  earlier step already built at the same settings, which was not measured. Against that, the
  Rules step's own run saves about 196 s. A hosted runner has fewer cores than this machine, so
  its absolute figures will differ. `OD-GATE-027` records that remote gate evidence is
  unavailable, so the CI figure is unmeasured and named as such rather than inferred.
- **Stepping into dependency code in a debugger shows less.** Optimized frames lose locals and
  inline calls. Anyone who needs that can pass
  `--config 'profile.dev.package."*".opt-level=0'` for their own build without changing this
  file.

The trade is favourable wherever a real gate run is paid at least once per cold build. One run
saves about 196 seconds and one cold build of the binary costs about 36.5. That is true of CI,
where the Rules step runs once per job, and of every predicate that performs a real run.

## What Does Not Change

- **No provider, rule, selection or demand changes.** The demand gate still keeps
  `ra_ap_hir` out of a run that selects no compiler-backed rule: the 118 MB peaks above show it
  end to end, and `Test_A_Selection_Declaring_No_Compiler_Family_Should_Never_Reach_Its_Provider`
  still passes.
- **No finding changes.** A finding set that differed in any line between the two builds would
  be a defect to report, never a price accepted for speed. None differed.
- **No test proves anything different.** Debug-assertions and overflow-checks are exactly what
  they were for every package, so every `debug_assert!` and every arithmetic overflow check
  that fired before still fires.
- **The release profile is not touched.**

## What Would Reopen It

- **A finding set that differs between a build with this setting and one without it.** That
  would be a correctness defect in a dependency or in this workspace, and the setting is
  withdrawn until it is explained.
- **A re-measurement in which one real gate run no longer pays for one cold build of the
  binary.** For example, a rust-analyzer release or a toolchain change after which the
  optimized frontend saves less than a cold build costs.
- **The compiler-backed families no longer linking `ra_ap_hir` in-process**, for instance by
  moving to a subprocess or a server. The saving this record buys would then leave with them,
  while the cold-build cost stayed.
- **CI evidence becoming available and showing that the hosted runner's cold-build increase
  exceeds its saving.**

## Relation to Sharing One Load

`P146-THE-SEVENFOLD-SELF-CHECK-COST-IS-INSIDE-EVERY-PREDICATE-THAT-RUNS-A-REAL-GATE-AND-IT-HAS-ALREADY-SERIALIZED-THE-BOARD-2`
holds the other lever: one crate-graph and sysroot load shared by the two families instead of
one each. That changes what one family may assume about another's resolution, and it is
decided there, after this record and against the profile this record sets. This record does
not decide it and makes no claim about its size. What it fixes is the baseline that question is
now weighed against: the two families together are about 34 of a 53-second real gate run, not
224 of 248.

## What Holds It

`Cargo.toml` is the one place the setting lives, and its comment points here. Nothing
mechanical asserts the stanza's presence, because the measurement is what justifies it, not a
test. If the stanza were removed, the only signal would be the four-minute gate run coming
back. The refusals above are held by what already held them: the P123 test for demand, the
unchanged debug-assertions and overflow-checks for what a test proves, and line-for-line
comparison of a real run's findings for correctness.

## What This Record Does Not Decide

- It does not decide sharing a load between families, which belongs to the item named above.
- It does not decide the release profile, or any profile other than dev and the test profile
  that inherits from it.
- It does not decide opt-level 1 or 3, which were not measured.
- It does not decide caching in CI.
