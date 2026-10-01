---
id: OD-RULES-028
type: decision
title: A platform implementation is its own zone, so a composition root reaches one only through a composer
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - architecture
  - layering
  - platform
relations:
  - target: OD-RULES-020
    type: affects
  - target: OD-HOST-001
    type: relates-to
  - target: OD-PLATFORM-002
    type: relates-to
---

# A platform implementation is its own zone, so a composition root reaches one only through a composer

## Question

`P88` built `nomos-composer-std` — the std backend set named once — and migrated all four
hosts onto it, so that no host names `StdFileSystem`, `SystemClock`, `FileLock`,
`StdProcessLauncher` or `StdEnvironment` any more. Nothing keeps them migrated.

`ZONES` put `nomos-platform`, `nomos-platform-std` and `nomos-composer-std` all in
`Zone::Substrate`, and `Permits` lets `Host` reach `Substrate` — it must, for
`nomos-model`, `nomos-workspace`, `nomos-capability` and four others. So a host reaching a
platform implementation directly was exactly as legal the day after the migration as the
day before it. The question is whether that should be closed by a rule, and if so by which
one.

## What Was Measured

**The composer is the only production consumer of the implementation.** Measured
2026-09-12, after the four-host migration: nineteen crates name `nomos-platform-std` in a
manifest, and `nomos-composer-std` is the only one that names it under `[dependencies]`.
The other eighteen — seven orchestration crates, three rust language providers, two agent
executors, `nomos-ledger`, `nomos-repo-policy`, `nomos-cap-requirement-trace`,
`nomos-connector-coderabbit`, `tests/contract` and `tests/integration` — all name it under
`[dev-dependencies]`, which `violations::Is_Dev_Dependency` already excludes from this
judgment by name and for a stated reason. A rule therefore lands against zero real
violations, which is what `done_when` asked to see before one was written.

**The manifest edge was already a guard, and a stronger one than `P88` claimed.** `P88`
reported the composer as "a convention with no rule behind it." That was too weak.
Injecting `use nomos_platform_std::StdFileSystem` into `nomos-cli::work` after the
migration fails to compile — `E0432`, unresolved import — because the crate no longer
carries the dependency. What is unguarded is not the use; it is the one
`[dependencies]` line that would restore it. That is a smaller gap than first stated, and
it is still the exact gap four hosts independently fell into when there was nothing else
to do.

**The mechanism the item first proposed does not close it.** The original spelling of this
item proposed a `Zone::Composer` holding `nomos-composer-std`. That forbids nothing:
`nomos-platform-std` would stay in `Substrate`, and `Host → Substrate` stays permitted for
seven other crates. Closing the edge requires moving the implementation *out*, not the
composer *in*.

**`nomos-platform-xvpe` is not a backend, measured rather than assumed.** It was the
obvious second member of any such zone, and it is not one. `XvpeLauncher` is
`XvpeLauncher<'a, Launcher: ProcessLauncher>` — generic over an *injected* launcher,
implementing `xvpe`'s `ProcessLauncherStrategy` over whatever it is handed. It implements
no `nomos-platform` port and can hand no caller a platform; `README.md` already called it
"one adapter, not a replacement for the port." Two Agent-zone crates
(`nomos-agent-executor-claude-code`, `nomos-model-backend-ollama`) depend on it under
`[dependencies]`, so moving it would have fired against real code on the day it landed —
the thing `P88` refused to do and the reason this item existed separately at all.

## The Decision

**`Zone::Backend` holds `nomos-platform-std` alone.** `Permits` grants `Substrate →
Backend` and `Backend → Protocol | Substrate`, and grants `Backend` to nobody else —
`Host`'s arm is the only one below it that names every other zone and deliberately omits
this one. Two entries leave `SAME_ZONE_EDGES`, because the move makes them cross-zone and
`Test_Same_Zone_Edges_Should_Each_Name_Two_Members_Of_The_Same_Zone` would reject them:
`nomos-platform-std → nomos-platform` and `nomos-composer-std → nomos-platform-std`.

The zone is an identity claim, which is the bar `OD-RULES-020` set when it replaced the
band numbers precisely because renumbering had become "an arithmetic side effect" rather
than "a decision about what the crate *is*." `nomos-platform-std` is the one crate in this
workspace that really opens a file, reads the clock and starts a process. A port
declaration and an implementation of it are not the same kind of thing, and the enforcement
follows from the classification rather than motivating it.

**This does not contradict `OD-HOST-001`.** That record states that naming a concrete
provider in a composition root was never the defect, and nothing here forbids it: a host
still names a concrete platform in its own composition root — `nomos_composer_std::
FILE_SYSTEM` is as concrete as `StdFileSystem` was. What is forbidden is reaching *past*
the composer to the implementation crate, a distinction `OD-HOST-001` never drew because no
composer existed when it was written.

## What It Costs

**A zone with one member.** `Zone::Protocol` has had one member since it was declared, so
this is not a new shape, but it is worth naming rather than leaving a reader to notice.

**A permission wider than the fact it protects.** `Permits` answers by zone, so `Substrate
→ Backend` grants the edge to all eight Substrate crates when only `nomos-composer-std`
uses it. A per-crate permission is `SAME_ZONE_EDGES`' shape and does not apply across
zones. Narrowing it means a `Zone::Composer` as well, which would be two new zones to carry
one member each; that is not worth its own row while there is one composer, and it becomes
worth it the moment a second backend set — an in-memory platform for a harness, or one over
`nomos-platform-xvpe` — gives `Composer` a second member. That is the trigger to revisit,
stated so the next reader finds a decision rather than an absence.

**The gate reports the violation as Advisory, not Blocking.** Verified by injection: adding
`nomos-platform-std` to `nomos-cli`'s `[dependencies]` produces
`[Advisory] dependency-direction: nomos-cli (Host) depends on nomos-platform-std (Backend)`
from the rule, which alone would not fail a build. What fails is
`tests/contract/tests/boundaries/graph.rs`'s
`Test_Dependencies_Should_Run_Strictly_Downward`, which reads the same `ZONES` table from
outside `nomos-rules` and panics with the same sentence. CI catches the regression through
the Test step rather than the rule's own severity. Both readers were watched failing on the
same injection and watched passing after it was reverted; neither was reasoned about.

## What This Does Not Do

It does not classify any crate other than `nomos-platform-std`. `nomos-platform-xvpe` stays
in `Substrate` for the reason measured above, and if a real backend under the XVPE ports
ever appears it is a new crate and a new row, not a reclassification of that one.

It does not make the composer mandatory by type. A host could still declare its own
`StdFileSystem`-shaped unit struct and implement the ports itself; nothing here prevents
that, and nothing should, because a host that writes its own platform has made a visible
decision rather than an invisible one.

## Status

Accepted, version 1. Twelve named zones; `nomos-platform-std` moves from `Substrate` to
`Backend`. Amends `OD-RULES-020`, whose own "What This Does Not Do" left open whether the
eleven zones it named were the final set.
