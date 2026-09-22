---
id: OD-POLICY-001
type: decision
title: Policy resolves across ten layers, each field by its shape, and the effective policy names what decided it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - policy
  - configuration
  - gate
  - architecture
relations:
  - target: OD-GATE-011
    type: relates-to
  - target: OD-GATE-015
    type: relates-to
  - target: OD-GATE-018
    type: relates-to
  - target: OD-GATE-029
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-009
    type: relates-to
  - target: OD-RULES-011
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-PACKAGE-011
    type: relates-to
  - target: OD-PACKAGE-016
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
  - target: OD-ROADMAP-002
    type: relates-to
  - target: OD-ROADMAP-003
    type: relates-to
  - target: OD-RULES-029
    type: relates-to
---

# Policy resolves across ten layers, each field by its shape, and the effective policy names what decided it

## Question

An end-user repository tells Nomos what its policy is through files at its root, and this
workspace reads them through five readers that know nothing of each other. `nomos-gate.json`
is read by `crates/orchestration/nomos-gate-orchestration/src/policy/gate_policy_file.rs`,
`nomos-architecture.json` by `crates/repository/nomos-repo-policy/src/architecture/reading.rs`,
`standards.json` by `crates/repository/nomos-repo-policy/src/standards_document.rs` beneath
five family readers, `nomos-test-material.json` by
`crates/repository/nomos-repo-policy/src/test_material/reading.rs`, and `suppressions.json` by
nothing under `crates/` at all. The only precedence rule anywhere in the tree is
`GatePolicyFile::Resolved_Over`, two levels down in the first of those: a policy a caller
built in code beats the file. Nothing records which of the two decided a field.

The corpus asks for more than that in three places. `CONFIG-002` requires the organization,
repository, workspace, user, workflow, gate, CLI, environment and temporary-run layers to
follow a documented deterministic precedence model. `CONFIG-001` requires every configuration
field to identify its schema, source layer, source artifact, precedence, merge/replace
behaviour, sensitivity class, validation state and snapshot identity. And `MODEL-ROUTE-005`
states, for one field, an eleven-position precedence ladder that
`crates/packages/nomos-model-package/src/execution_scope.rs` already transcribes as
`ExecutionScope` while saying in its own doc that it resolves nothing.

No record decides which layers a Nomos policy has, whether a higher layer overrides or merges
a field, or what an effective policy carries so that a report can say which layer decided
each field. An external review at `bc0aaacf` named that as a core product gap. Measured
below, it holds. This record decides the three questions, and a fourth the item asked
alongside them: how the layer set relates to `ExecutionScope`'s eleven tiers without saying
the same thing twice.

## What Was Measured

### The corpus's own configuration requirements

Read from the v14 corpus on disk at `NOMOS_V14_CORPUS`
(`C:/Users/kmett/source/repos/kevinmettias/code-standards/docs/nomos-spec-internal-artifacts`),
and cross-checked through the specification store: `nomos spec sources` reports ten domain
volumes at v14.36, 493 normative statements and 2619 catalog nodes, with nothing it expects
missing. It prints a governing-record count beside those, and that number is deliberately not
quoted here: it is a property of the binary the store was compiled into rather than of the
corpus, it moves with every record any session lands, and it is evidence about nothing below.

One fact about the source matters before any of its content: **a `CONFIG-*` identifier is not
a requirement artifact.** The requirements directory holds 363 files across 61 families, and
`CONFIG` is not one of them — `CONF-*` is there and is package conformance, a different
subject. `CONFIG-001` through `CONFIG-005` exist as five lines of domain volume 04, section
7.14, "Configuration precedence and effective policy", and as catalog nodes; `nomos spec
record --id CONFIG-002` answers that it is in the store as an artifact node, not a requirement
node. Two other families cited below have that same standing, checked the same way: `INT-003`
is a line of domain volume 03, and `MODEL-ROUTE-001`, `-005`, `-018` and `-023` are lines of
domain volume 06, every one an artifact node — the `MODEL-ROUTE` requirement artifacts begin
at `-037`, which is the range `OD-PACKAGE-011` is about. `ADOPT-CONFIG-002` and
`ADOPT-CONFIG-003` are requirement artifacts at maturity `draft`; `ARCH-008`, `WS-001`,
`PKG-005`, `ARCH-ENGINE-006` and `NFR-SEC-002` are requirement nodes; `US-CONFIG-001` and
`US-CONFIG-002` are user-story nodes. All are cited below by identifier, and the difference in
their standing is why `ADOPT-CONFIG-002` is the stronger authority where it and a `CONFIG-*`
line overlap.

- **`CONFIG-001`**: every configuration field shall identify schema, source layer, source
  artifact, precedence, merge/replace behaviour, sensitivity class, validation state and
  snapshot identity. Source *layer* and source *artifact* are two things, and that distinction
  carries most of decision 2 below.
- **`CONFIG-002`**: organization, repository, workspace, user, workflow, gate, CLI,
  environment and temporary-run layers shall follow a documented deterministic precedence
  model. It names the layers and requires that the model be documented; it does not itself
  state the order. This record is that document.
- **`CONFIG-003`**: security and organization policy may prohibit lower-level overrides, and
  rejected overrides shall remain visible with reason.
- **`CONFIG-004`**: users shall inspect and compare effective configurations and preview
  affected applicability, providers, gates, workflows, build variants, caches, retention and
  security boundaries before adoption.
- **`CONFIG-005`**: secret values shall be referenced through provider-neutral
  `SecretReference` objects and never materialized into snapshots, logs, prompts, exports or
  capsules except under explicit policy.
- **The glossary of volume 02** defines `ConfigurationLayer` as "a typed source of
  configuration such as Default, Organization, Repository, Workspace, User, Workflow, Gate,
  CommandLine, Environment, or TemporaryRunOverride, with precedence, merge/replace semantics,
  authority, and sensitivity policy", and `EffectiveConfiguration` as "the immutable resolved
  configuration for one snapshot/run, preserving every contributing, overridden, rejected,
  secret-referenced, and environment-specific value with provenance". That is ten names where
  `CONFIG-002` has nine: `Default` is the tenth, and it sits first.
- **`US-CONFIG-001`** (repository owner) requires the effective value of every governed setting
  to show its schema, source layer, precedence, merge behaviour, sensitivity class and snapshot
  identity, with the acceptance that "Default, organization, repository, workspace, user,
  workflow, gate, CLI, environment, and temporary-run contributions are distinguishable" — the
  same ten, in the same order. **`US-CONFIG-002`** (administrator) requires deterministic
  precedence, merge-versus-replace behaviour and override restrictions for typed fields, with
  the acceptance that "invalid cycles and ambiguous merges are rejected; security and
  organization policy may prohibit lower-layer overrides; the resolved policy is
  reproducible".
- **`ADOPT-CONFIG-002`**: the effective policy shall identify consumer-owned workflow files,
  calibration files, suppressions and temporary run overrides separately from Nomos-owned rule
  declarations and package defaults. **`ADOPT-CONFIG-003`**: a consumer disagreement with a
  rule is represented as typed configuration, suppression, waiver, baseline debt,
  false-positive disposition or risk acceptance.
- **`ARCH-008`**: multiple packages or providers claiming the same file, project, capability or
  operation shall be resolved through explicit precedence, composition or conflict rules;
  silent winner selection is prohibited. **`INT-003`** says the same of repository and
  organization pins. **`WS-001`**: multi-repository workspaces shall define configuration
  precedence, repository-specific rule sets, package isolation and shared contracts.
- **`PKG-005`**: repository and organization policy shall be composed through rule packages,
  gates and repository configuration, and a language package shall not embed repository
  policy.
- **`MODEL-ROUTE-005`**, for the model-execution profile alone: one normative scope-precedence
  order for every client — invocation override; exact logical-operation profile; exact
  judgment-implementation profile; rule/check profile; phase profile; gate profile; workflow
  profile; task-class profile; repository profile; organization profile; backend default —
  where "the first valid value for each profile field wins after higher-authority policy
  constraints, compatibility checks, and explicit merge semantics are applied", and every
  `ResolvedModelExecution` preserves "each candidate, match reason, scope rank, selected value,
  inherited value, rejected override, authority, and rejection reason". **`MODEL-ROUTE-001`**:
  the smallest explicit assignment overrides inherited defaults subject to organization and
  repository policy constraints. **`MODEL-ROUTE-023`**: equal-scope, equal-specificity
  selectors assigning incompatible values produce an `EqualSpecificityConflict` and prevent
  resolution; declaration order, client preference, package load order and hidden
  last-write-wins are all forbidden ways to resolve it.

Where `CONFIG-002`'s listing and `MODEL-ROUTE-005`'s ladder share a name, their orders agree:
organization below repository, repository below workflow, workflow below gate, gate below the
invocation. That agreement is the evidence that the listing order is a precedence order and
not an arbitrary enumeration.

### The five readers, by path

| File | Reader | Layers it reads | Precedence | Absent means | Malformed means | Provenance kept |
|---|---|---|---|---|---|---|
| `nomos-gate.json` | `gate_policy_file.rs`, `Resolve_Gate_Policy` | two: the file, and the `GateCommand` a caller built | `Resolved_Over`: a field the caller left at its default takes the file's; a field the caller built wins | `Ok(None)`, every default | refused; `NoVerdict::MalformedPolicy` withholds the verdict | a digest of the effective values in `GateRunProvenance.policy`; which side decided a field is not recorded |
| `nomos-architecture.json` | `architecture/reading.rs`, `Discover_Workspace` | one | none | empty declaration | refused | none needed today |
| `standards.json` | `standards_document.rs`, `Read_Standards_Document`, beneath `naming`, `limits`, `scripting`, `words`, `goals` | one | none | `serde_json::Value::Null`, which answers every lookup with `None` | refused | none; each row's `Scope` is `Repository` or `Language`, a scope *within* the repository layer, not a source |
| `nomos-test-material.json` | `test_material/reading.rs`, `Discover_Workspace` | one | none | empty list | refused | none needed today |
| `suppressions.json` | nothing under `crates/` | — | — | — | — | — |

Three things about the first row are the measured cost this record answers.

**The one precedence rule exists in two copies.** `Effective_Policy` in
`crates/orchestration/nomos-gate-orchestration/src/gate_environment.rs` and
`Effective_Policies` in `crates/orchestration/nomos-gate-orchestration/src/finding_query.rs`
each call `Resolve_Gate_Policy` and then `Resolved_Over`, with independently written handling
of the unreadable case. That is two artifacts answering one question, the shape `OD-GATE-011`
names, held together today only by their both delegating the merge itself to one method.

**The rule cannot tell absence from a stated default.** `OD-GATE-029` measured this for
`coverage`: `Resolved_Over` decides by `command.coverage == CoveragePolicy::default()`, and
`DeclaredCoverage` in the file defaults to `Unset` too, so a caller cannot state `Unset` over a
file that says `RequireCompleteness`, and a file that writes `"coverage": "unset"` is
indistinguishable from one that omits the key. The same holds for the three list-shaped
policies through `Preferred_Policy`, where an empty list is both "nothing to suppress" and "the
caller said nothing". `Test_A_Policy_A_Caller_Built_Should_Win_Over_The_File` pins the rule as
it stands.

**"The caller" is three layers wearing one coat.** A `GateCommand` reaches `Resolved_Over`
from `nomos-cli`'s gate parsing, which authors `scope` and `rules` from flags and sets every
policy field to its default with a comment saying no flag authors one; from `nomos-api`'s
`Handle_Gate_Run`, which takes a whole `GateCommand` from its caller; from
`nomos-workflow-orchestration`'s `GateBody.command`, a workflow step's own declaration; and
from tests pinning a policy in code. In `CONFIG-002`'s vocabulary those are the CLI, the
temporary-run override, and the workflow layer respectively, and the resolver receives all of
them as one undifferentiated value. A report cannot say which, because the run does not hold
the difference.

Two smaller measurements. `GateCommand.model: Option<ModelExecutionProfile>` is a fifth policy
field on the command that no file fills and nothing reads; `nomos-cli` sets it to `None`.
`suppressions.json` at this repository's root holds a `schema` key and 245 `waivers`; six
comments under `crates/` name it, and every one describes a waiver held by the external
`code-standards` tool, which `OD-GATE-018` adopted for a remediation campaign. It is that
tool's ledger, not a Nomos policy source.

### Which layers have a source on this host

Measured against the tree, not against the corpus's naming.

| Layer | Source today | Observable |
|---|---|---|
| Default | the `Default` impls — `CoveragePolicy::Unset`, empty `SuppressionPolicy`, `BaselinePolicy` and `AdoptionPolicy`, `ArchitecturePayload::default()`, `TestMaterialPolicyPayload::default()`, `Value::Null` for an absent `standards.json` — and the constants a family reader replaces when it reads a value | yes |
| Organization | none. No host, file, or identity names an organization; `NFR-SEC-002` places organization-level authorization in hosted deployments | no |
| Repository | the four files at the root a run is pointed at: `nomos-gate.json`, `nomos-architecture.json`, `standards.json`, `nomos-test-material.json` | yes |
| Workspace | none. `nomos-workspace-discovery` treats a nested `standards.json` as a nested repository root and stops there; nothing reads a file above a repository | no |
| User | none. Nothing under `crates/` reads a home or configuration directory, and the `Environment` port offers one variable and the working directory, nothing more | no |
| Workflow | `GateBody.command`, a workflow step's own `GateCommand` — a real source that reaches the resolver unlabelled | the source exists; the layer is not recorded |
| Gate | none distinct. A root has one gate, and `nomos-gate.json` is the repository's declaration of it; nothing lets a repository declare two | no, collapsed into Repository |
| CommandLine | `--include`, `--exclude` and `--rule` author a scope and a rule selection; no flag authors a policy field | for selection only |
| Environment | `Environment::Variable` is read for `CARGO` by three Rust providers and for `NOMOS_V14_CORPUS` by `nomos-api`; no policy field reads the environment | the mechanism, not a policy source |
| TemporaryRunOverride | a policy a caller built in code — what `Resolved_Over`'s own doc calls the more specific statement — reached through `nomos-api` and through tests | yes, unlabelled |

Three of ten are observable and labelled, two more are observable and unlabelled, and five are
declared by the corpus for a host this workspace does not yet have.

### `ExecutionScope`, and how model routing resolved its own precedence

`ExecutionScope` carries eleven variants in `MODEL-ROUTE-005`'s order, a `Label()`, and one
test, `Test_Labels_Are_Distinct`. Its doc states the boundary: it names the ranking a resolver
would walk and does not resolve, does not build `ResolvedModelExecution`, and is "one more piece
contributed to that eventual assembly, not a license to type a partial wrapper now".
`SelectorSpecificity` beside it transcribes `MODEL-ROUTE-018`'s six-level ranking for ties
*within* one scope, on the same declared-not-computed footing. Nothing outside
`nomos-model-package` names `ExecutionScope`. `OD-PACKAGE-011` at version 3 is what licensed
both transcriptions, as closed corpus enumerations rather than invented taxonomies.

`OD-PACKAGE-016` then decided the first real resolution step, and `Resolve_Profile` in
`crates/orchestration/nomos-agent-orchestration/src/profile_resolution.rs` is that decision
built: it resolves one `ModelExecutionProfile` against a caller-supplied `DeclaredTarget` set,
reads no file, discovers no package, consults no environment variable, resolves the
`BackendFamily` selector only, names the four other selectors' absences as values, and ranks
nothing. It walks no scope and no layer. Model routing resolved its precedence question by not
walking precedence yet: the profile a caller hands it is already the profile that won.

Read against `CONFIG-002`, the eleven tiers are two kinds of thing interleaved. Six name a
layer — `BackendDefault`, `OrganizationProfile`, `RepositoryProfile`, `WorkflowProfile`,
`GateProfile`, `InvocationOverride` — and five name how narrowly a profile is attached to the
operation being routed — `ExactLogicalOperationProfile`,
`ExactJudgmentImplementationProfile`, `RuleCheckProfile`, `PhaseProfile`, `TaskClassProfile`.
The corpus interleaves them deliberately, because for this one field a narrower attachment
beats a more authoritative source: `MODEL-ROUTE-001` says the smallest explicit assignment
overrides inherited defaults, and the ladder puts a rule/check profile above a gate profile
above a repository profile wherever each was written.

## The Decision

### 1. Ten layers, in the corpus's own order, named now whether or not a host can observe them

**Nomos policy resolves across exactly these layers, lowest precedence first: `Default`,
`Organization`, `Repository`, `Workspace`, `User`, `Workflow`, `Gate`, `CommandLine`,
`Environment`, `TemporaryRunOverride`.** They are the ten of the volume 02 glossary and
`US-CONFIG-001`'s acceptance, which are `CONFIG-002`'s nine plus the default every field has
before anyone states it. The order is the order all three corpus listings share, confirmed
against `MODEL-ROUTE-005`'s explicit ladder at every name the two have in common. This record
does not reorder any of them, including the two whose position a reader might argue with —
`Workspace` above `Repository`, and `Environment` above `CommandLine` — because a documented
deterministic order is what `CONFIG-002` asks for and the corpus already documents one; what
would earn a change is named below, and it is an instance, not an argument.

Where each layer lives is the table above, and it is part of the decision rather than
background: five of the ten have no source on this host. **A layer with no source contributes
nothing and is reported as absent, never as an empty contribution.** "Organization: no source on
this host" and "Organization: declared nothing" are different facts, and `ARCH-ENGINE-006`'s
rule that a run with no effective providers reports a no-coverage state rather than success is
the same distinction one tier up. Naming all ten now costs one enum and buys that a report
never has to be re-read when an organization or workspace source appears, and that the layer a
value came from is a name a peer can compare rather than a sentence.

The type is `ConfigurationLayer`, ten variants in this order with a `Label()` in the form every
closed vocabulary in this workspace already has, and it lives in `nomos-contracts` beside
`KnowledgeSourceRole`. That is the same kind of thing — a closed, corpus-transcribed vocabulary
about the authority of a source, with a label a peer reads — and it crosses the same
boundaries: orchestration to host, inside a `GateRunResult`; and gate to agent, when the model
resolver reports the layer of a candidate. `OD-CONTRACTS-001`'s test is whether it crosses a
boundary, and it crosses two.

### 2. A layer is where a value came from; an artifact is which source; provenance carries both

`CONFIG-001` names source layer and source artifact as two fields, and the measurement shows
why: the `Repository` layer has four artifacts today, and `TemporaryRunOverride` has as many
as there are callers. **Every contribution to a field carries the layer it came from and the
artifact within that layer** — a repository-relative path for a file, the step for a workflow
body, the flag for the command line, the variable for the environment, the caller for a policy
built in code, and the build for a default. A composition root that constructs a `GateCommand`
says which layer it is speaking for; it does not get to be "the caller". `ADOPT-CONFIG-002` is
satisfied by the artifact half of this alone: a suppression from `nomos-gate.json` and one
supplied by an API request are told apart by artifact before any layer question arises.

### 3. A field combines by its shape: override, merge, or refuse

The rule is per field and the shape decides it. No layer, no reader and no caller gets a
different rule for the same field.

**Override — a single-valued field.** The highest layer that *states* the field wins; a layer
that does not state it contributes nothing; and a sentinel is not a statement. That last clause
is `OD-GATE-029` carried into the resolver: `Unset` keeps its meaning, `AllowPartial` is how a
layer states partial coverage, and a contribution is present only when the artifact carries
the key. `AllowPartial` is the third `CoveragePolicy` variant that record decided and nobody
has built — `policy/coverage_policy.rs` carries two today — so the resolver takes it as a
decided meaning rather than as a variant already in the tree. Fields: `coverage`; `model`;
`tooling_language`; `max_subsystems_per_goal`; and the architecture declaration as one value
— `components`, `membership`, `permissions`, `exceptions` and `authorities` replaced whole,
because a dependency order assembled from two layers is an order neither author wrote, which
is the silent winner `ARCH-008` prohibits.

**Merge — a set of entries keyed by an identity.** The union across layers, with two rules
inside it: the same key stated at two layers takes the higher layer's entry and records the
lower one as overridden, and no layer removes a lower layer's entry. Fields, with their keys:
suppressions by (rule, subject); baseline debt by (rule, subject); calibrations by rule; naming
rows by (scope, symbol); limits rows by (scope, key); the three words lists by word; forbidden
extensions by extension; fixture locations by location; goals by name; subsystems by name.
Removing a lower layer's entry needs a spelling — a retraction — that this record does not
decide, so today a higher layer can only supersede.

**Refuse — three cases, and declaration order never resolves any of them.** First, one key
stated twice within one layer with different values, whether by two artifacts of that layer or
one artifact twice: `MODEL-ROUTE-023`'s rule for equal-specificity selectors, applied to every
field. Second, a lower layer stating a field a higher layer has locked, per `CONFIG-003`: the
override is rejected and kept visible with its reason; a lock is something a contribution may
carry, and no source that could carry one exists today. Third, an artifact that is present and
cannot be read as its declared shape refuses the run rather than reading as empty, which is
what every reader already does and `Resolve_Gate_Policy`'s own doc says why.

**The model-execution profile is the one field with a precedence of its own, and this record
does not restate it.** `MODEL-ROUTE-005`'s ladder is the order for that field, `ExecutionScope`
is that ladder transcribed, and neither is re-declared or reordered here. What this record
supplies is the axis the ladder's own second sentence demands and cannot state for itself —
the *authority* of each candidate. Every profile candidate the ladder walks carries its
`ConfigurationLayer` and artifact like any other contribution. The six tiers that name a layer
are where that layer's default profile enters the ladder; they are positions in one field's
order, not a second layer vocabulary, and a resolver walking the ladder reports a candidate's
source through `ConfigurationLayer` rather than through the tier name. So the answer to the
item's fourth question is: the gate's layers are not the eleven tiers under another name.
Five tiers are attachment scopes no layer has, six coincide with layers by the corpus's own
choice, and the one vocabulary that governs where a value came from is the layer set. Where a
model resolver maps a layer-named tier onto a `ConfigurationLayer`, that is two encodings of
one corpus sentence and it is done under `OD-GATE-011`'s three conditions — the derivation
named at the site, the reason stated there, and a test pinning the pair — rather than avoided.

### 4. The effective policy is a provenance per field, not a merged blob

`CONFIG-001`, the `EffectiveConfiguration` glossary entry and `MODEL-ROUTE-005`'s second
sentence describe one shape, and it is the shape a report needs to name the layer that decided
each field. **For every single-valued field, the effective policy carries the decided value,
the layer and artifact that decided it, every contribution it overrode with that
contribution's own layer and artifact, and every override it rejected with the reason. For
every merged field it carries the same per entry.** It also carries the ordered list of the ten
layers as consulted, each marked observed, absent on this host, or declared-unobservable, so
that "organization: no source" is a line in the report and not an omission from it.

`GateRunProvenance.policy` stays what it is: a digest of the effective *values*, so that
`gate compare` attributes a difference between two runs to what judged them and not to which
layer happened to say it. Provenance rides beside the digest, never inside it; two runs judged
under identical values from different layers compare as the same policy, because they are.

`CONFIG-001`'s sensitivity class and `CONFIG-005`'s secret references are not adopted: no
policy field this workspace reads holds a secret, and a class with no member is the
population-of-zero shape `OD-ROADMAP-001` retired for its own cluster and not for this one.
Field-level provenance leaves the slot.

### 5. What is re-homed under the resolver, and what stays where it is

**`nomos-gate.json`'s reader is re-homed first.** It is the one reader with two contending
layers, the one precedence rule, and the two call sites that each re-derive the merge. Under
the resolver its contributions become per-field and optional, so absence and `Unset` are told
apart; `Resolved_Over` is retired in favour of the layered resolution; `Effective_Policy` and
`Effective_Policies` call one function; and `Run_Gate` records the `TemporaryRunOverride` or
`Workflow` layer a caller's command speaks for instead of "the caller".

**The five `standards.json` families and `nomos-test-material.json` are re-homed on a named
trigger: the first time a second layer states one of their fields.** Until an organization,
workspace, user or invocation source states a naming case, a limit, a word, an extension, a
goal or a fixture location, each has exactly one contribution, and each already reaches its
rule as a capability fact through the registry (`OD-RULES-011`), which is the distribution
path the resolver feeds rather than replaces. When the trigger fires, the materialization step
in `nomos-check-orchestration` consumes the resolver's contribution for that family instead of
calling the family reader directly. The reader itself does not move in either case, because
`standards.json` is another tool's file with another tool's schema (`OD-HOST-009` at version
2), and a resolver can only consume rows a reader produced from it.

**`nomos-architecture.json` stays independent, permanently.** It is a description the rules
judge against (`OD-RULES-029`), authored only by the repository it describes; no other layer can
contribute to it, so there is nothing to resolve and its provenance is a constant. That
`nomos-lsp` reads it directly as well is a duplicated *read*, not a duplicated decision, and is
not this record's to close.

**`suppressions.json` stays outside.** It is the external tool's waiver ledger under
`OD-GATE-018`, the resolver never reads it, and a report listing the artifacts a run resolved
lists it under none of Nomos's layers.

### 6. The capability item that builds the resolver, and its territory

This record builds nothing. The first increment is one item,
`P124-POLICY-001-EFFECTIVE-POLICY-FIRST-INCREMENT`, not yet on the board, that builds
`ConfigurationLayer`, the resolver, the effective policy with provenance, and re-homes the gate
reader. Its territory, proposed here and validated by execution as the `nomos-task` skill
requires:

- `crates/contracts/nomos-contracts/src/configuration_layer.rs`, new, and
  `crates/contracts/nomos-contracts/src/lib.rs` for its module and re-export;
- `tests/contract/surface/nomos-contracts.txt`;
- `crates/orchestration/nomos-gate-orchestration/src/policy.rs`, and beneath it a new
  `policy/effective_policy.rs` with its `policy/effective_policy/tests.rs`;
- `crates/orchestration/nomos-gate-orchestration/src/policy/gate_policy_file.rs` and
  `policy/gate_policy_file/tests.rs`;
- `crates/orchestration/nomos-gate-orchestration/src/gate_environment.rs` and
  `gate_environment/provenance.rs`;
- `crates/orchestration/nomos-gate-orchestration/src/finding_query.rs`;
- `crates/orchestration/nomos-gate-orchestration/src/gate_plan/gate_run_result.rs`, which
  carries the effective policy to a host;
- `crates/orchestration/nomos-gate-orchestration/src/lib.rs` for the re-exports, and
  `tests/contract/surface/nomos-gate-orchestration.txt`.

Its predicate is the two crates' own test suites, `cargo test --no-fail-fast -p nomos-contracts
-p nomos-gate-orchestration`, together with the workspace lint. Rendering the provenance in
`nomos-cli` and `nomos-api` — `US-CONFIG-001`'s inspection surface — is a second item that
depends on it, in host territory this one does not reserve. `OD-ROADMAP-002`'s pause on new
gate policy increments does not stand in that item's way, and this record does not have to
argue past it:
`OD-ROADMAP-003` finds all three of that record's pauses lapsed, each on its own stated
condition, and states the one constraint that survives the lapse. This record sits inside that
constraint rather than asking for an exception to it — decision 3 decides a field by its shape
and reads nothing but the contributions themselves, and a policy field the resolver adds is
still a field `Resolve_Gate_Policy` reads off `nomos-gate.json`. What the constraint forbids is
not restated here; `OD-ROADMAP-003` is where it is written and where a departure from it would
have to be argued.

## What This Does Not Do

- **It builds nothing.** The layer enum, the resolver, the effective policy and the re-homing
  are the named item's, and no reader changes because this record exists.
- **It gives no layer a source.** `Organization`, `Workspace`, `User` and a distinct `Gate`
  stay unobservable on this host; the record names them so the resolver reports their absence
  honestly, not so a file is invented for them.
- **It does not restate, reorder or amend `MODEL-ROUTE-005` or `ExecutionScope`.** The
  eleven-position ladder is the model-execution profile's own order, and nothing here touches
  `nomos-model-package`.
- **It does not decide a retraction spelling for merged entries**, nor which fields a higher
  layer may lock beyond the fact that a contribution may carry a lock. Both wait for a source
  that would use them.
- **It does not decide rendering.** What a report prints for an overridden or rejected
  contribution is the second item's, subject only to the three states of a layer being
  distinguishable in it.
- **It does not move any reader out of its crate**, and it does not close `nomos-lsp`'s
  second direct read of `nomos-architecture.json`.
- **It does not adopt sensitivity classes or secret references**, for the population reason
  given in decision 4.
- **It does not reopen `OD-GATE-029`.** `AllowPartial` and `Unset` keep exactly the meanings
  that record gave them; decision 3 depends on them.

## What Would Decide It Differently

- **A real `CommandLine` or `Environment` source for a policy field contending with a
  `TemporaryRunOverride`.** The relative order of the three invocation-side layers is the
  corpus's listing and has no instance yet; the first instance is the evidence that would
  confirm it or reopen it.
- **A host that gives "workspace" the container meaning** — an organization-like set of
  repositories rather than the local working set a person opens. `WS-001` can be read either
  way; read as a container, `Workspace` belongs below `Repository`, and that is a corpus
  question this record cannot settle by fiat.
- **A repository that can declare more than one gate.** That separates the `Gate` layer from
  `Repository` and gives `nomos-gate.json` a per-gate shape this record did not have to decide.
- **An organization source.** That activates `CONFIG-003`'s lock and the refuse rule's second
  case, both of which are declared here with no instance to check them against.
- **A merged field whose order is its meaning.** `components` is the only ordered list today
  and is overridden whole for that reason; a second one that must merge would need a rule
  keyed union cannot express.

## Status

Accepted. Decides the ten configuration layers Nomos policy resolves across and their order,
taken from the corpus's own listings and confirmed against `MODEL-ROUTE-005` at every shared
name; that a field combines by its shape, override for a single value, keyed union for a set,
and refusal for a same-layer contradiction, a locked override or a malformed artifact; that
the effective policy carries a layer and an artifact per field with what each overrode and
rejected; and that `ExecutionScope`'s tiers are one field's own ladder, not the layers under
another name, with `ConfigurationLayer` the single vocabulary for where a value came from.
Re-homes the gate reader first, the six family readers on a named trigger, and leaves the
architecture declaration and the external tool's ledger where they are. Builds nothing, and
names `P124-POLICY-001-EFFECTIVE-POLICY-FIRST-INCREMENT` as the item that would.
