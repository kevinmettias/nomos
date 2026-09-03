---
id: OD-RULES-019
type: decision
title: Whether the five repo-policy providers owe standards.json one shared read, or independent parsing stays the accepted cost of one-capability-one-provider
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - rules
  - repository
  - configuration
  - duplication
relations:
  - target: OD-RULES-011
    type: relates-to
  - target: OD-GATE-011
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
  - target: OD-RULES-005
    type: relates-to
  - target: OD-RULES-006
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
---

# Whether the five repo-policy providers owe standards.json one shared read, or independent parsing stays the accepted cost of one-capability-one-provider

## Question

`OD-RULES-011` decided a rule's configurable parameters are read as a capability fact through
a provider crate under `crates/repository/`, parallel to `crates/languages/` —
`nomos-repo-standards`, its first instance, providing `nomos.cap.naming.policy`. Four more
families followed the identical crate-per-capability shape, deferred by that same record's own
"What This Does Not Do" section to their own future instances: `nomos-repo-limits`,
`nomos-repo-scripting`, `nomos-repo-words`, `nomos-repo-goals`, one per `nomos.cap.*.policy`
contract. Nowhere in `OD-RULES-011` or any later record was a separate question asked: should
the five share how they read and parse `standards.json`, or does each read independently.

Measured directly against this workspace: each crate's `src/reading.rs` declares its own
`const STANDARDS_JSON: &str = "standards.json"`, its own `Discover_Workspace` function, its own
`filesystem.Read_To_String(root.join(STANDARDS_JSON))` followed by its own
`serde_json::from_str`, and its own error type — `NamingPolicyError`, `LimitsPolicyError`,
`ScriptingPolicyError`, `WordsPolicyError`, `GoalsPolicyError` — each a byte-for-byte identical
`{ reason: String }` struct with an identical `Display` impl, renamed per crate.
`run_context.rs::Materialize_Capabilities` runs all five crates' materialization sections in
sequence within one `Run` call whenever their rules are selected together, so a normal full
`nomos check` invocation launches five reads and five parses of the same 9,032-byte file.

The open question is whether that duplication is worth closing with a shared read/parse
boundary beneath the five capability contracts, or whether it is the accepted, bounded cost of
keeping each provider crate self-sufficient.

## What Was Measured

**The duplication is real, current, and already at a five-instance population — not a
hypothetical this record would be deciding ahead of need.** `OD-RULES-011`'s crate-per-capability
shape was chosen once, for one instance, and repeated four more times by precedent rather than by
a decision that weighed sharing the read side. `OD-PACKAGE-006` and `OD-RULES-005`/`OD-RULES-006`
— the closest precedent for "does a repeated shape warrant extraction" — each declined to extract
until a second or third real instance existed to check the design against; this workspace already
holds five, past any threshold those records applied favorably elsewhere.

**What is duplicated is mechanism, not the five capabilities' own authority.** This is not
`OD-GATE-011`'s named defect class in the strict sense that record defines it: the five providers
are not "independently readable as the authoritative answer to the same question" —
`nomos.cap.naming.policy` and `nomos.cap.limits.policy` answer different questions, and neither
could silently drift against the other the way two encodings of one decision can. What is shared
is the physical acquisition step beneath all five: the same file, at the same path, read through
the same filesystem port, parsed with the same library, into an error shape reinvented
identically five times. Collapsing that boundary does not touch the five capabilities' own
semantic independence, and this record does not invoke `OD-GATE-011` as its authority for that
reason — the case here is duplicated mechanism at real, measured, multi-instance scale, argued on
its own evidence.

**`OD-RULES-009`'s sixth amendment already leaned on this measurement as a real, if narrow,
instance of a materialization step wasting real work** — the first such instance that record has
found in six rounds, tracked there and not re-decided here. Declining to close this gap after
citing it as real evidence would leave that finding without a resolution.

## Decision

**A shared read/parse boundary is warranted, beneath the five capability contracts and
providers, which stay exactly as separate as `OD-RULES-011` built them.** A new crate under
`crates/repository/` — `nomos-repo-standards-document` is this record's working name, chosen to
read distinctly from `nomos-repo-standards` (the naming-policy provider, whose own name already
collides with what a reader would expect "the standards document" itself to mean, a landmine
`OD-RULES-011` left unaddressed and this record does not reach) — owns `const STANDARDS_JSON`,
the `Read_To_String`/`serde_json::from_str` sequence, and one shared error type each of the five
providers' own errors can wrap or convert from rather than re-declare. It hands back a parsed
`serde_json::Value` (or an equivalently thin wrapper), not a typed naming/limits/scripting/
words/goals payload — semantic extraction of the `naming`, `limits`, `scripting`, `words` and
`goals` blocks stays exactly where `OD-RULES-011` put it, in each provider's own `reading.rs`.
The five providers each gain one new dependency and lose their own copy of the acquisition step;
their five capability contracts, five `nomos.cap.*.policy` ids, and five provider registrations
are unchanged.

This follows the same discipline `OD-PACKAGE-006`'s resolution already applied to
`KNOWN_PROVIDERS`: share the mechanism once a real population justifies it, without collapsing
the semantics the population exists to keep separate.

## What This Record Does Not Decide

It does not build `nomos-repo-standards-document` or migrate the five providers onto it; that is
a later item's own territory, sized by how much of the migration a single item can honestly
close. It does not decide the new crate's exact public shape beyond "a parsed document and one
error type, no semantic extraction" — the working name and API sketch above are this record's own
proposal, not a binding interface a later item cannot refine. It does not touch
`nomos-repo-standards`'s own name, despite naming the collision risk; renaming a shipped,
registered provider crate is a correction with its own territory and its own cost, not a
consequence of this decision. It does not decide whether a future sixth repo-policy family should
depend on the new crate from its first commit or retrofit later — `OD-RULES-011`'s own "the rest
is deferred" precedent for future families applies unchanged.

## Status

Accepted. The five repo-policy providers stay five capability contracts and five providers; the
file-acquisition mechanism beneath them is decided to warrant one shared crate, not built here.
Revisit if the migration, once attempted, finds the five providers' semantic extraction more
entangled with the raw read than this record's measurement assumed.
