---
id: OD-PACKAGE-005
type: decision
title: A materialized asset's publication scope decides whether it may leave the machine, and an undeclared asset defaults to local
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - packages
  - projection
  - publication
  - ownership
  - ecosystem
relations:
  - target: OD-PACKAGE-004
    type: relates-to
  - target: OD-GATE-005
    type: relates-to
  - target: OD-PROJECT-002
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-PACKAGE-003
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# A materialized asset's publication scope decides whether it may leave the machine, and an undeclared asset defaults to local

## Question

`OD-PACKAGE-004` names who authors a materialized asset's bytes — `GeneratedOwned`,
`Composed`, `UserOwned` — and settles what regeneration may do to each. That record does not
answer a second question about the same asset, and the two do not collapse into one answer:
whether the asset is meant to leave the machine it was written on.

The two vary independently in the four real cases this repository already carries.
`diagrams/relations.mmd` is `GeneratedOwned` and committed — it leaves the machine on every
push. `.claude/settings.local.json` is `UserOwned` and never committed — it never leaves.
Nothing about *who wrote the bytes* predicts *whether the bytes travel*: a renderer's output
is shared today only because this repository chose to ship it, and a scratch rebuild in
`/build/` is exactly as `GeneratedOwned` as the diagram while never being pushed anywhere.
Collapsing the two questions into one enum, as the why-text for this item observes, would
force values that name combinations — "generated-and-shared", "generated-and-scratch",
"user-owned-and-local" — rather than properties, and the set of needed combinations would
grow with every new asset instead of staying fixed at three scopes times three ownership
classes.

This repository already holds the local/shared distinction informally, and unevenly.
`CLAUDE.md` states: "The file .claude/settings.local.json is personal and stays out of
git." `.github/workflows/gate.yml` is tracked and committed
(`git ls-files .github/workflows/gate.yml` returns it). Checking what actually keeps the
first one out of git turns up less than the why-text assumed: this repository's own
`.gitignore` carries no entry for `.claude/` or `settings.local.json` at all — its only
local-scope entries are `/build/`, `/release-artifacts/`, `.nomos/`, `/work/*.lock` and
`/work/*.tmp`. What keeps `.claude/settings.local.json` out of a fresh clone's tracked set
today is the `CLAUDE.md` sentence, full stop; any exclusion a given contributor happens to
see beyond that is at most a personal, machine-local global-`gitignore` entry, which is not
part of this repository, is not reviewed, and does not travel with a clone. That is a
sharper version of the problem this record exists to settle, not a softer one: the flagship
"local, protected" asset in this workspace is today held out of git by prose alone, with no
mechanical backing in the repository whatsoever.

And prose-plus-convention has already failed once, in the adjacent way `AGENTS.md` warns
against directly: "Scope every commit to explicit paths. `git add -A` in this tree sweeps up
work that belongs to somebody else." `CLAUDE.md` repeats the same rule under "The obvious
mistakes." A rule stated twice, in both agent-facing contract files, because a broad stage
already swept in content nobody meant to publish is not a hypothetical this record is
guarding against — it is one this repository has already paid for.

## The Three Scopes

| Scope | What it means | Worked case |
|---|---|---|
| `Ephemeral` | Should not survive the session that produced it. Not meant to be read again once the process that wrote it exits, let alone committed. | `/build/`, `/release-artifacts/`, `.nomos/`: `.gitignore` calls these "Scratch build roots for generated projections... nothing here is a canonical source." `/work/*.lock`, `/work/*.tmp`: the same file's own comment calls the claims, leases and heartbeats stored there "ephemeral" by name. |
| `Local` | Meant to persist on the machine that wrote it, and never published — not this commit, not a later one, not by anyone. | `.claude/settings.local.json`: `CLAUDE.md`, "is personal and stays out of git." |
| `Shared` | Meant to enter repository-distributed state: committed, reviewed, and handed to every clone by `git clone` itself. | `.github/workflows/gate.yml`, `diagrams/relations.mmd`, `spec/domain-specification.md`, `README.md` — all tracked, all part of what `git clone` hands a new contributor. |

**This axis is about repository-distributed state specifically, not every channel bytes could
leave a machine through.** Every worked case above is a Git example — a clone, a push, a
review — because that is the evidence this repository actually has. A backup, a cloud-sync
folder, a diagnostic bundle, a user-initiated export, a remote execution transfer: each is a
different question about a different boundary, none of them addressed by `Shared` here and
none of them answered by this record's silence on them. `Shared` names entry into the
repository's own distributed state; a future asset that needs a policy for one of those other
channels needs a different, separately-argued scope, not an overloaded reading of this one.

**An asset with no recorded scope defaults to `Local`, and publishing it is refused.** The
asymmetry is the same shape `OD-PACKAGE-004` already argues for ownership, pointed the same
direction for the same reason. An asset wrongly kept `Local` when it should have been
`Shared` costs a rerun once somebody notices the omission — the missing file is visible, and
declaring it fixes it going forward. An asset wrongly published when it should have stayed
`Local` is gone the moment it leaves: it is in a commit, possibly in history on a remote
this repository does not control, and there is no diff to recover the world where it never
went. Refusing to publish an undeclared asset is the mistake whose cost is cheap and
reversible; publishing one by default is the mistake whose cost is neither. The safe
direction is `Local`.

## Enforcement For The Local Scope

**A `.gitignore` entry is not the enforcement, and this repository's own recent history is
why.** An ignore entry is advisory in exactly the sense `OD-PACKAGE-004` uses that word for
a self-declared ownership tag: it changes what git *offers* to stage by default and nothing
else.

- It does nothing against an explicit `git add <path>` or `git add -f`, both of which name
  the path directly and bypass the pattern.
- It does nothing against `git add -A` sweeping in a path that was never gitignored to begin
  with but belongs to a different, in-flight session's untracked work — and that is not a
  hypothetical: `AGENTS.md` states the rule "Scope every commit to explicit paths. `git add
  -A` in this tree sweeps up work that belongs to somebody else" as a hazard already
  realized, and `CLAUDE.md` carries the identical rule under "The obvious mistakes."
- It does nothing once a path has ever been tracked, even by accident: `.gitignore` governs
  untracked paths only, so a single wrongful commit permanently defeats the entry for that
  path going forward, silently, with no error and no warning at the moment the follow-up
  `git add -A` re-stages it.
- And, as the previous section found, it may not even be present: `.claude/settings.local.json`
  — the case this repository already treats as the canonical `Local` asset — has no
  committed `.gitignore` entry to be defeated in the first place.

**What `Local` actually requires is a guard on the transition itself** — an asset moving from
`Local` toward `Shared` requires explicit authorization — evaluated at the moment something
attempts that transition, not an omission from a listing. That is a statement about the
transition, not about Git specifically: this repository happens to enforce repository
distribution through Git, so the concrete case is a refusal that inspects the set of paths
about to enter a commit and refuses if any of them is declared `Local`, regardless of how the
path arrived at that set — named explicitly, force-added, or swept in by a wildcard. A
repository under a different distribution mechanism would need the same guard evaluated at
whatever moment *that* mechanism commits to sharing state; a CI publish step, a package
transaction's commit phase, or a repository-host integration's own push hook are each another
projection of the identical rule. None of these exists in this repository today, and this
record states the requirement it owes rather than building one: the procedural version of the
Git-specific case already exists as discipline in `AGENTS.md`'s loop ("Commit the paths you
touched — explicitly, never `git add -A`") and in this skill's own operating rule, but
discipline followed by a person is exactly the advisory category this record is
distinguishing itself from — it protects only for as long as everyone remembers to follow it,
which is the same failure mode as the ignore entry it would replace. A mechanical
transition-time refusal, keyed to a scope declared the same way `OD-PACKAGE-004` declares
ownership — by the thing placing the asset, not by the asset itself — is what closes the gap
regardless of which projection enforces it; this record does not build any of them (see "What
This Is Not").

## Where The Scope Is Declared

The same place ownership is declared, for the same reason `OD-PACKAGE-004` gives: **the
asset does not declare its own scope.** A file that could tag itself `Shared` is authored by
the actor whose mistake this record exists to catch, so self-attestation checks nothing here
either. Scope is recorded by whatever places the asset — the package, or a manifest, once
either exists — next to the ownership class it already records there, as one more fact of the
same kind and at the same site.

This repository builds neither package nor manifest today, the same absence
`OD-PACKAGE-001` and `OD-PACKAGE-003` already found for ownership's declaration site. Until
one exists, this record's own tables are where the real assets' scopes are written down.

## How Scope Composes With Ownership

The two axes are orthogonal, and every real asset in this repository is a coordinate on both
at once. The worked cases below are the same four assets `OD-PACKAGE-004` classifies for
ownership, plus the `Ephemeral` cases `.gitignore` already names, so that all three scope
values have at least one grounded instance and not merely a definition.

| Asset | Ownership (`OD-PACKAGE-004`) | Publication scope | Basis |
|---|---|---|---|
| `diagrams/relations.mmd` | `GeneratedOwned` | `Shared` | Rendered by `nomos spec render --profile diagram-set`, committed with its `.nomos-projection.json` sidecar, required by the gate. |
| `spec/domain-specification.md` | `GeneratedOwned` | `Shared` | Rendered by `nomos spec render --profile domain-specification`, committed the same way, under `OD-PROJECT-002`'s required set. |
| `README.md` | `Composed` | `Shared` | Hand-authored and committed; one region checked against `tests/contract/tests/boundaries.rs`, one region free prose — both regions still ship in every clone. |
| `.claude/settings.local.json` | `UserOwned` | `Local` | `CLAUDE.md`: "is personal and stays out of git." No committed `.gitignore` entry covers it (see Question); prose is today's only barrier. |
| `/build/`, `/release-artifacts/`, `.nomos/`, `/work/*.lock`, `/work/*.tmp` | `GeneratedOwned` | `Ephemeral` | `.gitignore`: scratch build roots holding "nothing... a canonical source," and coordination state its own comment calls "ephemeral." |

The composition is the point of putting scope beside ownership rather than inside it:
`GeneratedOwned` alone predicts nothing about scope — it appears against `Shared`
(`relations.mmd`) and against `Ephemeral` (`/build/`) in this same table, for the same
ownership reason (the renderer is the sole author of every byte) and for two different
publication reasons (one output is a promise this repository ships, the other is a rebuild
scratch root that is never promised to anyone). `UserOwned` and `Local` happen to coincide in
this repository's only `UserOwned` instance, but nothing in either record ties them together:
a future `UserOwned` file a person deliberately wants to share — a personal preset checked in
on purpose — would be `UserOwned` and `Shared` at once, and neither axis would need to change
shape to say so.

## What This Costs If Left As Read Today

Nothing changes at HEAD for any of the five worked cases: the diagram and the specification
render and ship exactly as `OD-GATE-005` and `OD-PROJECT-002` already require, `README.md`
keeps its checked table and free prose, `.claude/settings.local.json` stays untracked by the
same convention it always has, and the scratch build roots stay gitignored and unpublished.
What changes is that a future package or installer has a named third question to answer for
every asset it places — not just what class of ownership the write is, but whether the
result may ever leave the machine — and a stated default for the asset it does not
recognize, instead of the local/shared line being redrawn per asset, informally, the way it
is today.

## What This Is Not

**Not a build of the transition-guard `Local` is stated to require, in any projection.** This
record states the enforcement the scope owes and why a `.gitignore` entry does not meet it; it
does not add a pre-commit hook, a `git` wrapper, a CI step, or a check to `nomos work finish`.
That is future work this record makes nameable, not work it performs.

**Not a manifest, and not the first consumer of one**, for the same reason `OD-PACKAGE-004`
gives for ownership: `OD-PACKAGE-001`'s four-step path to a first manifest reader is
untouched.

**Not a change to any of the five worked assets or the checks over them.** `.gitignore`,
`tests/contract/tests/boundaries.rs`, and the render commands are unchanged. This record
documents the scope each asset already has and states the model that generalizes it.

**Not a claim that `Ephemeral`, `Local` and `Shared` are the only scopes any future asset will
ever need.** They are the three this repository's own worked cases exhibit today, per
`done_when`. A scope this repository has not yet produced an instance of is a question for
whichever record adds the instance.

## Controls

| Weakening | What it produces |
|---|---|
| let an asset declare its own scope | the same self-attestation failure `OD-PACKAGE-004` already refuses for ownership, aimed at the axis that leaks off the machine instead of the one that overwrites a file on it |
| default an undeclared asset to `Shared` | the first installer bug, or the first uncredited `git add -A`, publishes something irrecoverably; the mistake this record already found already realized in this tree once |
| treat a `.gitignore` entry as sufficient enforcement for `Local` | the exact gap this record opens with: advisory against an explicit path, against `git add -A`, and absent outright for the one asset this repository already calls `Local` |
| collapse scope into ownership as combined values | the value set grows with every new asset instead of staying fixed at three scopes composed with three ownership classes, and the combined names describe combinations rather than properties — the failure the why-text for this item states directly |
| treat `Ephemeral` and `Local` as the same scope | a scratch rebuild root and a personal settings file are asked to obey the same rule even though only one of them is meant to persist on the machine at all; `/build/` is rebuilt and discarded every session, `.claude/settings.local.json` is not |

## Status

Accepted. Three scopes are named, `Local` defaults for the undeclared case, and five real
assets are classified against both axes at once. A `Local`-scope transition guard remains
future work; this record is what makes that work nameable rather than assumed. Amended to
version 2 by `P13-PACKAGE-REFINE`, which narrowed `Shared` to repository-distributed state
specifically — every worked case is a Git example, and the prior wording claimed a broader
"leaves the machine" scope no evidence here supports — and generalized the enforcement
section from a Git-specific stage-or-commit refusal to a transition guard with Git named as
one projection among others.
