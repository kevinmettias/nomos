//! `P45-RULES-CALIBRATED-AGAINST-CODE-THEY-WERE-NOT-TUNED-ON`'s own `done_when`: judges
//! `tests/integration/fixtures/third-party/hex-0.4.3/` -- a committed, unmodified excerpt of
//! `hex` 0.4.3's own library source (see the fixture directory's own `PROVENANCE.md`) -- with
//! this workspace's full composed rule set, through the identical `nomos_check_orchestration::
//! Run` seam `nomos-cli check`/`nomos gate run` compose through
//! (`crates/orchestration/nomos-check-orchestration/tests/run_seam.rs` is the pattern this
//! file follows, adapted to a fixture root rather than this repository's own).
//!
//! # Every one of the composed rules gets a stated verdict, not a total
//!
//! [`Verdicts`] names one entry per rule [`nomos_check_orchestration::Composed_Rules`]
//! currently composes -- [`Test_Table_Names_Exactly_The_Composed_Rule_Set`] asserts the two
//! sets are identical, so a rule added or removed there must gain or lose an entry here
//! rather than silently changing what this file measures. [`Expected::Clean`] is a real,
//! checked zero: the rule looked at the fixture's own code and found nothing to report.
//! [`Expected::TruePositive`] names a count where every finding is individually justified
//! below as a real, deserved report against this specific fixture's own code.
//! [`Expected::NotAllDeserved`] -- currently claimed by no rule, see its own doc -- pins a
//! real, currently-produced count where at least one of
//! the findings is *not* claimed as deserved -- named and explained below, per `done_when`'s
//! own "or is an open item naming the rule" allowance; the count still guards against a
//! silent regression, but is not itself a claim that the rule is right.
//! [`Expected::Uncalibrated`] makes no claim about the count at all: what the rule checks
//! does not, or structurally cannot, describe a real property of a third-party repository's
//! own code, so a number here would be exactly what `done_when` warns against -- "a number
//! that matches whatever it currently does."
//!
//! # Two real capability-materialization defects, found and worked around while building this
//!
//! Both are about `root` -- the tree `Materialize_Dependencies`/`Materialize_Lint`/
//! `Materialize_Policy` hand to a real subprocess -- and neither is about this crate's own
//! rule logic.
//!
//! **`cargo clippy`/`cargo metadata` silently escape a nested fixture root.** Measured
//! directly: before `../hex-0.4.3/Cargo.toml` existed, running this file's own `Run` call
//! with `root` pointed at the fixture directory (which sits inside *this* repository's own
//! Cargo workspace) produced 174 `lint-diagnostics` findings about dozens of unrelated crates
//! under `crates/` -- `cargo clippy --workspace`, invoked with the fixture directory as its
//! working directory and no local manifest to stop there, walked upward and clippy'd *this
//! whole repository* instead of the two-file fixture. Giving the fixture its own `Cargo.toml`
//! declaring an empty `[workspace]` (stopping cargo's own upward manifest search at that
//! boundary) fixed this for both `dependency-direction`'s `cargo metadata` call and
//! `lint-diagnostics`'s `cargo clippy` call: `lint-diagnostics` now reports exactly one real,
//! fixture-scoped clippy diagnostic (see below), and `dependency-*`'s facts are real, scoped
//! `cargo metadata` output about the fixture's own one-member workspace.
//!
//! **`cargo deny`'s own config discovery is a *second*, independent escape the same fix does
//! not close.** Measured directly: with the `[workspace]` fix already in place,
//! `dependency-policy` still reported eight `license-not-encountered` advisories --
//! one per license this *repository's own* root `deny.toml` allows, none of which a
//! zero-dependency fixture could ever "encounter". `cargo deny`'s own manifest/config
//! discovery is independent of cargo's workspace-root resolution and is not scoped by
//! `nomos_lang_rust_deny::Materialize_Workspace` at all (`Cargo_Deny_Command` sets only
//! `working_directory`; no `--config` is ever passed) -- it walked upward past the fixture's
//! own workspace boundary and picked up this repository's real policy regardless.
//! `../hex-0.4.3/deny.toml`'s own module doc names this exactly, and giving the fixture a
//! minimal, permissive `deny.toml` of its own (mirroring what `../hex-0.4.3/standards.json`
//! already does for naming) fixed it the same way: `dependency-policy` now reports a real,
//! checked zero. **Nothing in this crate's own rule logic changed either time** -- both are
//! `nomos-lang-rust-clippy`/`nomos-lang-rust-deny`/`nomos-lang-rust-cargo`'s own materialization
//! failing to confine a real subprocess to `root`, worth a follow-up item in their own right
//! (flagged in this session's own final report; not filed here, since authoring a new ledger
//! item is outside this file's territory).
//!
//! # The naming-policy hypothesis, proven with real before/after numbers
//!
//! `naming.rs`'s own module doc says `OD-RULES-011` generalized `function-naming-convention`'s
//! `Pascal_Snake_Case`/`upper-snake` default onto a resolved read of a repository's own
//! `nomos.cap.naming.policy`. Measured directly, twice, over the identical fixture: with
//! `../hex-0.4.3/standards.json` deleted (temporarily, to measure -- it is restored in the
//! committed fixture), `function-naming-convention` reported **13** findings, one for every
//! real function and method the fixture declares (`encode_hex`, `decode_to_slice`, `val`,
//! `byte2hex`, ...), each carrying the exact hardcoded citation
//! `P45-RULES-CALIBRATED-AGAINST-CODE-THEY-WERE-NOT-TUNED-ON`'s own `why` names: "README.md's
//! Conventions section requires function names to be `Pascal_Snake_Case` ... nothing else was
//! checking it" -- *this* repository's README, describing *this* repository's convention,
//! cited against a fixture that is not this repository. With `standards.json` declaring
//! `{"naming": {"function": "lower-snake"}}` (this repository's own root `standards.json`, so
//! `Check_Naming_Convention`'s `Resolve_Case(facts, None, "function", ...)` call reads the
//! repository-wide row rather than a language-scoped one), the same 13 real names resolve
//! clean: **0** findings. The hypothesis holds, exactly as predicted, with no residual
//! findings and nothing further to fix in this rule for this fixture. The README-citation
//! wording this rule's `Finding::summary` still hardcodes is real (seen verbatim in the
//! `standards.json`-deleted run above) but is provably inert once a repository declares its
//! own convention -- it did not fire once in the fixture's own committed, correctly-configured
//! state, so this file makes no claim needing it fixed; flagged in this session's own report
//! as a latent wording defect for whichever future finding does reach it.

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::{CheckOutcome, Run, RunContext};
use nomos_contracts::Finding;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// The fixture's own root -- also `Materialize_Naming_Policy`/`Materialize_Dependencies`/
/// `Materialize_Lint`/`Materialize_Policy`'s own `root`, so its own `standards.json`,
/// `Cargo.toml` and `deny.toml` are what every capability this run materializes resolves
/// against, never this repository's.
fn Fixture_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest.join("fixtures").join("third-party").join("hex-0.4.3");
}

/// The fixture's own two files, read directly rather than walked: a fixture of exactly two
/// named files needs no walk at all, and `nomos_platform::FileSystem`'s `Read_Directory` is
/// one level rather than the recursive shape `nomos-cli::check::sources` documents.
/// need one.
fn Fixture_Sources() -> Vec<SourceFile>
{
    let root = Fixture_Root();
    let mut sources = Vec::new();
    for name in ["lib.rs", "error.rs"]
    {
        let text = std::fs::read_to_string(root.join(name)).expect("the fixture's own two files are committed and readable");
        sources.push(SourceFile::New(name, nomos_model::Subject_Of_Path(name), text));
    }
    return sources;
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// Runs the real, composed rule set over the fixture exactly once and files every finding
/// under the rule id that produced it -- one real `Run`, since three of its own
/// materializations launch a real subprocess (`cargo metadata`/`clippy`/`deny`) and doing
/// that once per rule assertion, over dozens of rules, would be both slow and a second, differently-
/// shaped run per rule rather than the one real run a caller like `nomos-cli check` performs.
fn Findings_By_Rule() -> BTreeMap<String, Vec<Finding>>
{
    let sources = Fixture_Sources();
    let root = Fixture_Root();

    let outcome = Run(
        &sources,
        RunContext {
            variant: Test_Variant(),
            root: &root,
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            environment: &StdEnvironment,
            workspace: &mut None,
            store: &mut MemoryFactStore::New(),
        },
        &[],
    );

    let CheckOutcome::Judged { findings, examined, claim } = outcome
    else
    {
        panic!("the fixture is a real, readable tree the syntax provider recognizes; a refusal here is a test-setup bug, not a caller-facing failure: {outcome:?}");
    };

    // Both fixture files must have a current syntax fact and nothing must be left
    // incomplete -- the same vacuity guard `nomos-cli::check` applies before trusting any
    // finding list at all.
    assert_eq!(examined, nomos_check_orchestration::Examined { files: 2, facts: 2 }, "both fixture files must be read and parsed");
    assert_eq!(claim, nomos_check_orchestration::Claim::Complete, "no subject may be left in a debt or agent-required state");

    let mut by_rule: BTreeMap<String, Vec<Finding>> = BTreeMap::new();
    for finding in findings
    {
        by_rule.entry(finding.rule.As_Str().to_owned()).or_default().push(finding);
    }
    return by_rule;
}

/// One rule's stated expected outcome against the fixture -- see this file's own module doc
/// for what each variant claims and does not claim.
enum Expected
{
    Clean,
    TruePositive(usize),
    /// Uninhabited since `P96` closed the one rule that used it, and kept rather than
    /// deleted: this is the state that lets a calibration author pin a real count without
    /// claiming the rule producing it is right, and without it the next author of an
    /// undeserved finding must choose between calling it a true positive, which is a lie,
    /// and `Uncalibrated`, which stops measuring the count at all. `#[expect]` rather than
    /// `#[allow]` on purpose -- when a rule lands here again, the attribute itself goes red
    /// rather than sitting on a variant that is no longer dead.
    #[expect(dead_code)]
    NotAllDeserved(usize),
    Uncalibrated,
}

/// One row of the per-rule verdict table: the rule this entry is about, its expected outcome,
/// and why -- printed on assertion failure so a regression names its own cause rather than
/// just a changed number.
struct RuleVerdict
{
    id: &'static str,
    expected: Expected,
    reason: &'static str,
}

/// One entry per rule [`nomos_check_orchestration::Composed_Rules`] currently composes, in
/// that function's own table order. [`Test_Table_Names_Exactly_The_Composed_Rule_Set`]
/// verifies the id sets are identical -- see this file's own module doc for the three
/// findings this table's own construction surfaced.
fn Verdicts() -> Vec<RuleVerdict>
{
    return vec![
        RuleVerdict { id: "completeness-mirror", expected: Expected::Clean, reason: "opt-in: judges a declared completeness-universe doc-comment claim (`OD-COMPLETENESS-001`'s own nomos-specific marker); the fixture's real rustdoc comments never use it, so there is nothing to reconcile -- a generalizable, unused mechanism, not a hardcoded nomos-only table." },
        RuleVerdict { id: "function-naming-convention", expected: Expected::Clean, reason: "the calibration's own centerpiece. Every real function/method (encode_hex, decode_to_slice, val, byte2hex, new, next, size_hint, len, from_hex, ...) is genuine lower_snake, and resolves clean ONLY because ../hex-0.4.3/standards.json declares {\"naming\":{\"function\":\"lower-snake\"}}. Measured directly with that file removed: 13/13 real names flagged, citing this repository's own README verbatim -- see this file's own module doc for the exact before/after counts and text." },
        RuleVerdict { id: "dependency-direction", expected: Expected::Uncalibrated, reason: "judges an edge against nomos_rules::checks::dependency::zones::ZONES, a table hardcoded to this workspace's own crate names (see dependency.rs's own module doc: \"zones is the one declaration\"). hex-fixture can never appear in it, so Violations_In silently has nothing to compare regardless of how the fixture is organized -- a 0 here would mean \"never checked\", not \"compliant\"." },
        RuleVerdict { id: "dependency-completeness", expected: Expected::Uncalibrated, reason: "the coverage half of the identical zones::ZONES table: reports \"hex-fixture has no declared zone\" for literally every non-nomos crate, regardless of how well-organized its real architecture is, since zones names only this workspace's own crates. The one real finding it does produce says nothing about this fixture's own dependency practices -- only that it is not nomos." },
        RuleVerdict { id: "dependency-write-authority", expected: Expected::Uncalibrated, reason: "checks whether an edge into one of nomos_rules::checks::dependency::write_authority::WRITE_DOORS (nomos's own private crates, e.g. nomos-store) comes from an authorized crate. A published third-party crate cannot even depend on one of these names, so this rule can structurally never fire against non-nomos code, positively or negatively." },
        RuleVerdict { id: "lint-diagnostics", expected: Expected::TruePositive(1), reason: "one real clippy::uninlined_format_args at error.rs:27 (write!(f, \"...{:?}...{}\", c, index) instead of write!(f, \"...{c:?}...{index}\")) -- clippy's own accurate, unmodified opinion about hex's real Display impl, reached only once ../hex-0.4.3/Cargo.toml's own [workspace] table stopped cargo clippy's upward escape into this repository (see this file's own module doc)." },
        RuleVerdict { id: "dependency-policy", expected: Expected::Clean, reason: "cargo deny check bans/licenses/sources against the fixture's own scoped deny.toml and its correctly declared `license = \"MIT OR Apache-2.0\"`: a real, checked zero, reached only once the fixture carried its own deny.toml (see this file's own module doc for the escape this closes and the 8-then-3-then-1-then-0 findings measured while closing it)." },
        RuleVerdict { id: "unread-reaches-finding", expected: Expected::Clean, reason: "heuristic over control-flow shapes specific to nomos's own fact-read-failure-must-reach-a-Finding idiom (nomos_cap_controlflow's ArmShape analysis); hex's code contains no fact-reading/Applicability-shaped control flow of that kind for the heuristic to examine." },
        RuleVerdict { id: "review-finding", expected: Expected::Uncalibrated, reason: "Materialize_Review() is unconditionally empty for every caller today -- no pull-request/comment discovery mechanism is wired to anything yet (ARC-CONNECTOR-001). A zero here reflects that nothing has ever been connected, not a judgment about this fixture." },
        RuleVerdict { id: "cross-language-correspondence", expected: Expected::Clean, reason: "opt-in: judges a `/// Corresponds to `Name`.` doc-comment marker (OD-CAPABILITY-010) declaring a Go counterpart struct. hex's real doc comments never declare one -- a generalizable, unused mechanism." },
        RuleVerdict { id: "no-trailing-whitespace", expected: Expected::Clean, reason: "the committed fixture text (copied from the registry cache, trimmed only of the excluded sections named in PROVENANCE.md) carries no trailing whitespace." },
        RuleVerdict { id: "todo-format-is-todo-name-description-ticket", expected: Expected::Clean, reason: "no TODO/FIXME marker anywhere in the fixture's real text." },
        RuleVerdict { id: "deprecation", expected: Expected::Clean, reason: "no #[deprecated] attribute in the fixture." },
        RuleVerdict { id: "a-rust-path-stays-within-its-own-subtree", expected: Expected::Clean, reason: "no #[path = \"...\"] attribute in the fixture." },
        RuleVerdict { id: "shared-interior-mutability-says-why", expected: Expected::Clean, reason: "no Rc<RefCell<...>>-shaped construct anywhere in the fixture." },
        RuleVerdict { id: "every-allow-carries-a-justification", expected: Expected::TruePositive(1), reason: "lib.rs:38's real #![allow(clippy::unreadable_literal)] (hex's own crate-level attribute, copied verbatim) has no adjacent comment of any kind -- Has_Local_Allow_Justification's own \"any non-empty comment satisfies it\" bar is not met by an absent one. A real, deserved finding against hex's own unmodified source." },
        RuleVerdict { id: "unsafe-justification", expected: Expected::Clean, reason: "no `unsafe` block anywhere in the fixture." },
        RuleVerdict { id: "scripts-use-a-portable-shebang", expected: Expected::Clean, reason: "not applicable: this rule judges shell/PowerShell/batch script sources, and the fixture is walked as exactly two named .rs files -- no script source is ever handed to this rule." },
        RuleVerdict { id: "a-script-declares-its-purpose", expected: Expected::Clean, reason: "not applicable, same reason as scripts-use-a-portable-shebang: no script source in the fixture's own source list." },
        RuleVerdict { id: "executed-scripts-set-nounset", expected: Expected::Clean, reason: "not applicable, same reason as scripts-use-a-portable-shebang." },
        RuleVerdict { id: "sleep-based-synchronization", expected: Expected::Clean, reason: "no `sleep`-based synchronization pattern in the fixture." },
        RuleVerdict { id: "zero-flake-policy", expected: Expected::Clean, reason: "the fixture carries no #[test] functions at all (PROVENANCE.md: hex's own dev-dependency test module was excluded), so there is nothing shaped like a retried test to examine -- a real but shallow zero, not evidence of disciplined test authorship one way or the other." },
        RuleVerdict { id: "no-mod-rs-files", expected: Expected::Clean, reason: "the fixture's files are lib.rs and error.rs, neither named mod.rs." },
        RuleVerdict { id: "a-credential-is-not-hardcoded-in-source", expected: Expected::Clean, reason: "no credential-shaped literal anywhere in the fixture." },
        RuleVerdict { id: "a-secret-does-not-travel-in-a-url", expected: Expected::Clean, reason: "no URL literal anywhere in the fixture." },
        RuleVerdict { id: "certificate-verification-is-not-disabled", expected: Expected::Clean, reason: "no certificate-verification code of any kind in the fixture." },
        RuleVerdict { id: "a-discarded-error-is-explained", expected: Expected::Clean, reason: "Go-only (Check_A_Discarded_Error_Is_Explained gates on source.Is_Written_In(GO_LANGUAGE) despite living in nomos-rules::checks::go_text): not applicable to a Rust-only fixture." },
        RuleVerdict { id: "a-skipped-test-states-why", expected: Expected::Clean, reason: "Go-only, same gate as a-discarded-error-is-explained: not applicable." },
        RuleVerdict { id: "an-excluded-file-says-why", expected: Expected::Clean, reason: "Go-only, same gate as a-discarded-error-is-explained: not applicable." },
        RuleVerdict { id: "suppression-directives-carry-a-reason", expected: Expected::Clean, reason: "Go-only (judges `//nolint` directives), same gate as a-discarded-error-is-explained: not applicable." },
        RuleVerdict { id: "workspace-markers-carry-a-reason", expected: Expected::Clean, reason: "Go-only, same gate as a-discarded-error-is-explained: not applicable." },
        RuleVerdict { id: "a-package-is-named-after-its-directory", expected: Expected::Clean, reason: "Go-only despite its language-agnostic-sounding name (Check_A_Package_Is_Named_After_Its_Directory returns None immediately for a non-Go source): never reads Cargo.toml or the fixture's own directory name at all, so the real `hex-fixture` vs `hex-0.4.3` naming mismatch is never even examined by this rule." },
        RuleVerdict { id: "atomic-ordering-choices-are-justified", expected: Expected::Clean, reason: "no atomic type or Ordering:: usage anywhere in the fixture." },
        RuleVerdict { id: "seqcst-justified-explicitly", expected: Expected::Clean, reason: "no Ordering::SeqCst usage in the fixture." },
        RuleVerdict { id: "relaxed-not-used-when-ordering-matters", expected: Expected::Clean, reason: "no Ordering::Relaxed usage in the fixture." },
        RuleVerdict { id: "data-names-stay-lower-snake", expected: Expected::Clean, reason: "judges module/field names against Case::LowerSnake, which is this rule's own hardcoded default (data_names.rs's Resolve_Case calls already pass Case::LowerSnake, not nomos's own upper-snake) -- already the real, idiomatic Rust convention hex's one real module (error) and its struct fields (inner, table, next) already follow, with no fixture-side override needed." },
        RuleVerdict { id: "file-name-matches-declared-type", expected: Expected::TruePositive(1), reason: "error.rs declares the public type `FromHexError` but is named for what it holds generically rather than for that type -- file_names.rs's own Comparable_Stem exempts lib/main/mod but not error, and this file exports no free public function alongside the type (Declares_A_Public_Operation is false), so the Declares_A_Public_Operation exemption OD-RULES-015 carved out (a module named for what it does) does not apply either. nomos's own tree renames this exact shape to `<type>_error.rs` (clippy_error.rs, metadata_error.rs, deny_error.rs); hex's `error.rs` is a real, ordinary difference in file-naming convention between the two projects, correctly caught." },
        RuleVerdict { id: "constants-split-by-export", expected: Expected::Clean, reason: "Go-only: not applicable to a Rust-only fixture." },
        RuleVerdict { id: "variables-use-lower-snake-case", expected: Expected::Clean, reason: "Go-only: not applicable." },
        RuleVerdict { id: "exported-functions-use-upper-snake-case", expected: Expected::Clean, reason: "Go-only: not applicable." },
        RuleVerdict { id: "unexported-functions-lowercase-only-the-first-letter", expected: Expected::Clean, reason: "Go-only: not applicable." },
        RuleVerdict { id: "types-use-upper-camel-case-lower-camel-case", expected: Expected::Clean, reason: "Go-only: not applicable." },
        RuleVerdict { id: "parameter-count", expected: Expected::Clean, reason: "every real function in the fixture takes at most 2 value parameters (e.g. decode_to_slice(data, out), val(c, idx)), well under the default 4-parameter ceiling (5 with an allowed receiver)." },
        RuleVerdict { id: "go-helpers-package-five-inputs", expected: Expected::Clean, reason: "Go-only: not applicable." },
        RuleVerdict { id: "declared-tooling-language-for-scripts", expected: Expected::Clean, reason: "no script source in the fixture's own source list (same reason as scripts-use-a-portable-shebang)." },
        RuleVerdict { id: "1500-lines", expected: Expected::Clean, reason: "lib.rs is roughly 330 lines, far under the default 1500-line justification-trigger ceiling." },
        RuleVerdict { id: "nonnegative-storage-is-unsigned", expected: Expected::Clean, reason: "opt-in: only examines a field preceded by a #[validate(range(min = _, max = _))] attribute (nomos's own validation-attribute convention). The fixture uses no such attribute anywhere -- a generalizable, unused mechanism, composed 2026-09-06 after being measured at zero findings against this repository's own tree." },
        RuleVerdict { id: "a-known-range-picks-its-type", expected: Expected::Clean, reason: "the same #[validate(range(...))] opt-in as nonnegative-storage-is-unsigned, over the fixture's own text: nothing to examine." },
        RuleVerdict { id: "named-fields-over-positional-variant-payloads", expected: Expected::Clean, reason: "real, non-vacuous: error.rs's FromHexError::InvalidHexCharacter { c: char, index: usize } is a genuine multi-field enum variant, and it already uses named-field form -- exactly what this rule wants and nothing this rule's own tuple-variant text scan (Tuple_Variant_Match) matches. The fixture's other two variants (OddLength, InvalidStringLength) carry no payload at all." },
        RuleVerdict { id: "one-thousand-line-hard-trigger", expected: Expected::Clean, reason: "Go-only: not applicable." },
        RuleVerdict { id: "five-hundred-line-review-trigger", expected: Expected::Clean, reason: "Go-only: not applicable." },
        RuleVerdict { id: "lowercase-first-letter", expected: Expected::Clean, reason: "only judges a #[error(\"...\")] thiserror-style attribute's own message text (error_text.rs's own module doc: judging every write!/writeln! would convict ordinary logging). hex's FromHexError implements Display by hand with write!/writeln! and carries no #[error(...)] attribute at all, so this rule has nothing in its own deliberately narrow scope to examine here -- a real design boundary, not a gap, and not exercised by this fixture's own shape." },
        RuleVerdict { id: "no-trailing-punctuation", expected: Expected::Clean, reason: "same #[error(\"...\")]-only scope as lowercase-first-letter: nothing in the fixture for it to examine." },
        RuleVerdict { id: "eager-vs-lazy-context", expected: Expected::Clean, reason: "looks for a literal `.With_Context(` call (nomos's own Pascal_Snake-cased helper name, not the real ecosystem's lowercase anyhow/eyre `.with_context(`/`.context(`); the fixture has no error-context-chaining code in any casing, so this is a real zero either way. Worth flagging separately: this hardcoded spelling would also miss the real anti-pattern in ordinary third-party code using the standard lowercase method name, which this fixture happens not to exercise." },
        RuleVerdict { id: "goals-and-parts-line-up", expected: Expected::Clean, reason: "opt-in: judges a repository's own declared nomos.cap.goals.policy (standards.json's \"goals\" block); the fixture declares none, and Check_Goals_And_Parts_Line_Up's own doc states a repository that has not written one down has not opted in -- a generalizable, unused mechanism, not a hardcoded nomos-only table." },
        RuleVerdict { id: "requirement-trace-staleness", expected: Expected::Clean, reason: "opt-in: judges a repository's own tests/contract/requirements/*.assessment corpus; the fixture has none, which Discover_Workspace's own doc treats identically to \"declares none\" or \"every entry resolves\" -- a generalizable, unused mechanism." },
        RuleVerdict { id: "abbreviations", expected: Expected::TruePositive(1), reason: "1 finding, deserved. `val` (the real, private function fn val(c, idx) -> Result<u8, FromHexError>) is a genuine, deserved true positive: DEFAULT_BANNED_WORDS lists \"val\" verbatim (ported from code-standards' own defaults.go) and hex's author really did choose that terse name. `alloc` -- the name of `extern crate alloc;`, fixed by the real crate being linked, not chosen at this site -- was a real false positive when this verdict was first written (`P68-ABBREVIATIONS-DOES-NOT-EXEMPT-EXTERN-CRATE`); Is_Exempt now recognizes ItemKind::ExternCrate the same way it already recognized ItemKind::Use, and this fixture is what proved the fix." },
        RuleVerdict { id: "single-letter-names", expected: Expected::Clean, reason: "a real, checked zero, and it was 1-finding-0-deserved until P96. `T` was never a struct field or an ordinarily-declared item name a human carelessly abbreviated -- it is `impl<T: AsRef<[u8]>> ToHex for T`'s own Self type, and nomos_lang_rust::syntax::walk's visit_item_impl records an Implementation item's `name` as `Type_Head(&node.self_ty)`, which for this common, idiomatic blanket-impl-over-a-generic-parameter shape is literally the generic parameter's own already-declared name. The gap took two increments and not one: P68 was DECLINED because the payload carried no fact that could tell this apart from `impl Trait for X` over a real, single-letter-named struct somebody chose, and a name-only heuristic would have hidden that second, deserved case. OD-CAPABILITY-014 decided the extension, P96-AN-IMPL-BLOCKS-GENERIC-PARAMETERS-REACH-THE-SYNTAX-PAYLOAD built it (Impl_Shape now carries the block's own type parameters, read back by Impl_Generics), and the rule now exempts an Implementation item whose own name is one of them -- still reporting one whose name is not. This fixture is what surfaced the gap, by using an entirely ordinary Rust idiom nomos's own tree does not happen to write, and is what now measures it closed." },
        RuleVerdict { id: "a-disabled-test-states-why", expected: Expected::Clean, reason: "the fixture carries no #[test] functions at all (see zero-flake-policy's identical note), so there is no #[ignore] attribute for this rule to examine either." },
        RuleVerdict { id: "inline-always-requires-justification", expected: Expected::Clean, reason: "no #[inline(always)] attribute in the fixture (only plain #[inline], which this rule does not judge)." },
        RuleVerdict { id: "no-wildcard-imports", expected: Expected::Clean, reason: "every `use` in the fixture names what it imports (core::iter, alloc::{string::String, vec::Vec}, core::fmt); no `use ...::*;` anywhere." },
        RuleVerdict { id: "no-single-line-function-bodies", expected: Expected::Clean, reason: "no function in the fixture is written as a single-line body in the shape this rule judges." },
        RuleVerdict { id: "no-orphan-modules", expected: Expected::Clean, reason: "lib.rs's own `mod error;` is backed by a real, read error.rs -- no orphaned module declaration." },
        RuleVerdict { id: "parameters-borrow-unless-ownership-is-taken", expected: Expected::Clean, reason: "the fixture's few owned-by-value parameters are all generic (`T: AsRef<[u8]>`), the shape this rule's own text scan does not flag; no concrete owned type (String/Vec/...) is taken by value where a borrow would do." },
        RuleVerdict { id: "lifetimes-follow-the-descriptive-naming-rule", expected: Expected::Clean, reason: "every lifetime-carrying declaration in the fixture (struct BytesToHexChars<'a>, impl<'a> ... for BytesToHexChars<'a>) names exactly one lifetime, and this rule's own LIFETIMES_NEEDING_NAMES gate (2 or more distinct lifetimes on one declaration) is the documented, deliberate reason a lone 'a is exempt: \"where there is nothing to tell apart, 'a names the only borrow there is\". Correctly, not vacuously, exercised: the fixture is exactly the common single-lifetime-named-'a shape this rule is designed to leave alone." },
        RuleVerdict { id: "static-bounds-are-justified", expected: Expected::Clean, reason: "the fixture's only `'static` usage (`table: &'static [u8; 16]`) is a reference's own lifetime, not a trait bound (`T: 'static`); Bounds_By_Static's own scan only recognizes the latter, and its own doc states a `&'static` reference is deliberately not this rule's subject. Correctly, not vacuously, exercised." },
        RuleVerdict { id: "prefer-macro-rules-over-procedural-macros", expected: Expected::Clean, reason: "the fixture's one macro (from_hex_array_impl!) is declarative macro_rules! -- the approved form this rule prefers -- and there is no proc-macro definition anywhere in the fixture. A real, meaningfully-exercised clean." },
        RuleVerdict { id: "nesting-depth", expected: Expected::Clean, reason: "every function body in the fixture stays within a shallow, ordinary nesting depth (at most a for-loop over one if/match)." },
        RuleVerdict { id: "closure-bounds-are-minimal", expected: Expected::Clean, reason: "judges an inline Fn*-bound parameter on a `pub fn` line, or a closure bound explicitly widened with Send/Sync/'static; the fixture's closures (plain `.map(|byte| {...})` calls) carry no explicit trait bound of any kind for this rule to examine." },
        RuleVerdict { id: "boxed-closures-are-justified-and-off-hot-paths", expected: Expected::Clean, reason: "no Box<dyn Fn*>/Arc<dyn Fn*>/Rc<dyn Fn*> anywhere in the fixture." },
    ];
}

#[test]
fn Test_Table_Names_Exactly_The_Composed_Rule_Set()
{
    let table: Vec<&str> = Verdicts().iter().map(|verdict| return verdict.id).collect();
    let table_set: BTreeSet<&str> = table.iter().copied().collect();
    assert_eq!(table.len(), table_set.len(), "the verdict table names the same rule id twice");

    let composed = nomos_check_orchestration::Composed_Rules();
    let composed_set: BTreeSet<String> = composed.iter().map(|id| return id.As_Str().to_owned()).collect();
    let table_owned: BTreeSet<String> = table_set.iter().map(|id| return (*id).to_owned()).collect();

    assert_eq!(
        table_owned, composed_set,
        "this file's own verdict table must name exactly the rules nomos_check_orchestration::Composed_Rules() \
         currently composes -- a rule added or removed there must gain or lose an entry here"
    );
}

/// The full per-rule calibration: every entry in [`Verdicts`] checked against one real run
/// of the composed rule set over the fixture. See this file's own module doc for the
/// fixture's provenance, the two capability-materialization defects found while building it,
/// and the naming-policy before/after measurement.
#[test]
fn Test_Composed_Rules_Against_An_Idiomatic_Third_Party_Fixture()
{
    let by_rule = Findings_By_Rule();

    for verdict in Verdicts()
    {
        let empty: Vec<Finding> = Vec::new();
        let findings = by_rule.get(verdict.id).unwrap_or(&empty);

        match verdict.expected
        {
            Expected::Clean =>
            {
                assert!(findings.is_empty(), "{}: expected zero findings ({}), got {findings:?}", verdict.id, verdict.reason);
            }
            Expected::TruePositive(count) =>
            {
                assert_eq!(findings.len(), count, "{}: expected {count} true-positive finding(s) ({}), got {findings:?}", verdict.id, verdict.reason);
            }
            Expected::NotAllDeserved(count) =>
            {
                assert_eq!(
                    findings.len(), count,
                    "{}: expected {count} finding(s), not all deserved ({}), got {findings:?}", verdict.id, verdict.reason
                );
            }
            Expected::Uncalibrated =>
            {
                // No assertion: this rule's own expected verdict cannot be honestly stated
                // today (see `reason`), so no count here would mean what it looks like it
                // means. `Test_Table_Names_Exactly_The_Composed_Rule_Set` still guards that
                // this entry exists and stays paired with a real, composed rule id.
                let _ = verdict.reason;
            }
        }
    }
}
