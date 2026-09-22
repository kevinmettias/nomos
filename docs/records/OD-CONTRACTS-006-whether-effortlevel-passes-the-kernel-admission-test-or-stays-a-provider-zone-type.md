---
id: OD-CONTRACTS-006
type: decision
title: EffortLevel fails the kernel admission test because nothing outside this workspace reads its serialized form, and the envelope that would carry it is not kernel protocol either
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - contracts
  - layering
  - agent
relations:
  - target: OD-CONTRACTS-005
    type: relates-to
  - target: OD-CONTRACTS-004
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-HOST-014
    type: relates-to
  - target: OD-PACKAGE-016
    type: relates-to
  - target: OD-PACKAGE-015
    type: relates-to
  - target: OD-CAPABILITY-002
    type: relates-to
---

# EffortLevel fails the kernel admission test because nothing outside this workspace reads its serialized form, and the envelope that would carry it is not kernel protocol either

## Question

`nomos-agent-contracts` depends on `nomos-model-package` for one symbol. `TaskEnvelope`
carries `effort: EffortLevel`, added by `OD-CONTRACTS-004` to give `MODEL-ROUTE-004`'s
six-value enumeration its first real reader, and the crate's only other mention of that
dependency is a doc comment naming `ModelExecutionProfile`. So the agent protocol's envelope
links a package-manifest crate in order to name a six-value enum, and every site that builds
an envelope spells the type by that crate's path.

An external review of `bc0aaac` called the enum protocol vocabulary and asked that it move
down into a neutral contract crate, on the argument that this is the kind of small ownership
mistake that becomes painful later when twenty schemas import the package layer to obtain one
enum.

`OD-CONTRACTS-005` is the test that decides admission to `nomos-contracts`, and it had never
been applied to this type: `OD-CONTRACTS-004` chose the existing type without asking the
question, because its subject was the field and not the field's home.

## What Was Measured

Checked 2026-09-21 at `0104d0b7`, against the code rather than against what the types say
about themselves.

**The edge the review objects to is a permitted one.** `nomos-architecture.json` places
`nomos-agent-contracts` in `Agent` and `nomos-model-package` in `Provider`, and `permits`
grants `Agent` exactly `Protocol`, `Substrate` and `Provider`. An agent contract naming a
provider crate is the declared shape, not a crossing that slipped through.

**No system outside this workspace reads the enum's serialized form.** Three places could
carry it and none does.

- *The transport does not serve the operations that would carry it.* `ServedMethod` names
  `GatePlan`, `GateRun`, `GateExplain`, `GateCompare` and `Correction`. `OD-HOST-014` refuses
  `Agent_Execute` and `Agent_Judge_Role` deliberately and on a stated criterion -- they start a
  subprocess and spend against a ceiling, which a transport may not hand an unauthenticated
  caller -- so their absence is a decision rather than a frontier.
- *No caller supplies an effort through the host at all.* `Handle_Agent_Execute` takes a goal
  and a backend selection; effort is not a parameter of it, and no request type anywhere
  deserializes one. The value is constructed inside this workspace, by the command line's own
  parser.
- *Nothing serializes a `TaskEnvelope` in production.* The two `serde_json::to_string` calls
  on the agent path both serialize a response. The envelope's `Serialize` and `Deserialize`
  derives are exercised by tests and by nothing else.

**The backend adapters translate rather than transmit.** The Claude Code adapter maps the six
values onto that program's own `--effort` vocabulary, `Minimal` and `Low` both onto `low` and
`BackendDefault` onto the flag being omitted, which is `OD-CONTRACTS-004`'s own decision. What
crosses to the external program is a flag word, never this enum's serialized form.

**The manifest does not carry it either.** `ModelExecutionProfile` holds a selector and an
effort and derives no serde implementation at all, and the manifest reader in
`nomos-model-package` parses no effort field. So the package-format argument for keeping the
type where it is -- that its serialized form is part of a manifest a package author writes --
is not available, and is not made below.

**The claim that it is wire vocabulary is prospective, and the type says so itself.**
`TaskEnvelope`'s own doc calls it the wire-crossing shape a peer executor *would* need to see
the requested effort in. That is a statement about a peer that does not exist, which is not
the same kind of fact as a wire that is served today, and this record does not read it as one.

**Where the consumers actually are.** Outside `nomos-model-package`, fifteen files name the
type: the command line's parser and its tests, the Claude Code adapter and its tests, the
resolver and dispatch in `nomos-agent-orchestration`, the envelope itself, and the host
handlers. Inside `nomos-model-package`, three further types name it -- the execution profile,
the effort mapping record and the executor exposure -- so the crate that owns it is also the
crate with the most uses of it that are not the envelope's.

## The Decision

**`EffortLevel` stays in `nomos-model-package`. It is not admitted to `nomos-contracts`, and
no crate is created for it.**

`OD-CONTRACTS-005`'s test asks whether a system that has never linked this workspace must
construct or interpret the type's serialized form correctly in order to agree with Nomos about
what happened. Measured above, no such system exists and none is served: the two operations
that would carry an effort across a boundary are refused by `OD-HOST-014` for a reason
unrelated to this type, and the one shape that would carry it is serialized nowhere.

The test is not satisfied by a prospective peer either, and the difference from the types
`OD-CONTRACTS-005` did admit is the reason. `WorkflowStep` and `PackageKind` were admitted
although their peers do not exist yet, because each is itself the thing a peer must agree
about: a step is the unit a peer executor performs, a package kind is the label a peer
reimplements. `EffortLevel` is not such a thing. It is a field of `TaskEnvelope`, and the
parties that must agree about a field are exactly the parties that must agree about the shape
carrying it -- so the question of where the field lives is settled by where the envelope
lives, and the envelope is in `nomos-agent-contracts`, not in the kernel. This workspace has
already decided that the agent envelope is the `Agent` zone's own contract rather than
protocol every peer speaks, and admitting one of its fields to the kernel while leaving the
shape outside it would assert the opposite one field at a time.

That is `OD-CONTRACTS-001`'s worked example applied unchanged: `SyntaxPayload` was admitted one
band up rather than into the kernel, because the parties who must agree about it are one
capability's own providers rather than every peer that speaks to Nomos.

## Why The Two Other Homes Are Refused

**`nomos-agent-contracts`, the crate that owns the envelope, cannot hold it.** Three of the
enum's consumers are `nomos-model-package`'s own types, and `permits` grants `Provider` only
`Protocol`, `Substrate` and `Capability Contract`. A provider naming an agent crate is
refused, which is the same constraint `OD-PACKAGE-016` reasoned from when it placed the
dispatch resolver in an application service rather than in the package crate. Moving the type
down to the envelope would make the package layer unable to name its own profile's field.

**`nomos-package` is structurally available and semantically wrong.** It sits in `Provider`
beside `nomos-model-package`, which already names it under a declared exception, so both
consumers could reach it. But its subject is the language-agnostic core of an installable-unit
manifest, and effort is execution policy that no manifest in this workspace declares. Putting
it there would widen that crate's subject to hold one type that has nothing to do with
manifests, which is the ownership mistake the review objects to, relocated rather than fixed.
`OD-PACKAGE-015` refuses a new crate for the same reason it refuses most: a crate extracted
now would have no independent versioning, enforce no isolation a module does not, and have no
consumer that does not already depend on its siblings.

## What Would Decide It Differently

Each of these is a fact that can be observed, not a judgement that can be re-argued.

- **An agent operation admitted to the transport.** If `OD-HOST-014`'s criterion is met --
  most plausibly by an authorization mechanism for the spend it refuses today -- then a caller
  that never linked this workspace constructs an envelope, and `OD-CONTRACTS-005`'s test must
  be re-applied to every field type the envelope carries rather than to this one alone.
- **A production site that serializes a `TaskEnvelope`.** Would turn the envelope's own
  prospective claim into a measured one and reopen this record on its own terms, whether or
  not a transport serves it.
- **A consumer of the enum that needs neither the envelope nor the package layer.** Would be
  the second independent party `OD-CAPABILITY-002`'s contention criterion asks about, and
  would put the crate boundary back in front of `OD-PACKAGE-015`'s three clauses.
- **A published schema that names the six values.** If any surface publishes a JSON Schema
  carrying `effort`, the label becomes a protocol commitment in exactly the sense that
  admitted `PackageKind`, and this record's central measurement is falsified.

## What This Does Not Decide

Whether `nomos-agent-contracts` should depend on `nomos-model-package` at all for anything
else, or whether the envelope's other seven fields sit in the right crates. Only the eighth
was asked about here.

Whether `EffortLevel`'s six values are the right ones, or whether the Claude Code mapping's
two approximations should be represented differently. `OD-CONTRACTS-004` owns both, and
neither depends on where the type lives.

Whether `ModelExecutionProfile` should gain serde derives so that a profile can be declared in
a manifest. That is the package layer's own question, and if it is answered yes, the fourth
trigger above is how it reaches this record.

## Status

Accepted. No code moves under this record: it applies `OD-CONTRACTS-005`'s test to one type,
answers that the type fails it today, names the two alternative homes and why each is refused,
and states four observable conditions under which the question is asked again.
