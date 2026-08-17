---
id: OD-AGENT-003
type: decision
title: A skill routes to a contract step or cites it where a citation is checked
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - documentation
  - agents
  - ledger
relations:
  - target: OD-AGENT-001
    type: relates-to
  - target: OD-LEDGER-023
    type: relates-to
---

# A skill routes to a contract step or cites it where a citation is checked

## Question

`OD-AGENT-001` refused a documentation tree for agents on the grounds that a summary of a
checked file is an unchecked copy of it, and it drew the committed agent surface as
`AGENTS.md`, a `CLAUDE.md` that imports it, and skills under `.claude/skills/`. What it did
not reach is a skill restating one *step* of `AGENTS.md`'s own loop in its own words. That
is not a second architecture document — it is a procedure elaborating a single instruction
the contract already gives, which is exactly the shape `OD-AGENT-001` admits ("a reusable
multi-step procedure that cannot be reduced to one command plus a pointer", per `CLAUDE.md`'s
statement of the same rule) — so the restatement it produces was never checked by anything
`OD-AGENT-001` built.

`P11-NEXT-WORK` gave the concrete instance. `AGENTS.md`'s loop step 2 said "Pick an item, or
add one"; `.claude/skills/nomos-task/SKILL.md` section 3 said "Choose an item, or author
one" — one instruction, in two files, in different words. When `OD-LEDGER-023` moved
selection from a read to a computed `next:` line, step 2's sentence changed to say so.
Section 3's did not, because nothing pointed from one to the other and nothing compared
them. The item that exists to stop the contract saying *pick* could close its own
`done_when` while the skill a session actually opens still said it — a defect one step
removed from the one `OD-AGENT-001` already refused, at an address that check does not
reach.

`tests/contract/tests/agent_harness.rs` already runs its route-existence check over every
skill, so a path a skill names is held to the same standard as a path `AGENTS.md` names.
What it does not check is *agreement*: a route can point at a file that still exists while
saying the opposite of what the file now says, and a paraphrase carries no path for that
check to find in the first place.

## The Decision

**A skill adapting one of `AGENTS.md`'s numbered loop steps does it one of two ways, and
both are held by a mechanical check rather than a reviewer's memory.**

**Route, by default.** Where a step's instruction is the whole of what the skill needs to
say, the skill points at the step instead of restating it — "step 2 already answers which
item to claim; read it there" rather than a second sentence describing how the choice is
made. There is one copy, and the question of agreement cannot arise because there is
nothing to disagree. `nomos-task/SKILL.md` section 3 is the fixed instance: the sentence
that mirrored step 2's selection instruction is gone, and what replaced it is a route plus
the one thing the route does not cover — that `next:` does not weigh territory overlap with
a live session, which no step of `AGENTS.md` says and is therefore not a restatement of
anything.

**Cite, where a route is not enough.** Some skill content is a real elaboration of a step —
added detail, a caveat, a reason a plain pointer would strip out — and reducing it to a
pointer would lose exactly the thing `CLAUDE.md` says earns a skill its place. Content in
that shape carries an explicit citation of the step it adapts, in the fixed form
`` AGENTS.md step N: "anchor" ``, where the anchor is a fragment quoted verbatim from that
step's current wording. The citation is not decoration; it is the thing the mechanical
check reads.

## What Is Guarded, And What Is Not

`tests/contract/tests/agent_harness.rs` gains a generic check, not a hardcoded assertion
about step 2: it derives every numbered step's text straight out of `AGENTS.md`'s own loop
section, finds every `AGENTS.md step N: "anchor"` citation any harness file carries — a
skill today, potentially `AGENTS.md` or `CLAUDE.md` themselves later — and fails when a
cited step no longer contains the anchor quoted from it, or no longer exists at all. A
citation is what makes a paraphrase legible to a machine that cannot judge whether two
sentences mean the same thing; the anchor is the fragment the machine can check stayed
true. The check is proven capable of failing over fixture text it constructs itself, not
only shown passing over the real files, for the same reason every derived check in this
file already is: a check whose failing case has never been observed is `OD-GATE-001`'s
defect wearing a new file name.

`Test_The_Contract_Should_Name_The_Selection_Authority_Rather_Than_Instruct_Picking`,
`OD-LEDGER-023`'s own instance check, stays. It is the one case this record's mechanism does
not need to reach, because that content is now routed rather than cited — but it remains
the fixed proof that the specific defect this record generalizes was real.

What is not guarded is whether a route or a citation is the *right* choice for a given piece
of content — whether something reducible to a pointer was actually reduced to one, and
whether a citation's anchor was chosen well. That is the same judgment call `OD-AGENT-001`
already declines to mechanize, applied one level down: the check enforces that a citation
stays true once made, not that a citation should have been made in the first place.

## Status

Closed by `P11-SKILL-AGREEMENT`.
