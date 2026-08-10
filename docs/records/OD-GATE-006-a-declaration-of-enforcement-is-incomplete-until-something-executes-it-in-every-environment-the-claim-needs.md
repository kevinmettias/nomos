---
id: OD-GATE-006
type: decision
title: A declaration of enforcement is incomplete until something executes it, in every environment the claim needs
status: closed
version: 1
authority: canonical-normative-record
tags:
  - gate
  - ci
  - enforcement
  - determinism
relations:
  - target: D-130
    type: affects
  - target: OD-GATE-001
    type: relates-to
  - target: OD-GATE-004
    type: relates-to
  - target: OD-LEDGER-003
    type: relates-to
  - target: OD-DETERMINISM-001
    type: relates-to
  - target: OD-PROJECT-002
    type: relates-to
---

# A declaration of enforcement is incomplete until something executes it, in every environment the claim needs

## Question

This repository governs by mechanism. `OD-AGENT-001` refuses an instruction file that restates
architecture, because a document read by a machine every session and reviewed by a person
approximately never promotes an undecided design to normative status. `OD-GATE-002` derives the
surface check here rather than trusting a tool nobody has. `OD-LEDGER-003` derives the gate's
lint argv from the workflow rather than retyping it, because "two guards for one rule is how
they come to disagree". The pattern is consistent: a claim is worth what the thing that checks
it is worth.

Four claims in this repository are checked by nothing, and each of them reads, in the file that
makes it, like a claim about a mechanism. That is the failure mode the pattern was supposed to
prevent, arriving through a door nobody was watching: not an undocumented rule, but a documented
rule with no executor.

## What Was Measured

At `4844c2b`, on 2026-08-10.

**The gate did not run on the branch the work is on.** `.github/workflows/gate.yml` triggered on
`pull_request` and on `push` to `main`. Every commit in this repository is pushed to `dev` and
reaches `main` only by a merge, so the workflow named `gate` covered the branch that receives
finished work and not the branch that produces it. Earlier in the session that measured this,
the gate was red on `dev` at the `Lint` step, and the only reason anybody knew is that a session
ran clippy by hand. `P10-DIAGRAM-OWED` measured the same shape once already at the
`Required projections` step, and its own reasoning ends by observing that the honest agents are
the ones who leave it broken.

**`deny.toml` had never been executed.** It configures four classes and no step in any workflow
ran `cargo deny`. `D-130` cites it as governing — "`deny.toml` bans the GPU, windowing and UI
crates on the default feature set" — and the file says of that ban that it "is what makes the
distinction hold mechanically rather than by intention". It held by intention.

Its first execution, `cargo-deny` 0.20.2 against `Cargo.lock`:

| class | result |
|---|---|
| advisories | ok |
| sources | ok |
| licenses | **failed, 22 errors** |
| bans | **failed, 19 errors** |

Neither failure was a supply-chain finding, and that is this record's subject rather than a
mitigation. `licenses` refused all 22 workspace members because `license = "UNLICENSED"` is not
an SPDX expression and `private` was unset, so every `publish = false` crate here read as
unlicensed third-party code. `bans` refused 19 of them because `wildcards = "deny"` and a
`{ path = … }` dependency carries no version, so every first-party edge read as a wildcard and
`allow-wildcard-paths` was unset. The GPU ban `D-130` cites did pass — there is no `wgpu`,
`egui`, `winit`, `ash` or `naga` in the tree — and it had passed unobserved since the file was
written.

**`rust-version` claims a floor nothing compiles.** `Cargo.toml` declares
`rust-version = "1.85"`; `rust-toolchain.toml` pins the channel to `nightly` for rustfmt's
unstable brace style, so no build here or in CI has ever used a compiler near the floor. Nine
`&& let` chains are in the workspace today and let chains stabilized in Rust 1.88 under edition
2024, so the declared floor cannot compile the code that declares it.

**Every determinism claim is checked on one operating system.** The gate runs one job on
`ubuntu-latest`. Six crates carry a `determinism.rs` under `OD-DETERMINISM-001` and
`OD-DETERMINISM-002`, and the properties those claims are usually wrong about — path
separators, directory iteration order, filename case, line endings — are exactly what differs
between a Linux runner and the Windows host this repository is developed on. `.gitattributes`
states the risk in as many words, that "the same commit would validate on one machine and not
on another", and defends one instance of it with `eol=lf` and a single test.

## The Decision

> **A declaration of mechanical enforcement, compatibility, or environmental validity is
> incomplete until an executable gate exercises that declaration in every environment necessary
> for the claim to hold.**

Three consequences follow, and each is the reason the sentence is worded the way it is.

**An unexecuted declaration is not merely unverified; it is untested as a declaration.** This is
the finding, and it is stronger than the suspicion that prompted the work. `deny.toml` was
assumed to be correct-but-unrun. Two of its four classes could not have passed on any input,
because they were misconfigured against this workspace's own shape, and nothing in the file or
in `D-130` could have revealed that without running it. A declaration that has never executed
has never had its own syntax, its own scope, or its own applicability checked. So the cost of
leaving one unrun is not the failures it would have caught — it is that nobody knows whether it
could catch anything.

**The class that passes is as load-bearing as the class that fails.** The narrow repair here —
run `advisories` and `sources`, leave `licenses` and `bans` until somebody has time — would have
produced a green step that exercised half a file while reading as though it exercised the file.
That is `OD-GATE-001`'s defect, a check reporting ok having examined nothing, one level up from
where that record found it. A gate step is a claim about what it covers, and a step that names
its config file covers all of it or names the part it covers.

**"In every environment necessary for the claim to hold" is decided by the claim, not by
convenience.** A compatibility floor's necessary environment is that floor. A determinism claim's
necessary environments are the ones the claim says produce identical output, which is more than
one by construction — a determinism claim checked in a single environment is a tautology.
Nothing here requires a matrix for its own sake; what it forbids is choosing the environment
because it was the one already configured.

## Cadence, And Why It Is Not Separated Here

Of the four classes now running, three are functions of `Cargo.lock` and one is not: an advisory
published tomorrow can redden this gate with no commit in between. The obvious response is a
second cadence — the deterministic classes per commit, the advisory database on a schedule.

That is deliberately not done, and the reason is `OD-GATE-001` again rather than laziness. A
scheduled run has no pull request to fail and nobody obliged to act on it, and this repository
already has one step that cannot fail by design and had to argue for itself in a paragraph. A
second signal nobody is required to answer is a gate everybody learns to ignore, which is worse
than a per-commit failure arriving on a day the diff did not cause it. The failure is legible
either way; only one of the two arrangements makes somebody deal with it.

This is a decision about this repository at this size and it is the part of this record most
likely to be revisited. What would change it is a second cadence that somebody is accountable
to, not a larger dependency graph.

## What This Record Does Not Decide

It does not decide anything about a declaration that *is* executed. The workspace lint table in
`Cargo.toml` and the gate's `-D warnings` are two authorities for one severity policy, and both
of them run — that is a disagreement between executors rather than an absent one, and it belongs
to `P11-LINT-AUTHORITY`.

It does not decide what this repository's supported platforms or minimum toolchain *are*. It
decides that whatever they are declared to be must be exercised. Choosing the floor belongs to
`P11-MSRV-UNCHECKED`; choosing the second operating system belongs to `P11-PLATFORM-UNCHECKED`.

It does not make CI a product surface. Nothing here is about a canonical CI policy, a generated
workflow, or a vendor-neutral runner. `ARC-ECOSYSTEM-001`'s rule that current repository
location alone does not make something Nomos applies to this workflow exactly as it applies to
the ledger: `gate.yml` is bootstrap.

## What Was Considered And Rejected

**Write the obligation into `AGENTS.md`.** Refused, and this is the third time a record has had
to refuse it. `OD-AGENT-001` is the reasoning and `ARC-HARNESS-001` cites it for the same
purpose. A hazard line telling authors to run `cargo deny` by hand is a rule enforced by memory,
in a file that is read by a machine and reviewed by a person approximately never — and the
subject of this record is precisely a rule that nothing enforces.

**Delete the unexecuted declaration instead of executing it.** Tempting for `rust-version`,
which is wrong rather than merely unchecked, and wrong for `deny.toml`, whose ban `D-130` cites
as load-bearing. Deleting is the right answer only where the claim is not wanted, and deciding
that is per-declaration work. What this record refuses is the third state: keeping the
declaration, citing it in a governing record, and running nothing.

**Add the checks and accept a red gate.** Refused because a gate that is red on arrival teaches
the same lesson as a gate that never runs. `deny.toml` was repaired in the same commit that
began executing it, and the repairs are visible as configuration rather than as suppression:
`private` and `allow-wildcard-paths` both narrow the check to the case it was written for, and
the ban list was measured still live afterwards by adding a crate that *is* in the graph and
watching `bans` refuse it.

**Require every declaration to name its executor mechanically.** This is the version of this
record that would prevent recurrence rather than describing it — a check that reads the
repository's declarations and refuses one that no gate step reaches. It is refused here for
scope rather than on the merits: there is no enumeration of "declarations" to read, and
inventing one to satisfy this record would put a second authority beside the files that already
make the claims. It is named here so the next reader knows the gap is known.

## What Holds It

`.github/workflows/gate.yml` itself, and the assertions that already read it as text:
`Test_Only_One_Step_Should_Be_Named_Lint` keeps the new steps from colliding with the argv
`work finish` derives, `Test_No_Step_In_The_Gate_Should_Excuse_Itself` refuses
`continue-on-error` on any of them, and `Test_The_Workflow_Should_Not_Appear_Empty` is the floor
that fails if the file is emptied rather than fixed.

What does **not** hold it is worth stating plainly, because the shape of this record invites the
opposite reading. Nothing prevents the next declaration from arriving unexecuted. The two
instances still open are open items and not tests, and if both are declined this record's
principle survives with one instance discharged. That is the honest state and it is why the
paragraph above names the mechanical version and says why it was not built.

## Status

Closed. `P11-UNRUN-POLICY` carries it and discharges two of the four instances measured above:
the gate now runs on the branch the work lands on, and `cargo deny` runs all four classes and
exits 0. `P11-MSRV-UNCHECKED` and `P11-PLATFORM-UNCHECKED` carry the other two and depend on
this item, so the record exists before the instances that cite it rather than after.
