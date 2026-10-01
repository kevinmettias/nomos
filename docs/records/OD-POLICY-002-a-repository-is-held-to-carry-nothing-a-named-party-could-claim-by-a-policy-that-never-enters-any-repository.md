---
id: OD-POLICY-002
type: decision
title: A repository is held to carry nothing a named party could claim, by a policy that never enters any repository
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - policy
  - configuration
  - transition
  - provenance
relations:
  - target: OD-POLICY-001
    type: relates-to
  - target: OD-PACKAGE-005
    type: relates-to
  - target: OD-RULES-035
    type: relates-to
  - target: OD-ANALYSIS-012
    type: relates-to
  - target: D-138
    type: relates-to
---

# A repository is held to carry nothing a named party could claim, by a policy that never enters any repository

## Question

A person who writes software for someone else and also keeps repositories of their own needs
the second set to hold nothing the first party could point to as its own: no commit recorded
under the party's identity, no name of the party, its products, systems or tickets, and
nothing drawn from the party's line of business, which is what an invention-assignment clause
reaches for. The need is real for any party and any repository, so the check belongs to this
tool. The difficulty is that everything the check needs to know is private. Who the party is,
which addresses are its identities, which words name it and what its line of business is are
exactly the facts the repositories being protected must not contain.

This record decides where that knowledge lives, what the check judges and when, what happens
when the knowledge is absent, and what the check promises never to write down.

## What Was Measured

Measured on this repository on 2026-09-30.

**No source outside the repository exists.** `OD-POLICY-001`'s table of layers records `User`
as having no source on this host: nothing under `crates/` reads a home or configuration
directory, and the `Environment` port offers one variable and the working directory. Every
policy file the workspace reads is JSON at the root of the repository being judged
(`crates/orchestration/nomos-workspace-discovery/src/policy_file.rs`).

**No transition is judged.** Nothing in this workspace runs at the moment content enters a
commit or leaves the machine. There is no hook, no reading of staged changes, commit messages,
commit ranges or author identities. `OD-PACKAGE-005` names exactly such a guard for the
`Local` publication scope, "a refusal that inspects the set of paths about to enter a commit",
and records that it builds none.

**The walk judges code, not text.** `check`'s walk collects the sources its language packages
recognise. Markdown, JSON, TOML and plain text are never judged, and a third party's name in
`docs/` is as much a claim as one in `src/`.

**A composed rule could not do this.** `OD-RULES-035` decision 4 holds that a rule's axis is
readable only from a place the repository can write it. That is right for the rules it
governs, whose settings are the repository's own choices. It is the opposite of what this
check needs. The repository is the thing under judgment here, and a repository able to write
its own exemption, or to learn the party's name from a committed file, would defeat the check.
The CI gate also has no private policy by construction, so a composed rule would judge nothing
on every CI run.

**The need was met outside the workspace first.** On this machine the same guard ran as a
standalone program installed as global git hooks, with one party's vocabulary compiled into
it. It worked, and its shape is the one this record generalises. What it got wrong was
compiling the party into the tool, which is the part this record forbids.

## The Decision

### 1. The party's knowledge lives in a policy file the user keeps outside every repository

The policy is a JSON file on the user's machine. It is the first source `OD-POLICY-001`'s
`User` layer has. A command names it with `--policy <file>`. With no flag, the command reads
the one variable the `Environment` port offers, `NOMOS_PARTY_POLICY`.

Nothing in this workspace names a party, ships a party's vocabulary, or reads a policy from a
repository. Tests and documentation use invented parties at `example.invalid`. A policy file
committed to any repository is the failure this record exists to prevent. The command does not
look for one there, and it would read one no differently from a policy supplied by flag.

### 2. What a policy says, in a vocabulary with no pattern language

A policy names:

- the **party**, a label used only in refusals;
- the **identities** that are the party's: e-mail domains, matched case-insensitively against
  a commit's author and committer;
- **rules**, each with an id, a reason, and either phrases or ticket keys;
- the party's **own repositories**, where the guard stands aside, as remote-URL prefixes and
  local path prefixes;
- **exceptions**, each a rule id, an exact repository-relative path (or `(message)` for a
  commit message) and a reason. An exception without a reason is ignored.

A phrase is matched without regard to letter case, and without regard to spaces, dots,
underscores or hyphens between its words. "Northwind Parts" matches `northwind-parts`,
`NorthwindParts` and `northwind.parts`. A rule states where the phrase must begin and end:

| Boundary | Meaning |
|---|---|
| `anywhere` | also inside a longer token, which is how `northwind.example.invalid` and `Northwind.Billing` are caught |
| `word` | neither side touches a letter or digit |
| `word-start` | only the start is bounded, so a prefix catches the names built on it |

A rule may ask for exact letter case, for an acronym that is also an ordinary word in lower
case. A ticket key `NW` matches `NW-123`, with at least two digits, bounded on both sides and
case-sensitive.

There is no regular-expression language. The workspace carries no regex dependency, and a
policy written by a person under time pressure should not be able to express a pattern that
matches everything or nothing by accident.

### 3. The guard judges transitions, not trees

`nomos guard` is a new command group:

| Verb | Judges |
|---|---|
| `pre-commit` | the author and committer the commit will record, and every path and added line in the staged changes |
| `commit-msg <file>` | the message, ignoring git's comment lines and everything below a verbose commit's scissors line |
| `pre-push <remote> <url>` | every commit the push would give the remote that the remote does not already have: its identities, its message, and every path and line it adds |
| `scan <root>` | every file at `HEAD`, for auditing a repository before it is guarded |

Only added lines are judged. A line already in a file never blocks unrelated work, and
`pre-push` is the net under `git commit --no-verify` and under commits made before the guard
existed. A file git reports as binary is skipped.

The guard reads git through the `ProgramLauncher` port, the same seam every other external
program in this workspace is reached through.

### 4. An absent policy refuses; it is never read as clean

At a transition, a policy that is not named, or is named but missing or unreadable, is a
refusal with exit `5`, not a pass. A policy that does not parse, or declares a rule with
nothing to look for, is unreadable. The same exit applies when git itself cannot be asked. The hooks exist because a user declared a policy, and a guard that goes quiet when
its policy disappears is the worst way for it to fail. `scan` with no policy at all judges
nothing and exits `6`, which `OD-ANALYSIS-012` keeps apart from a clean `0`. A finding exits
`1`, and usage errors exit `2`. These are the codes `check` already gives the same meanings.

### 5. Nothing the policy says is written anywhere a repository can carry it

A refusal is printed to standard error and nowhere else. The guard writes no report, SARIF,
baseline, ledger entry or cache. Identity findings cannot be excepted: the fix for committing
under the party's identity is to commit under another, not to wave it through. Exceptions live
only in the policy, so no repository file can excuse anything.

### 6. Installation is a file set the user points git at

`nomos guard install --into <directory> --policy <file>` writes one hook script per git hook
name. The three judged hooks call `nomos guard <hook> --policy <file>` and then hand over to the
repository's own hook of the same name. Every other hook only hands over, because
`core.hooksPath` replaces `.git/hooks` entirely, and without that hand-over a repository's own
hooks (Git LFS among them) would stop running. Setting `core.hooksPath` is left to the user,
and the command prints the line to run. Nothing in this workspace edits a user's git
configuration.

### 7. Where it lives

The policy model, the matcher and the transitions are one new crate, `nomos-transition-guard`,
in the `Application Service` zone. The zone sits above `Substrate`, whose `ProgramLauncher` the
guard is handed. The command group is in `nomos-cli`, which composes the standard launcher and
environment.

The matcher is domain-neutral. `D-135` would have it authored in XVPE first. It is built here
under `D-138`, under an ordinary nomos name, because XVPE holds no text-policy facility today
and this is the first caller. The guard is also the transition mechanism `OD-PACKAGE-005`
named. That record's own `Local` path refusal is not built here, but it would run from the
same hooks.

### 8. The capability item that builds it, and its territory

`P183-A-REPOSITORY-IS-GUARDED-AGAINST-A-NAMED-PARTYS-MATERIAL-BY-A-POLICY-IT-NEVER-HOLDS` builds
decisions 1 to 7. Its territory is:

- the new crate, `crates/orchestration/nomos-transition-guard`;
- the `guard` group in `crates/host/nomos-cli` (its module, `main.rs`, `vacuity.rs` and
  `Cargo.toml`);
- the root `Cargo.toml` and `Cargo.lock`;
- `nomos-architecture.json`;
- `README.md`;
- the crate's surface snapshot under `tests/contract/surface/`.

## What This Does Not Do

- **Client-side only.** It does not enforce anything on a server. `git commit --no-verify` and
  `git push --no-verify` skip client hooks, and a hosting service's own rules are outside this
  workspace.
- **Names, not knowledge.** It does not detect knowledge. It catches the names and vocabulary a
  policy lists. Something learned from a party and restated without them passes, and that is a
  judgement for the author, not a pattern.
- **No composed rule.** It does not add a rule to `check` or the gate. Decision 4 and the CI
  gate's missing policy are why.
- **No third-party git reader.** It does not read git through a library. It runs `git`, as the
  user's own hooks do.
- **No repository opt-out.** A repository cannot opt itself out. The party's own repositories
  are excluded by the policy, not by anything a repository declares.

## What Would Decide It Differently

- **A hosting integration that runs server-side.** A pre-receive hook, or a CI step with access
  to a secret policy, would move enforcement off the author's machine. It would then need
  decision 5's promise restated for a place that keeps logs.
- **A second caller of the matcher in XVPE.** That would move it there under `D-135`, leaving
  this crate the policy model and the transitions.
- **A real need for patterns** the phrase and ticket vocabulary cannot express, measured on a
  real policy. That would reopen decision 2. The vocabulary here covers every rule the
  standalone guard carried.

## Status

Accepted. Version 1.
