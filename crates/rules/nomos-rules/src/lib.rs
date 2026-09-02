//! Band 3 — rules that judge source.
//!
//! The first thing in this workspace that judges code rather than judging the
//! workspace's own paperwork. `P10-FIRST-CHECK` opened for that reason: four types in
//! `nomos-contracts` described enforcement and nothing implemented them, and two
//! consecutive batches of work had produced audit rather than capability.
//!
//! # A rule takes its subject as an argument
//!
//! Nothing in this crate opens a file, walks a directory, or knows where the workspace
//! is. A rule takes [`SourceFile`]s and a [`nomos_analysis::FactReader`] and returns
//! findings, and the caller supplies both — the binary composes them from the real tree,
//! and a test composes them from three files it wrote by hand.
//!
//! That is not a style preference. `P10-FIRST-CHECK` requires the three instances
//! `OD-COMPLETENESS-001` analyses to fail this rule *as originally written*, and none of
//! the three can be replayed from git: each was repaired at the site. The only way to
//! judge code that no longer exists is for the rule to accept its subject as an
//! argument. A rule that reads the filesystem can only ever be tested against the tree
//! it is standing in.
//!
//! `D-134` recorded that argument and drew a wider conclusion from it — that the subject
//! must be *text*. It does not follow, and `OD-RULES-001` withdraws it: a reader handed
//! in as a parameter is an argument in exactly the sense a `&[SourceFile]` is, and
//! the replay property is met identically. `D-134` is amended at version 2 rather than
//! superseded; six of its seven decisions stand untouched.
//!
//! # Where this rule gets each half of what it needs
//!
//! Two halves, and they fall on opposite sides of what the agreed payload of
//! `nomos.cap.syntax.items` can carry.
//!
//! *Resolving a claimed mirror* against the checks that really exist is read from facts.
//! [`Syntax_Requirement`] is the floor this crate states for itself, and it is the rule's
//! and not the caller's: a composition root cannot lower it, so a line scanner that
//! reports a `fn Test_Renamed_Away` sitting inside a block comment cannot be substituted
//! for a parser and quietly resolve a claim that nothing checks.
//!
//! *Discovering which declarations are universes* still reads the text it is handed,
//! because the payload carries neither doc comments nor declared types and no version of
//! it exists that would. That is a measurement rather than a preference, and it names its
//! own end condition — see `universe_kind.rs` and `OD-RULES-001`.
//!
//! # What is here
//!
//! Forty-five rules. [`Check_Completeness_Mirrors`] was chosen first because it is the only
//! rule in this tree with three recorded historical instances to test a judgment against —
//! `P10-FIRST-CHECK` shipped with exactly this one and no more, because a single check
//! that is honest end to end is worth more than three that are nearly wired.
//! [`Check_Naming_Convention`] is the second, added once a real second rule was needed to
//! test `OD-PACKAGE-008`'s question — whether a future `RulePackage` manifest's field
//! boundaries generalize past a population of one — against something other than the
//! first rule's own shape. It judges a different kind of claim (a lexical convention
//! stated once in prose, not a per-subject doc comment) for exactly that reason.
//! [`Check_Dependency_Direction`] is the third, designed by `OD-RULES-003`: a declared
//! architecture is data, the observed dependency graph is a fact a capability provider
//! establishes, and this rule composes the two into findings — the same property
//! `tests/contract/tests/boundaries/graph.rs` already enforces for this repository by
//! hand. `P13-DEPENDENCY-EDGES-2` landed the rule and `P13-DEPENDENCY-WIRE-1` composed it
//! into `nomos-check-orchestration`'s real `Run`. [`Check_Unread_Reaches_A_Finding`] is
//! the fourth, the candidate `OD-RULES-008` named and `P13-CONTROLFLOW-REACHABILITY-
//! CAPABILITY` built: a control-flow edge inside one function body either reaches a
//! `Finding` after a fact-read failure or it does not, the first judgment in this crate
//! that is not a flat per-subject decode-and-compare. `P13-CONTROLFLOW-REACHABILITY-WIRE`
//! composed it into `nomos-check-orchestration::Run`, the same way `P13-DEPENDENCY-WIRE-1`
//! composed `Check_Dependency_Direction`.
//!
//! [`RuleRegistry`] is a fourth thing, deliberately not a rule: `OD-RULES-004` extracted a
//! registration contract ahead of a second rule, so a rule package can be designed and
//! built against a stated shape rather than by copying this crate's own hand-written
//! composition. It is additive and unconsulted for selection — `Run()` originally called
//! each rule directly, unconditionally, exactly as `OD-HOST-004` decided; `OD-GATE-017`
//! superseded that with a real per-call `selected: &[RuleId]` gate, not `RuleRegistry`,
//! so [`RuleRegistry`] itself still changes nothing about what runs on any given
//! `nomos check`.
//!
//! [`Check_Declared_Role_Matches_Surface`] is a fifth rule, additive and unwired into
//! `nomos-check-orchestration::Run` the same way [`RuleRegistry`] was before a second rule
//! existed to check its shape against. It is the first rule in this crate to always resolve
//! [`nomos_contracts::Applicability::AgentRequired`] — `CHK-003`'s seventh reporting
//! category, real since `OD-CONTRACTS-002` but produced by no rule until this one — because
//! whether a crate's declared role and its actual public surface agree is a semantic
//! judgment no mechanical provider can make, grounded against real, external
//! architecture-standards precedent (`role_surface_pair.rs`'s own module doc names it) rather
//! than invented. It takes plain data instead of a [`nomos_analysis::FactReader`], and its
//! own module doc explains why.
//!
//! [`Check_Lint_Diagnostics`] is a sixth rule, `OD-RULES-010`'s own first `ToolProvider`
//! increment: `nomos.cap.lint.diagnostics` facts, materialized by `nomos-lang-rust-clippy`
//! from a real `cargo clippy` run, relayed 1:1 as `Finding`s rather than judged a second
//! time — the first rule in this crate whose whole judgment is "the tool already decided,"
//! stated as its own `Requirement` the same way every other rule states its own floor.
//! Composed into `nomos-check-orchestration::Run`, behind `OD-GATE-017`'s own per-call rule
//! subset, the same `Wants(selected, ...)` gate `DEPENDENCY_DIRECTION` already sits behind.
//!
//! [`Check_Dependency_Policy`] is a seventh rule, `OD-RULES-010`'s second real
//! `ToolProvider` instance: `nomos.cap.dependency.policy` facts, materialized by
//! `nomos-lang-rust-deny` from a real `cargo deny check bans licenses sources` run,
//! relayed 1:1 the identical way [`Check_Lint_Diagnostics`] already relays `cargo clippy`'s
//! own verdict — minus even that rule's own per-member loop, since this capability's one
//! real provider materializes exactly one fact for the whole workspace.
//!
//! [`Check_Cross_Language_Correspondence`] is an eighth rule, `OD-CAPABILITY-010`'s own
//! work: the first rule in this crate that reads one capability twice for one judgment,
//! over a subject *pair* a doc-comment-declared correspondence names rather than a subject
//! the walk handed it directly. No new capability — both sides are already `nomos.cap.
//! syntax.items` facts — and no new materialization, since every source's syntax fact is
//! already written before any rule runs.
//!
//! [`Check_Every_Member_Declares_A_Band`] is a ninth rule, `OD-RULES-003`'s own design
//! applied a second time: [`Check_Dependency_Direction`] already judges declared
//! architecture against the observed `nomos.cap.dependency.edges` fact for *direction*,
//! and `dependency/violations.rs` has always silently skipped a package with no declared
//! band, naming the gap as `tests/contract`'s own `Test_Every_Member_Should_Declare_A_Band`
//! subject rather than a rule's. This rule is that subject, promoted to a Finding-producing
//! *coverage* judgment over the identical fact and the identical band table — no new
//! capability, no new provider, and no new contract citation, since it is the same design
//! rather than a second one.
//!
//! [`Check_No_Trailing_Whitespace`] is the tenth rule and the first imported from
//! code-standards' plain text formatting policies: `no-trailing-whitespace` needs no
//! provider, because the deciding evidence is the exact `SourceFile::text` this crate is
//! already handed.
//!
//! [`Check_Test_Names_Describe_Behavior`] is the eleventh and reuses the same syntax fact
//! path as [`Check_Naming_Convention`]: a function already visible as a test by its `Test_`
//! prefix must name the expectation with `_Should_` or `_Should_Not_`.
//!
//! [`Check_Todo_Format`] is the twelfth and shares the text-local source hygiene shape with
//! [`Check_No_Trailing_Whitespace`]: a deferred-work marker comment must carry owner,
//! description and ticket in the code-standards format. That rule's own documentation says
//! why neither it nor this line spells the marker out.
//!
//! [`Check_File_Size_Review_Trigger`] and [`Check_File_Size_Justification_Trigger`] are
//! the thirteenth and fourteenth rules, importing code-standards' ~500-line review and
//! ~1500-line justification thresholds as text-local judgments.
//!
//! [`Check_Data_Names_Stay_Lower_Snake`] is the fifteenth rule, importing the subset of
//! `data-names-stay-lower-snake` visible in `nomos.cap.syntax.items`: module names and
//! named struct fields.
//!
//! [`Check_Project_Owned_Function_Names_Use_Upper_Snake_Case`] is the sixteenth rule and
//! gives the existing function-name judgment the exact code-standards rule id and blocking
//! gate.
//!
//! [`Check_No_Mod_Rs_Files`] is the seventeenth rule, importing code-standards' Rust
//! module-layout rule as a path-local check over `src/**/mod.rs` while preserving the
//! standard's explicit shared integration-test-module exemption.
//!
//! [`Check_File_Name_Matches_Declared_Type`] is the eighteenth rule, importing the subset
//! of `file-name-matches-declared-type` visible through `nomos.cap.syntax.items`: public
//! structs, enums, traits and type aliases in files whose stem is meaningful to compare.
//!
//! [`Check_Scripts_Use_A_Portable_Shebang`] and [`Check_A_Script_Declares_Its_Purpose`]
//! are the nineteenth and twentieth rules, importing the text-local script-discipline
//! checks that can be decided from a shebang script's first lines.
//!
//! [`Check_Single_Letter_Names`] and [`Check_Boolean_Predicates`] are the twenty-first and
//! twenty-second rules, importing the parts of code-standards' general naming rules that
//! are visible in syntax item facts: declared item names and named struct fields.
//!
//! [`Check_One_Public_Type_Per_File`] is the twenty-third rule, importing the public-surface
//! form of code-standards' one-file-home-type rule from top-level public type-like syntax
//! items.
//!
//! [`Check_Parameter_Count`] is the twenty-fourth rule, importing the definitely decidable
//! part of code-standards' four-value-parameter cap from syntax item arity.
//!
//! [`Check_No_Decorative_Section_Dividers`] is the twenty-fifth rule, importing the
//! conservative text-local half of code-standards' comment-divider discipline.
//!
//! [`Check_Go_Type_Names_Use_Camel_Case`] is the twenty-sixth rule, importing
//! code-standards' Go type-name convention from the path, type-like syntax item names and
//! Go visibility facts the syntax payload already carries.
//!
//! [`Check_Exported_Go_Functions_Use_Upper_Snake_Case`] is the twenty-seventh rule,
//! importing code-standards' Go exported-function convention from function names and Go
//! visibility facts already present in the syntax payload.
//!
//! [`Check_Go_Helpers_Package_Five_Inputs`] is the twenty-eighth rule, importing the
//! Go-specific parameter-count rule id through the same syntax arity facts as
//! [`Check_Parameter_Count`]. Both are now presets over [`Check_Function_Arity_Policy`],
//! because rule id, file selection, threshold, receiver allowance and gate category are
//! policy dimensions rather than separate rule engines.
//!
//! [`Check_Unwrap_Expect_Discipline`], [`Check_Panics_Are_Justified_Documented_And_Validated`],
//! [`Check_A_Rust_Path_Stays_Within_Its_Own_Subtree`] and [`Check_Shared_Interior_Mutability_Says_Why`]
//! are the twenty-ninth through thirty-second rules, importing Rust text-local standards whose
//! evidence is visible in one source file without a new provider: panic primitive spelling, path
//! attribute values and explicit shared `RefCell` ownership escapes.
//!
//! [`Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter`] is the thirty-third rule,
//! the unexported half of [`Check_Exported_Go_Functions_Use_Upper_Snake_Case`]'s same
//! `Upper_Snake_Case` convention: Go decides visibility by a name's first letter rather than a
//! keyword, so the convention's unexported form lowercases only that letter and keeps the rest
//! of the shape the exported check already judges.
//!
//! [`Check_Go_File_Size_Review_Trigger`] and [`Check_Go_File_Size_Hard_Trigger`] are the
//! thirty-fourth and thirty-fifth rules, importing Go's own lower pair of the file-size
//! triggers [`Check_File_Size_Review_Trigger`] and [`Check_File_Size_Justification_Trigger`]
//! already judge generically — 500 and 1000 lines rather than 500 and 1500 — under their own
//! code-standards rule ids and scoped to `.go` sources.
//!
//! [`Check_Deprecation_Carries_A_Reason`] is the thirty-sixth rule, importing the two
//! text-decidable forms of code-standards' `deprecation` rule: a Rust `#[deprecated]` (or
//! one whose arguments close on the same line and name no `note`) and a Go `// Deprecated:`
//! with nothing after the colon. A multi-line Rust attribute is left unjudged rather than
//! guessed at.
//!
//! [`Check_Go_Constants_Split_By_Export`] and [`Check_Go_Variables_Use_Lower_Snake_Case`]
//! are the thirty-seventh and thirty-eighth rules, importing code-standards' remaining Go
//! data-name conventions from the syntax payload's `Constant` and `Variable` items: a
//! constant's case splits by export status the same way [`Check_Go_Type_Names_Use_Camel_Case`]
//! already splits Go type case, and a top-level `var` is judged against `lower_snake_case`
//! without that split — locals, parameters and struct fields are outside what either check
//! can see, the latter already covered by [`Check_Data_Names_Stay_Lower_Snake`] regardless
//! of language.
//!
//! [`Check_Every_Allow_Carries_A_Justification`] and [`Check_Unsafe_Justification`] are the
//! thirty-ninth and fortieth rules, the same "a Rust construct needs an adjacent
//! explanatory comment" shape [`Check_Panics_Are_Justified_Documented_And_Validated`] and
//! [`Check_Shared_Interior_Mutability_Says_Why`] already generalize through
//! `rust_text::Previous_Comment_Block_Has` — a real third and fourth consumer of that
//! primitive, not a new one invented for them.
//!
//! [`Check_A_Discarded_Error_Is_Explained`], [`Check_A_Skipped_Test_States_Why`],
//! [`Check_An_Excluded_File_Says_Why`], [`Check_Suppression_Directives_Carry_A_Reason`]
//! and [`Check_Workspace_Markers_Carry_A_Reason`] are the forty-first through forty-fifth
//! rules, the Go form of the same "a marker needs an adjacent or attached reason" shape —
//! a new `go_text` module rather than a capability, since nothing about whether these Go
//! markers need a reason is a value a repository would configure. `Check_A_Skipped_Test_
//! States_Why` judges two distinct shapes under one rule id: `t.Skip`/`t.Skipf` need a
//! non-empty call argument, and `t.SkipNow` — which the `testing` package gives no
//! argument to carry one in — needs an adjacent comment instead.
//!
//! [`Check_Declared_Tooling_Language_For_Scripts`] is the forty-sixth rule, and a third
//! `OD-RULES-011` instance after naming and numeric limits: whether a file's extension is
//! one a repository has declared its tooling does not use, read from `nomos.cap.
//! scripting.policy` rather than compiled in. Unlike its two siblings, an unconfigured
//! repository (or one that has not declared a tooling language) resolves to no findings
//! at all — this rule never existed before this capability did, so there is no earlier
//! hardcoded default to fall back to, and code-standards' own `check-script-discipline`
//! states the reason directly: an undeclared language is an opt-out, not a default-in.
//!
//! [`Check_A_Credential_Is_Not_Hardcoded_In_Source`], [`Check_A_Secret_Does_Not_Travel_In_A_Url`]
//! and [`Check_Certificate_Verification_Is_Not_Disabled`] are the forty-seventh through
//! forty-ninth rules, a new `security_text` module rather than a capability or a
//! `language:`-scoped file: each is decidable from raw text against a fixed, narrow
//! pattern set code-standards' own doc bounds explicitly (a provider-format credential
//! prefix, a named URL query parameter, a literal verification-disabling value), and none
//! has a repository-configurable dimension for `OD-RULES-011` to route through a
//! capability. Deliberately narrower than an entropy-based secret scanner or a runtime
//! value tracker — code-standards names both as a dedicated tool's job, not this rule's —
//! and all three treat a test, fixture, or example source the same way
//! `Is_Test_Or_Fixture_Source` already reads code-standards' own stated exemption.
//!
//! [`Check_A_Package_Is_Named_After_Its_Directory`] and [`Check_No_Wildcard_Imports`] are
//! the fiftieth and fifty-first rules, a new `placement` module for two genuine one-offs
//! that share no shape with each other or with anything else this crate ships: a Go
//! package's declared name must match its directory (hyphens and underscores in the
//! directory ignored), and an import in Rust or Go must name what it brings in rather than
//! reach for it wholesale. Both are text-local; neither has a repository-configurable
//! dimension. [`Check_No_Wildcard_Imports`] exempts `use super::*;` once it follows the
//! file's own first `#[cfg(test)]` attribute — the idiom a test module reaching for the
//! subject it exercises — and a Go external test package dot-importing its own subject,
//! but not a curated-prelude wildcard: neither this crate's syntax payload nor a
//! hand-rolled parser can tell a prelude apart from any other wildcard, so that exemption
//! is left unattempted rather than guessed at.
//!
//! [`Check_Atomic_Ordering_Choices_Are_Justified`], [`Check_Seqcst_Justified_Explicitly`] and
//! [`Check_Relaxed_Not_Used_When_Ordering_Matters`] are the fifty-second through
//! fifty-fourth rules, a new `concurrency_text` module and this crate's first rules to touch
//! concurrency: code-standards' own `check-atomic-ordering` is one mechanism enforcing all
//! three, because the five `std::sync::atomic::Ordering` variants partition exactly across
//! them — `Relaxed` to the third, `SeqCst` to the second, the rest to the first — so one
//! flagged call site is judged by exactly one of the three. A fourth consumer of the
//! marker-comment-carries-a-reason shape, but not [`Check_Every_Allow_Carries_A_Justification`]'s
//! looser "any adjacent comment" reading of it: the literal marker `check-atomic-ordering`
//! itself requires (`// atomic-ordering: allow: <reason>`) is what these three port, since the
//! standards' own worked example predates that marker and would pass the doc's prose but fail
//! the tool that enforces it. No repository-configurable dimension, so a shared module rather
//! than a capability.
//!
//! [`Check_Error_Message_Starts_Lowercase`], [`Check_Error_Message_Has_No_Trailing_Punctuation`]
//! and [`Check_Eager_Vs_Lazy_Context`] are the fifty-fifth through fifty-seventh rules, a new
//! `error_text` module: the first two judge a `#[error("message")]` attribute's own message text (an ordinary
//! capitalized first word, or a trailing `.`/`!`/`?`, both of which fight a chain walker's own
//! framing and separators), and the third judges whether a same-line `.With_Context(...)`
//! call's argument was built through one of a closed, standards-named set of allocating
//! constructors rather than passed lazily. Narrower than code-standards' own `check-error-
//! message` tool in one respect: that tool also judges a `write!`/`writeln!` literal inside a
//! hand-written `impl ... Display for ...` block via real parsing, which this crate's
//! text-only convention cannot soundly bound without the brace-depth block tracking it has
//! consistently declined to build — left unattempted rather than guessed at, the `#[error(...)]`
//! attribute form alone being syntactically unambiguous on the line that carries it. No
//! repository-configurable dimension in any of the three, so a shared module rather than a
//! capability.
//!
//! [`Check_Abbreviations`] is the fifty-eighth rule, and a fourth `OD-RULES-011` instance
//! after naming, numeric limits and scripting policy: a declared name's own words judged
//! against a fixed, faithfully-ported default vocabulary (code-standards' own `kernel/
//! config/words/lists.go` and `defaults.go`) extended by a repository's own `nomos.cap.
//! words.policy` declaration. Cross-language and reads no `source.language` restriction —
//! the judgment is over declared syntax-item names, a shape [`nomos_cap_syntax::PayloadItem`]
//! carries identically whichever language produced it, not a convention one language alone
//! states.
//!
//! [`Check_Inline_Always_Justification`] and [`Check_A_Disabled_Test_States_Why`] are the
//! fifty-ninth and sixtieth rules, two more `rust_text` additions on the same "a Rust
//! construct needs an adjacent explanatory comment" shape [`Check_Every_Allow_Carries_A_
//! Justification`] and [`Check_Unsafe_Justification`] already generalize through
//! `rust_text::Previous_Comment_Block_Has` — a fifth and sixth consumer of that primitive.
//! `#[inline(always)]` reads exactly like `#[allow(...)]`: any adjacent comment satisfies
//! it. A bare `#[ignore]` is flagged the same way, but an `#[ignore = "reason"]` value in
//! the attribute itself already states why and is never flagged regardless of a comment —
//! code-standards' own worked example gives the inline value as the primary form and a
//! comment as the fallback, not the reverse. No repository-configurable dimension in
//! either, so `rust_text` rather than a capability.
//!
//! [`Check_No_Single_Line_Function_Bodies`] is the sixty-first rule, a `formatting`
//! addition rather than a `rust_text` one: the deciding evidence is a whole function's
//! shape (signature, brace, body and closing brace all on one line, including an empty
//! `{}`), not a construct-plus-adjacent-comment pattern the file's other Rust rules share.
//! Scoped to Rust only — code-standards names a distinct C# strategy for the same rule id,
//! and Go's own collapsing shape is left unattempted rather than guessed at.
//!
//! [`Check_A_Facade_Publishes_A_Child_One_Way`],
//! [`Check_A_Renamed_Facade_Re_Export_Names_The_Contract`] and
//! [`Check_A_Consumer_Imports_Through_The_Facade`] are the sixty-second, sixty-third and
//! sixty-fourth rules, and the first family here to arrive as three rule documents out of
//! one code-standards tool rather than one document per function. `check-facade-surface`
//! is written as a single binary because all three read the same two statement shapes —
//! `pub mod <child>;` and `pub use <child>::<item>;` — and differ only in what they
//! conclude; this crate's unit of export is the rule rather than the tool, so they are
//! three functions in one `facade` module. No repository-configurable dimension in any of
//! the three, so a leaf module rather than a fifth `OD-RULES-011` capability, by the same
//! test the marker-comment family already applied: a facade either publishes a child twice
//! or it does not, and there is no threshold or vocabulary a repository would state
//! differently.
//!
//! The consumer-import rule is the third in this crate to read across files rather than
//! within one — it collects what every facade publishes before it can judge any import —
//! and it is the first to derive a subject's *module path* from where the file sits, which
//! is why `src/pipeline/mod.rs` and `src/pipeline.rs` are read as one module and a source
//! outside a crate's own source directory roots no facade at all.
//!
//! Composition into a real run is deliberately not part of the increment that landed
//! these. Measured against this workspace first: the double-publication rule reports
//! nothing, the consumer rule reports twelve findings over ten real imports reaching around
//! a crate-root facade, and the alias rule reports fifty-nine, because
//! `pub use id::Id as EntityId;` is this
//! workspace's own settled idiom for a one-type module. That last number is a genuine
//! disagreement between two standards rather than a defect in either, and weighing it is
//! its own decision rather than a side effect of porting the rules.
//!
//! [`Check_Goals_And_Parts_Line_Up`] is the sixty-fifth rule, and the only one here whose
//! subject is not source. It takes no [`SourceFile`]s at all: code-standards'
//! `check-goal-traceability` reads nothing but a repository's own declaration, because
//! which parts are *for* what is intent, and intent is in no file's text. `OD-RULES-001`
//! is satisfied exactly as the other rules satisfy it — the subject is handed in as an
//! argument, here through [`nomos_analysis::FactReader`] rather than through a slice — and
//! nothing in this crate types rules uniformly (`RuleOffer` carries a [`RuleId`] and a
//! record citation, not a function pointer), so a rule whose subject is not source says so
//! in its own signature rather than accepting a parameter it would never read.
//!
//! It is `OD-RULES-011`'s fifth family and its clearest case: every input to the judgment
//! is a value a repository states — the goal set, the part-to-goal mapping, and the
//! ceiling on how thinly one purpose may be spread — so no version of this rule could have
//! compiled its parameters in and still meant anything. `nomos-cap-goals-policy` carries
//! all three and `nomos-repo-goals` reads them out of `standards.json`.
//!
//! One rule document, four judgments — the exact opposite of `facade`'s split one
//! paragraph above, where one code-standards tool carried three published rule documents.
//! The unit of export is the rule either way. An undeclared goal, an orphaned goal, a
//! purposeless part and a smeared goal are four kinds of finding under one id because
//! `goals-and-parts-line-up.md` is one rule.
//!
//! Both of its opt-outs are the Go implementation's own: a repository declaring no goals
//! is judged nothing, since a goal cannot be inferred from code and a system that has not
//! written one down has not taken the discipline on; and a ceiling of zero drops only the
//! spread bound while the two-way audit stands. This workspace declares neither, which
//! makes it the first rule here whose honest answer against its own tree is silence.
//!
//! [`Check_Executed_Scripts_Set_Nounset`] is the sixty-sixth rule, the fourth and last of
//! `script_discipline`'s own family: an executed shebang script that never enables `set -u`
//! (or `-euo pipefail`, or the long `-o nounset`) is flagged on line 1, unless it is a
//! *sourced library* — a file whose only top-level statements define and never do. No
//! repository-configurable dimension, so a leaf addition beside its two siblings rather
//! than a fourth thing needing `nomos.cap.scripting.policy`.

#![forbid(unsafe_code)]

mod checks;
mod declared_universe;
mod facts;
mod reading;
mod registry;
mod universe_kind;

use nomos_cap_syntax::Language;
use nomos_capability::Requirement;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SubjectId};

pub use checks::{
    Check_Atomic_Ordering_Choices_Are_Justified, Check_Relaxed_Not_Used_When_Ordering_Matters,
    Check_Seqcst_Justified_Explicitly, ATOMIC_ORDERING_CHOICES_ARE_JUSTIFIED, RELAXED_NOT_USED_WHEN_ORDERING_MATTERS,
    SEQCST_JUSTIFIED_EXPLICITLY,
    Check_Completeness_Mirrors, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION,
    Check_Dependency_Direction, Check_Every_Member_Declares_A_Band, DEPENDENCY_COMPLETENESS, DEPENDENCY_CONTRACT_RECORD,
    DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION,
    Check_Eager_Vs_Lazy_Context, Check_Error_Message_Has_No_Trailing_Punctuation, Check_Error_Message_Starts_Lowercase,
    EAGER_VS_LAZY_CONTEXT, LOWERCASE_FIRST_LETTER, NO_TRAILING_PUNCTUATION,
    Check_A_Consumer_Imports_Through_The_Facade, Check_A_Facade_Publishes_A_Child_One_Way,
    Check_A_Renamed_Facade_Re_Export_Names_The_Contract, FACADE_ALIASES_NAME_THE_CONTRACT,
    FACADE_CHOOSES_FLATTENING_OR_NAMESPACE, FACADE_CONSUMERS_USE_THE_FACADE_PATH,
    Check_Goals_And_Parts_Line_Up, GOALS_AND_PARTS_LINE_UP,
    Check_Deprecation_Carries_A_Reason, Check_No_Decorative_Section_Dividers, Check_No_Single_Line_Function_Bodies,
    Check_No_Trailing_Whitespace, Check_Todo_Format, DEPRECATION, NO_DECORATIVE_SECTION_DIVIDERS,
    NO_SINGLE_LINE_FUNCTION_BODIES, NO_TRAILING_WHITESPACE, TODO_FORMAT,
    Check_Function_Arity_Policy, Check_Go_Helpers_Package_Five_Inputs, Check_Parameter_Count, FunctionArityPolicy,
    FunctionAritySource, ReceiverAllowance, GO_HELPERS_PACKAGE_FIVE_INPUTS, PARAMETER_COUNT,
    Check_Lint_Diagnostics, LINT_CONTRACT_RECORD, LINT_CONTRACT_RECORD_VERSION, LINT_DIAGNOSTICS,
    Check_Dependency_Policy, DEPENDENCY_POLICY, DEPENDENCY_POLICY_CONTRACT_RECORD,
    DEPENDENCY_POLICY_CONTRACT_RECORD_VERSION,
    Check_Cross_Language_Correspondence, CROSS_LANGUAGE_CONTRACT_RECORD, CROSS_LANGUAGE_CONTRACT_RECORD_VERSION,
    CROSS_LANGUAGE_CORRESPONDENCE,
    Check_Abbreviations, Check_Boolean_Predicates, Check_Data_Names_Stay_Lower_Snake, Check_File_Name_Matches_Declared_Type,
    Check_Exported_Go_Functions_Use_Upper_Snake_Case, Check_Go_Constants_Split_By_Export, Check_Go_Type_Names_Use_Camel_Case,
    Check_Go_Variables_Use_Lower_Snake_Case,
    Check_Naming_Convention, Check_One_Public_Type_Per_File, Check_Project_Owned_Function_Names_Use_Upper_Snake_Case,
    Check_Single_Letter_Names, Check_Test_Names_Describe_Behavior, Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter,
    ABBREVIATIONS, BOOLEAN_PREDICATES, CONSTANTS_SPLIT_BY_EXPORT, DATA_NAMES_STAY_LOWER_SNAKE,
    EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, FILE_NAME_MATCHES_DECLARED_TYPE, GO_VARIABLES_USE_LOWER_SNAKE_CASE,
    NAMING_CONVENTION,
    ONE_PUBLIC_TYPE_PER_FILE, PROJECT_OWNED_FUNCTION_NAMES_USE_UPPER_SNAKE_CASE, SINGLE_LETTER_NAMES,
    TEST_NAME_DESCRIBES_BEHAVIOR, TYPES_USE_UPPER_CAMEL_CASE_LOWER_CAMEL_CASE,
    UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER,
    Check_Unread_Reaches_A_Finding, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
    Check_Declared_Role_Matches_Surface, RoleSurfacePair, DECLARED_ROLE_MATCHES_SURFACE,
    Check_A_Disabled_Test_States_Why, Check_A_Rust_Path_Stays_Within_Its_Own_Subtree,
    Check_Every_Allow_Carries_A_Justification, Check_Inline_Always_Justification,
    Check_Panics_Are_Justified_Documented_And_Validated, Check_Shared_Interior_Mutability_Says_Why,
    Check_Unsafe_Justification, Check_Unwrap_Expect_Discipline,
    A_DISABLED_TEST_STATES_WHY, A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE, EVERY_ALLOW_CARRIES_A_JUSTIFICATION,
    INLINE_ALWAYS_JUSTIFICATION, PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED, SHARED_INTERIOR_MUTABILITY_SAYS_WHY,
    UNSAFE_JUSTIFICATION, UNWRAP_EXPECT_DISCIPLINE,
    Check_A_Discarded_Error_Is_Explained, Check_A_Skipped_Test_States_Why, Check_An_Excluded_File_Says_Why,
    Check_Suppression_Directives_Carry_A_Reason, Check_Workspace_Markers_Carry_A_Reason,
    A_DISCARDED_ERROR_IS_EXPLAINED, A_SKIPPED_TEST_STATES_WHY, AN_EXCLUDED_FILE_SAYS_WHY,
    SUPPRESSION_DIRECTIVES_CARRY_A_REASON, WORKSPACE_MARKERS_CARRY_A_REASON,
    Check_A_Script_Declares_Its_Purpose, Check_Declared_Tooling_Language_For_Scripts, Check_Executed_Scripts_Set_Nounset,
    Check_Scripts_Use_A_Portable_Shebang,
    A_SCRIPT_DECLARES_ITS_PURPOSE, DECLARED_TOOLING_LANGUAGE_FOR_SCRIPTS, EXECUTED_SCRIPTS_SET_NOUNSET,
    SCRIPTS_USE_A_PORTABLE_SHEBANG,
    Check_A_Credential_Is_Not_Hardcoded_In_Source, Check_A_Secret_Does_Not_Travel_In_A_Url,
    Check_Certificate_Verification_Is_Not_Disabled, A_CREDENTIAL_IS_NOT_HARDCODED_IN_SOURCE,
    A_SECRET_DOES_NOT_TRAVEL_IN_A_URL, CERTIFICATE_VERIFICATION_IS_NOT_DISABLED,
    Check_A_Package_Is_Named_After_Its_Directory, Check_No_Wildcard_Imports, A_PACKAGE_IS_NAMED_AFTER_ITS_DIRECTORY,
    NO_WILDCARD_IMPORTS,
    Check_File_Size_Justification_Trigger, Check_File_Size_Review_Trigger, Check_Go_File_Size_Hard_Trigger,
    Check_Go_File_Size_Review_Trigger, Check_No_Mod_Rs_Files,
    FILE_SIZE_JUSTIFICATION_TRIGGER, FILE_SIZE_REVIEW_TRIGGER, FIVE_HUNDRED_LINE_REVIEW_TRIGGER,
    NO_MOD_RS_FILES, ONE_THOUSAND_LINE_HARD_TRIGGER,
};
pub use declared_universe::DeclaredUniverse;
pub use reading::Reading;
pub use registry::{RuleOffer, RuleRegistry, RuleRegistryError};
pub use universe_kind::{UniverseKind, Universes_In};

/// What this crate needs from a syntax provider before it will believe an answer.
///
/// A floor, not a preference, and stated by the rule rather than by whoever runs it.
///
/// # Why the floor is here and not at the call site
///
/// Because the rule is the party that knows what an approximation would cost it. A
/// composition root that could lower this would be able to feed the resolver a line
/// scanner, and `nomos-lang-rust-scan`'s own
/// `Test_The_Declared_Unsoundness_Should_Be_Demonstrable` establishes exactly what that
/// buys: it reports declarations written inside block comments. A `fn Test_Renamed_Away`
/// in a comment or a string literal would then resolve a mirror claim that nothing checks
/// — the defect this rule exists to find, arriving through the rule's own resolver.
/// [`nomos_contracts::Guarantee::Satisfies`] refuses that offer against this floor and
/// `Registry::Resolve` reports `Unmet::BelowRequirement`.
///
/// # Why each axis is what it is
///
/// [`FactVariant::Syntactic`] because a check name is what a file says on its face;
/// nothing here resolves a name or follows a `use`.
///
/// Soundness [`Assurance::Sound`] because every name that resolves a claim of coverage
/// must really be in the token stream. This is the axis that separates the two providers
/// and the only one this rule cannot compromise on.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound`. A parser cannot bound
/// what a macro hid, so no provider of this capability can honestly claim complete — and
/// a floor no provider can meet is not caution, it is a declared need with nothing behind
/// it. What follows from an unknown-complete index is handled where it matters: a check
/// name the index is missing produces a phantom finding, and `mirror.rs` refuses to raise
/// one while any subject went unread.
///
/// [`IncrementalGranularity::File`] because a check name belongs to the file that declares
/// it, and nothing coarser would let one edited file be re-read on its own.
///
/// There is deliberately no `Preferring`. Naming a provider would be the rule deciding
/// what the registry exists to decide.
#[must_use]
pub(crate) fn Syntax_Requirement() -> Requirement
{
    let guarantee = Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );

    return Requirement::New(
        nomos_cap_syntax::Capability(),
        nomos_cap_syntax::CONTRACT_VERSION,
        guarantee,
    );
}

/// [`Syntax_Requirement`], narrowed to `preferred` when the caller names one —
/// `OD-CAPABILITY-009`'s decided fix for a capability whose real offers partition by
/// subject rather than compete over one.
///
/// Takes the preference as data rather than computing it from a path: this crate never
/// depends on a language-provider crate, and recognizing which language a path belongs to
/// is exactly that kind of dependency. The composition root already depends on every
/// registered syntax provider by name — it is the one place allowed to compute
/// `Recognition::Of_Path`, once, and carry the result here as
/// [`SourceFile::preferred_syntax_provider`], the same carried-rather-than-derived
/// convention that field's own sibling `subject` already documents. `None` carries no
/// preference and falls through to the floor above, unpreferenced — the case of a path
/// neither registered provider recognizes, which `OD-CAPABILITY-009` names explicitly
/// rather than leaves implicit. That fallthrough is still safe: nothing materializes a
/// fact under either provider's identity for a path neither recognizes, so an unrecognized
/// path still surfaces as an honestly unread subject rather than a wrongly-addressed one.
#[must_use]
pub(crate) fn Syntax_Requirement_For(preferred: Option<ProviderId>) -> Requirement
{
    let need = Syntax_Requirement();

    return match preferred
    {
        Some(provider) => need.Preferring(provider),
        None => need,
    };
}

/// The language name Go-specific rules name, as `nomos-lang-go` declares it.
///
/// A rule naming one language is inherent to a language-specific norm; knowing the set of
/// languages is not, and this crate names two literals rather than a set. The value has to
/// agree with `nomos_lang_go::LANGUAGE`, which this crate cannot depend on to check --
/// `run_context.rs` carries the test that fails loudly if the two ever drift.
pub const GO_LANGUAGE: &str = "go";

/// The language name Rust-specific rules name, as both Rust providers declare it.
///
/// Both `nomos_lang_rust::LANGUAGE` and `nomos_lang_rust_scan::LANGUAGE` carry this one
/// value, which is why a rule asks about the language and never about a provider identity.
pub const RUST_LANGUAGE: &str = "rust";

/// One file of source, as the caller found it.
///
/// `path` is repo-relative with forward slashes, and it is reporting only. Nothing in
/// this crate keys anything on it: `identity.rs` states the rule for the whole system,
/// and a finding identified by its path is a finding that closes and reopens every time
/// somebody moves a file.
///
/// `check-file-name` names this file for not matching this type. `lib.rs` is the crate
/// root every crate needs as its entry point, so renaming it to follow a type it declares
/// is not a real option — a structural exception this file cannot resolve on its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile
{
    /// Repo-relative, forward slashes. For reporting.
    pub path: String,
    /// The subject the composition root filed this file's facts under.
    ///
    /// Carried rather than derived, and that is the whole of why it is a field. A
    /// [`SubjectId`] is a digest of a *normalized* path, and normalization is an
    /// addressing convention the root owns — `tests/integration/src/corpus.rs` folds
    /// case and drops `.` segments and says why. A rule that computed its own would be a
    /// second answer to that convention, and the two would disagree silently: every
    /// `Require` would miss, every mirror would fail to resolve, and the run would report
    /// a workspace full of unavailable facts rather than a mistake in one function.
    /// Carrying it makes rule and root agree by construction.
    ///
    /// The *inputs* digest is not carried and is recomputed in the rule from `text`. That
    /// is the same deliberate choice `tests/integration/src/slice.rs` documents: if the two
    /// disagree the read misses loudly, and a shared helper would make that whole class of
    /// mismatch untestable.
    pub subject: SubjectId,
    /// The file's full text.
    pub text: String,
    /// Which `nomos.cap.syntax.items` provider identity to narrow toward when this file's
    /// syntax fact is required, if any — `OD-CAPABILITY-009`'s fix for a capability whose
    /// real offers partition by subject rather than compete over one.
    ///
    /// Carried rather than derived, for the identical reason [`SourceFile::subject`] is: a
    /// capability with more than one registered offer over disjoint subjects (today,
    /// `nomos-lang-rust` over `.rs` and `nomos-lang-go` over `.go`) needs its read side and
    /// its write side to agree on which provider answers for one file, and computing that
    /// twice independently is how the two sides drift. The composition root recognizes
    /// `path` against every provider it registers and sets this once, before any rule ever
    /// sees the file — this crate itself never depends on a language-provider crate to
    /// compute it. `None` means either no registered provider recognizes this path, or the
    /// caller named none; [`Syntax_Requirement_For`] treats both the same way, falling
    /// through to the subject-agnostic floor.
    pub preferred_syntax_provider: Option<ProviderId>,
    /// Which language this file is written in, if any registered provider recognizes it.
    ///
    /// Carried rather than derived, for the identical reason [`SourceFile::subject`] and
    /// [`SourceFile::preferred_syntax_provider`] are. A rule whose norm is about one
    /// language used to answer this privately by looking at the extension, and eleven
    /// copies of that three-line test had accumulated across nine modules before
    /// `OD-RULES-014` measured them; the composition root already recognizes `path`
    /// against every provider it registers, so it sets this once and every rule reads one
    /// answer.
    ///
    /// Distinct from `preferred_syntax_provider` and not derivable from it: that field
    /// names the tool that would read the file, and `nomos.cap.syntax.items` has two
    /// registered offers for Rust alone, so a provider identity answers a narrower
    /// question than "which language is this".
    ///
    /// `None` means no registered provider recognized the path, or the caller named none.
    /// A rule restricted to a language treats that as "not my subject", the same as a
    /// different language.
    pub language: Option<Language>,
}

impl SourceFile
{
    /// Builds one, for callers that have a path, a subject and the text.
    ///
    /// `preferred_syntax_provider` starts `None` — a caller that has already resolved one
    /// sets the field directly, since every field here is public for exactly that reason.
    #[must_use]
    pub fn New(path: impl Into<String>, subject: SubjectId, text: impl Into<String>) -> Self
    {
        return Self {
            path: path.into(),
            subject,
            text: text.into(),
            preferred_syntax_provider: None,
            language: None,
        };
    }

    /// Whether this file is written in `language`.
    ///
    /// The one comparison a language-restricted rule makes. It takes the name rather than
    /// a [`Language`] so a rule states its own literal without allocating one per file,
    /// and it answers `false` for an unrecognized file rather than guessing.
    #[must_use]
    pub fn Is_Written_In(&self, language: &str) -> bool
    {
        return self
            .language
            .as_ref()
            .is_some_and(|carried| return carried.As_Str() == language);
    }
}

/// Recognizes a path's language the way the composition root does, for tests only.
///
/// Every `mod tests` in this crate that builds a [`SourceFile`] needs its `language` set,
/// because in production the root sets it and no rule derives it. This is the one stand-in
/// for that step: eleven private copies of an extension test is what `OD-RULES-014` removed
/// from the rules, and eight copies in their tests would be the same defect wearing a
/// `#[cfg(test)]`.
#[cfg(test)]
#[must_use]
pub(crate) fn Recognized_Language_In_Tests(path: &str) -> Option<Language>
{
    return match std::path::Path::new(path).extension()?.to_str()?
    {
        "rs" => Some(Language::New(RUST_LANGUAGE)),
        "go" => Some(Language::New(GO_LANGUAGE)),
        _ => None,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Syntax_Requirement_For_Should_Narrow_The_Floor_To_A_Named_Preference()
    {
        let preferred = ProviderId::New("nomos.test.provider");

        let need = Syntax_Requirement_For(Some(preferred.clone()));

        assert_eq!(need.preferred, Some(preferred));
        assert_eq!(need.minimum, Syntax_Requirement().minimum, "the floor itself is untouched");
    }

    /// The case `OD-CAPABILITY-009` names explicitly: no preference is named, and the
    /// floor falls through to the registry's own, subject-agnostic ranking — the same as
    /// before this fix existed.
    #[test]
    fn Test_Syntax_Requirement_Should_Be_The_Bare_Floor_Returned_When_No_Preference_Is_Named()
    {
        let need = Syntax_Requirement_For(None);

        assert_eq!(need, Syntax_Requirement());
    }

    #[test]
    fn Test_New_Should_Build_A_Source_File_Whose_Preferred_Provider_Starts_Unset()
    {
        let subject = SubjectId::From_Digest(nomos_model::Content_Digest(b"a.rs"));

        let source = SourceFile::New("a.rs", subject, "fn Test_Something() {}");

        assert_eq!(source.path, "a.rs");
        assert_eq!(source.text, "fn Test_Something() {}");
        assert_eq!(source.subject, subject);
        assert_eq!(source.preferred_syntax_provider, None);
    }
}
