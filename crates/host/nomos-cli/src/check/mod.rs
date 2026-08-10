//! `nomos check` — run the rules over a tree and report what they find.
//!
//! The composition root for `nomos-rules`. That crate takes its subject as an argument
//! and deliberately cannot read a disk; this module is where the tree is decided, walked
//! and handed over, which is the seam that lets the same rule be run against the real
//! workspace here and against code that no longer exists in a test.
//!
//! # What a composition root has to assemble now
//!
//! Four things, and none of them is decoration. `OD-RULES-001` moved check-name
//! resolution onto the fact layer, so this command:
//!
//! 1. ingests the walk through [`Workspace::Apply`], the one door a workspace state comes
//!    through, which is where the snapshot and generation every fact is filed under come
//!    from;
//! 2. declares [`nomos_cap_syntax::Capability_Contract`] and registers
//!    [`nomos_lang_rust::Provider_Offer`] — a *floor* is stated by the rule and the
//!    registry decides who serves it, so this root registers providers and does not pick
//!    one;
//! 3. materializes one syntax fact per file into a [`MemoryFactStore`];
//! 4. runs the rule over a [`Reader`] on that store.
//!
//! `nomos-lang-rust-scan` is not registered. It offers the same capability at
//! `FactVariant::Approximate` and `Assurance::Unsound`, which is below
//! `nomos_rules::Syntax_Requirement`; registering it would make the refusal a runtime
//! event nobody sees rather than a decision written down here.
//!
//! # The vacuity guard lives here, and it now has two shapes
//!
//! A rule that finds no subjects returns no findings, which renders identically to a
//! clean run. That is the defect the sibling workspace recorded four times over —
//! `check-standards-tree /nonexistent` walked nothing, found nothing and reported CLEAN —
//! and `boundaries.rs` already guards its own assertions against it. The guard belongs
//! here rather than in the rule, because "did I see a plausible amount of the world" is a
//! question only the caller that chose the tree can answer.
//!
//! The second shape arrived with the fact layer: a walk that found source and materialized
//! no fact for any of it. That is the same lie one layer in — the rule would resolve every
//! mirror claim against an empty index and, because the index is empty, would refuse to
//! block on any of them. Both exit [`ExitCode::Vacuous`], which already means "the answer
//! is empty because something expected was not there". No seventh code: an exit code means
//! one thing per binary, and this is that thing.
//!
//! Where the vacuity guard *belongs* is a live question — `P10-VACUITY-HOME` holds it, and
//! a rule that can now report "I could not run" is evidence for that item rather than an
//! answer to it.
//!
//! # This command is a gate step now, and that changes what an exit code costs
//!
//! `OD-GATE-004` wired the `Rules` step of `.github/workflows/gate.yml` to
//! `cargo run --quiet -p nomos-cli --bin nomos -- check --root .`. Until then nothing in
//! CI ran this command, so the rule layer enforced nothing: a broken walk, a provider that
//! stopped answering, a rotted argv or a panic out of [`Registered`] all shipped green.
//!
//! **Zero is the only success, and the workflow says so by containing no branch.** Actions
//! fails a step on any non-zero exit, and that default *is* the policy — so the numbers
//! below stay spelled here, once, rather than being restated in YAML where they would go
//! stale against this enum. What each code now does to a pull request:
//!
//! - [`ExitCode::Ok`] — the rules ran over the workspace, materialized facts, and nothing
//!   they found can fail a build. Not "found nothing": twelve admitted gaps and the two
//!   `tests/corpus/analysis/gamma/broken.rs` advisories print on every run, and the counts
//!   line carries both denominators. **Green.**
//! - [`ExitCode::Violations`] — the arm the step exists for. Reachable only since
//!   `OD-RULES-002` made incompleteness a property of the claim rather than of the run;
//!   before that a phantom anywhere in this workspace was downgraded by `broken.rs` and the
//!   step could not have failed for its own reason. **Red.**
//! - [`ExitCode::Usage`] — from a step, this means *the workflow's own argv is wrong*. A
//!   gate that mistypes its invocation and passes is a gate checking nothing. **Red.**
//! - [`ExitCode::Unreadable`] — no tree, so nothing was checked. **Red.**
//! - [`ExitCode::Vacuous`] — the one this wiring is really about. A gate treating "I judged
//!   nothing" as success would be `OD-GATE-001`'s defect installed one level up from where
//!   that record found it, this time with a green tick beside it. **Red.**
//! - anything else — `101` from a build failure or from [`Registered`]'s own `expect`, a
//!   signal, a truncation. Absence, unknown and error must not become success, and the
//!   default gives that for free. **Red.**
//!
//! Two consequences for anybody editing this module. A sixth code must survive
//! `main`'s `u8::try_from(code).unwrap_or(1)`, or a distinct outcome arrives at CI as an
//! ordinary violation. And [`ExitCode::Vacuous`] must never be renumbered to `0` "because
//! there is nothing to report" — `Test_Only_Ok_Should_Carry_The_Passing_Exit_Code` below is
//! the whole exit-code policy as one assertion, and it is where that would go red.
//!
//! No corpus is involved. This command reads no environment variable at run time —
//! `main.rs` passes it none — so a CI runner with no `NOMOS_*` set produces the full
//! answer. Measured. What it does need is what `build.rs` baked in for
//! [`Host_Variant`], and a runner that cannot supply those cannot link the binary and
//! fails at `Lint` long before this step.

mod exit_code;
mod check_command;

pub use exit_code::ExitCode;
pub use check_command::CheckCommand;

use crate::arguments::Named_Value;
use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{CapabilityId, ConfigurationId, Finding, Guarantee};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_model::{Content_Digest, Subject_Of_Path};
use nomos_rules::{Check_Completeness_Mirrors, SourceFile};
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: nomos check [--root <path>]\n\n\
     Runs every rule over the tree and reports what they find.\n\n\
     rules:\n  \
     completeness-mirror   a declared universe must name a check that compares it\n\
     \x20                     against the reality it enumerates, and that check must\n\
     \x20                     exist. See OD-COMPLETENESS-001.\n\n\
     exit codes: 0 nothing blocking, 1 findings that can fail a build, 2 usage,\n\
     \x20           5 unreadable tree, 6 nothing was judged";

/// Parses the group's arguments.
///
/// # Errors
///
/// Returns the usage message when an argument is not understood.
pub fn Parse(arguments: &[String]) -> Result<CheckCommand, String>
{
    if let Some(unknown) = arguments
        .iter()
        .find(|argument| return argument.starts_with('-') && argument.as_str() != "--root")
    {
        return Err(format!("unknown argument `{unknown}`.\n\n{USAGE}"));
    }

    return Ok(CheckCommand {
        root: Named_Value(arguments, "--root").map_or_else(|| return PathBuf::from("."), PathBuf::from),
    });
}

/// How much of the world this run actually saw.
///
/// Two denominators and not one. "0 findings over 400 files" and "0 findings over 400
/// files none of which produced a fact" are different claims, and the second is a broken
/// run — the prototype reported the first shape for a check that had walked nothing, and
/// the defect was invisible because the report had no place to put the number that would
/// have shown it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Examined
{
    /// Files the walk read.
    files: usize,
    /// Files a syntax fact was materialized for.
    facts: usize,
}

/// Runs the rules and renders what they say.
pub fn Run(command: &CheckCommand, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    let prepared = match Prepare(&command.root, stderr)
    {
        Ok(prepared) => prepared,
        Err(code) => return code,
    };

    let mut store = MemoryFactStore::New();
    let facts = Materialize_Syntax(&prepared.sources, &prepared.context, &mut store);
    if facts == 0
    {
        return Nothing_Materialized(prepared.sources.len(), &command.root, stderr);
    }

    let mut reader = Reader::On(&store, &prepared.registry, prepared.context);
    let findings = Check_Completeness_Mirrors(&prepared.sources, &mut reader);
    let examined = Examined {
        files: prepared.sources.len(),
        facts,
    };

    return Report(&findings, examined, stdout);
}

/// Everything the run needs before it can judge anything.
struct Prepared
{
    sources: Vec<SourceFile>,
    registry: Registry,
    context: Context,
}

/// Walks the tree and ingests it as one workspace state.
fn Prepare(root: &Path, stderr: &mut impl Write) -> Result<Prepared, ExitCode>
{
    let sources = Walked(root, stderr)?;
    let registry = Registered();
    let context = Ingested(&sources, root, &registry, stderr)?;

    return Ok(Prepared {
        sources,
        registry,
        context,
    });
}

/// The Rust sources under the root.
///
/// A walk that found nothing is a refusal rather than a clean result: a run that judged no
/// file renders exactly like a run that judged every file and found nothing to say.
fn Walked(root: &Path, stderr: &mut impl Write) -> Result<Vec<SourceFile>, ExitCode>
{
    if !root.is_dir()
    {
        let _ignored = writeln!(stderr, "cannot read `{}`: not a directory", root.display());

        return Err(ExitCode::Unreadable);
    }

    let sources = Read_Sources(root);
    if sources.is_empty()
    {
        let _ignored = writeln!(
            stderr,
            "no Rust source found under `{}`, so nothing was judged.\n\
             A clean result here would mean only that the walk found nothing.",
            root.display()
        );

        return Err(ExitCode::Vacuous);
    }

    return Ok(sources);
}

/// The walk applied to an empty workspace, and the context every fact is filed under.
fn Ingested(
    sources: &[SourceFile],
    root: &Path,
    registry: &Registry,
    stderr: &mut impl Write,
) -> Result<Context, ExitCode>
{
    let configuration = Resolved_Configuration(registry);
    let variant = Host_Variant();
    let mut workspace = Workspace::Empty(variant.clone(), configuration);
    let checkout = As_One_Checkout(sources);

    let applied = workspace.Apply(&checkout).map_err(|error| {
        let _ignored = writeln!(
            stderr,
            "the walk of `{}` could not be ingested as a workspace state: {error:?}",
            root.display()
        );

        return ExitCode::Unreadable;
    })?;

    return Ok(Context {
        snapshot: applied.Snapshot(),
        variant: variant.Id(),
        configuration,
        generation: applied.Generation(),
    });
}

/// The whole walk as one change set, because a walk is one event.
///
/// Applying a file at a time would produce one generation per file, and every intermediate
/// one would describe a tree that never existed.
fn As_One_Checkout(sources: &[SourceFile]) -> WorkspaceChangeSet
{
    let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);

    for source in sources
    {
        checkout = checkout.Present(source.path.clone(), source.text.clone());
    }

    return checkout;
}

/// A walk that read source and produced no fact from any of it.
///
/// The same lie as an empty walk, one layer in: the rule would resolve every mirror claim
/// against an empty index and, because the index is empty, refuse to block on any of them.
fn Nothing_Materialized(read: usize, root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "{read} file(s) were read under `{}` and no syntax fact was materialized for any \
         of them, so no mirror claim could be resolved.\n\
         A clean result here would mean only that the analysis never ran.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// The capability this run declares and the providers it admits.
///
/// # Panics
///
/// If the registry refuses a declaration or an offer. Both are decided by this function's
/// own constants, so a refusal is a contradiction in the composition rather than a runtime
/// condition, and continuing past it would produce a run whose facts nobody offered.
fn Registered() -> Registry
{
    let mut registry = Registry::New();

    registry
        .Declare(nomos_cap_syntax::Capability_Contract())
        .expect("the syntax capability is declared once");
    registry
        .Offer(nomos_lang_rust::Provider_Offer())
        .expect("the Rust provider's offer is within its capability's ceiling");

    return registry;
}

/// The build variant this binary was compiled as.
///
/// Every component is captured by `build.rs` from cargo's own environment, because none of
/// them survives into the compiled program.
fn Host_Variant() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES")
            .split(',')
            .filter(|feature| return !feature.is_empty()),
    );
}

/// The identity of this run's effective policy.
///
/// # Why the registry is the configuration
///
/// [`ConfigurationId`] is documented as a digest of a fully resolved effective policy, and
/// for an analysis run the resolved policy *is* which capabilities are declared and which
/// providers may answer for them, at what versions and under what guarantees — which is
/// exactly what a [`Registry`] holds once composition is finished. Inventing a second
/// policy object beside it would give the run two answers to what it is configured to do.
///
/// Line-oriented, tab-separated and hand-written, because nothing derived may sit between
/// the data and its digest: a `Debug` implementation changing its spacing in a point
/// release would re-address every fact this run produces.
///
/// This is the second rendering of a registry in the workspace;
/// `tests/integration/src/context.rs` has the other, at band 100 where this crate cannot
/// reach it. Two renderings of one policy is a real duplication and it is not this item's
/// to remove.
fn Resolved_Configuration(registry: &Registry) -> ConfigurationId
{
    let mut rendered = String::from("nomos.check.configuration.v1\n");

    for contract in registry.Declared()
    {
        rendered.push_str("capability\t");
        rendered.push_str(contract.id.As_Str());
        rendered.push('\t');
        rendered.push_str(&contract.version.to_string());
        rendered.push('\t');
        rendered.push_str(&Rendered_Guarantee(contract.ceiling));
        rendered.push('\n');
        Render_Offers(&mut rendered, registry, &contract.id);
    }

    return ConfigurationId::From_Digest(Content_Digest(rendered.as_bytes()));
}

/// Every offer standing against one capability, each on its own line.
///
/// The offers are part of the configuration and not only the declarations, because the same
/// floor served by a different provider is a different composition and has to hash apart.
fn Render_Offers(rendered: &mut String, registry: &Registry, capability: &CapabilityId)
{
    for offer in registry.Offers(capability)
    {
        rendered.push_str("offer\t");
        rendered.push_str(capability.As_Str());
        rendered.push('\t');
        rendered.push_str(offer.provider.As_Str());
        rendered.push('\t');
        rendered.push_str(&offer.version.to_string());
        rendered.push('\t');
        rendered.push_str(&Rendered_Guarantee(offer.guarantee));
        rendered.push('\n');
    }
}

/// A guarantee as one field, by its four stable labels.
fn Rendered_Guarantee(guarantee: Guarantee) -> String
{
    return format!(
        "{}/{}/{}/{}",
        guarantee.variant.Label(),
        guarantee.soundness.Label(),
        guarantee.completeness.Label(),
        guarantee.incremental.Label()
    );
}

/// Produces one syntax fact per source and returns how many were written.
///
/// A file the provider refuses materializes nothing and is not dropped silently: the count
/// returned is the denominator the report prints beside the file count, and the rule's own
/// unread-subject finding names each one individually. Two independent readings of one
/// file — this provider's and `nomos-rules`' own universe parser — can refuse
/// independently, and the run is entitled to see which.
fn Materialize_Syntax(
    sources: &[SourceFile],
    context: &Context,
    store: &mut MemoryFactStore,
) -> usize
{
    let production = Production(context);
    let mut written = 0_usize;

    for source in sources
    {
        let Materialization::Materialized(fact) =
            nomos_lang_rust::Materialize(source.subject, &source.text, production)
        else
        {
            continue;
        };

        // No dependency edges: a syntax fact is a leaf, read from one file's bytes and
        // from nothing this store holds.
        if store.Materialize(*fact, &[]).is_ok()
        {
            written = written.saturating_add(1);
        }
    }

    return written;
}

/// The reading context as the provider takes it.
fn Production(context: &Context) -> FactContext
{
    return FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
}

/// Renders the findings and decides the exit code.
fn Report(findings: &[Finding], examined: Examined, stdout: &mut impl Write) -> ExitCode
{
    for finding in findings
    {
        let _ignored = writeln!(stdout, "{}", finding.Describe());
    }

    let blocking = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .count();
    Counts(findings.len(), blocking, examined, stdout);

    if blocking > 0
    {
        return ExitCode::Violations;
    }

    return ExitCode::Ok;
}

/// What was looked at, which is part of the result and not decoration.
///
/// "0 findings" over 4 files and "0 findings" over 400 are different claims, and so are
/// "400 files" and "400 files, 12 of which produced a fact" — a reader who cannot tell
/// them apart cannot tell a clean tree from a broken walk.
fn Counts(found: usize, blocking: usize, examined: Examined, stdout: &mut impl Write)
{
    let _ignored = writeln!(
        stdout,
        "\n{} file(s) examined, {} with a syntax fact, {found} finding(s), {blocking} of \
         which can fail a build",
        examined.files,
        examined.facts
    );
}

/// Every `.rs` file under `root`, with its text and the subject its facts are filed under.
///
/// `target` is skipped: it holds generated source that nobody authored, and judging a
/// build artifact would report findings against code the author cannot edit.
fn Read_Sources(root: &Path) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path = entry.path();
            Read_Entry(root, path, &mut pending, &mut sources);
        }
    }

    sources.sort_by(|left, right| return left.path.cmp(&right.path));
    return sources;
}

/// One entry of a walked directory: queued if it is a directory worth descending into,
/// read if it is a `.rs` file, and ignored otherwise.
fn Read_Entry(
    root: &Path,
    path: PathBuf,
    pending: &mut Vec<PathBuf>,
    sources: &mut Vec<SourceFile>,
)
{
    if path.is_dir()
    {
        let skipped = path
            .file_name()
            .is_some_and(|name| return name == "target" || name == ".git");

        if !skipped
        {
            pending.push(path);
        }

        return;
    }

    if path.extension().is_some_and(|extension| return extension == "rs")
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        sources.push(source);
    }
}

/// One `.rs` file as the rule takes it.
///
/// This root files a fact under the subject and hands the same value to the rule on
/// `SourceFile::subject`, so the two cannot disagree about addressing. It is the kernel's
/// rule and not a local one, which is what keeps that agreement from being a coincidence —
/// see `OD-MODEL-001`.
fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    let relative = Relative(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
///
/// Forward slashes on every platform, because a finding's location appears in output that
/// gets pasted between machines, and the same file must not render two ways.
fn Relative(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, GateCategory};

    /// The composition this command really ships, over sources a test wrote by hand.
    ///
    /// Not a stub. The registry holds the real contract and the real parser's real offer,
    /// and the store holds facts the real provider produced — which is what makes every
    /// assertion below a statement about the shipped binary rather than about a fixture.
    struct Composed
    {
        registry: Registry,
        store: MemoryFactStore,
        context: Context,
    }

    impl Composed
    {
        fn Over(sources: &[SourceFile]) -> Self
        {
            let registry = Registered();
            let mut refused = Vec::new();
            let context = Ingested(sources, Path::new("."), &registry, &mut refused)
                .expect("the fixture is a valid tree");
            let mut store = MemoryFactStore::New();
            let _written = Materialize_Syntax(sources, &context, &mut store);

            return Self {
                registry,
                store,
                context,
            };
        }

        fn Findings(&self, sources: &[SourceFile]) -> Vec<Finding>
        {
            let mut reader = Reader::On(&self.store, &self.registry, self.context.clone());

            return Check_Completeness_Mirrors(sources, &mut reader);
        }
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        return SourceFile::New(path, Subject_Of_Path(path), text);
    }

    /// ---- the exit-code policy the gate step rests on ----
    ///
    /// Every code this group can leave the process with, written twice on purpose.
    ///
    /// The array is what the assertions below iterate. [`Labelled`] is an exhaustive `match`,
    /// so a variant added to [`ExitCode`] fails to compile *there* — which is the only
    /// mechanism available without a derive that a code-adder cannot walk past, and it stops
    /// them inside the function they have to extend. What the match does not force is adding
    /// the new code to this array; that residual is closed from the other side by
    /// `Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With`, which
    /// compares the array against the usage text a person reads.
    ///
    /// # Why this is not `ExitCode::All()`
    ///
    /// That was the tidier shape and it was tried. An `All()` in an inherent implementation is
    /// a *declared universe* — `nomos-rules` finds it by that exact name — so the enum this
    /// gate step's policy rests on would need a row in
    /// `tests/contract/tests/completeness_universes.rs` saying what compares the list against
    /// the reality it enumerates. Measured: without that row,
    /// `Test_The_Declared_Table_Should_Match_What_Is_Derived` and
    /// `Test_The_Scan_And_The_Table_Should_Name_The_Same_Mirror` go red naming
    /// `ExitCode::All`, and the scanned total goes from sixteen universes to seventeen. That
    /// file is outside `P10-CHECK-GATE`'s territory, so the census stays private here, where
    /// it is not a universe at all. Promoting it is worth doing by whoever holds that file
    /// next — the mechanism refusing an unclassified list is the rule working, not an
    /// obstacle.
    fn Every_Exit_Code() -> [ExitCode; 5]
    {
        return [
            ExitCode::Ok,
            ExitCode::Violations,
            ExitCode::Usage,
            ExitCode::Unreadable,
            ExitCode::Vacuous,
        ];
    }

    /// A code's name, as an exhaustive match, so that adding one stops the build here.
    fn Labelled(code: ExitCode) -> &'static str
    {
        return match code
        {
            ExitCode::Ok => "Ok",
            ExitCode::Violations => "Violations",
            ExitCode::Usage => "Usage",
            ExitCode::Unreadable => "Unreadable",
            ExitCode::Vacuous => "Vacuous",
        };
    }

    /// The whole exit-code policy as one assertion, and the reason the workflow needs no
    /// branch.
    ///
    /// `OD-GATE-004` decided that zero is the only success and implemented it by writing no
    /// policy: Actions fails a step on any non-zero exit. So this is the only place in the
    /// tree where the policy is checkable. If [`ExitCode::Vacuous`] were renumbered to `0`
    /// "because there is nothing to report", CI would start passing runs that judged nothing
    /// and nothing else would notice.
    #[test]
    fn Test_Only_Ok_Should_Carry_The_Passing_Exit_Code()
    {
        for code in Every_Exit_Code()
        {
            assert_eq!(
                code.Value() == 0,
                code == ExitCode::Ok,
                "{} exits {}, and the gate reads zero and only zero as success",
                Labelled(code),
                code.Value()
            );
        }
    }

    /// The codes this file documents are the codes this group can exit with.
    ///
    /// `P10-CHECK-GATE`'s `done_when` asks that the codes the gate rests on be "the ones
    /// crates/host/nomos-cli/src/check.rs documents, read from there rather than restated".
    /// The workflow honours the second half by restating nothing. This is what makes the
    /// first half true of *this* file: [`USAGE`] is prose a person reads and [`ExitCode`] is
    /// what the process returns, the two were written separately, and a code added or
    /// renumbered in one of them and not the other is the failure that actually happens.
    #[test]
    fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
    {
        let (_, spelled) = USAGE
            .split_once("exit codes:")
            .expect("the usage text documents the exit codes");

        let mut documented: Vec<i32> = spelled
            .split_whitespace()
            .filter_map(|word| return word.parse::<i32>().ok())
            .collect();
        documented.sort_unstable();

        let mut implemented: Vec<i32> = Every_Exit_Code()
            .iter()
            .map(|code| return code.Value())
            .collect();
        implemented.sort_unstable();

        assert!(
            !documented.is_empty(),
            "no exit code was parsed out of the usage text, so this compared nothing: \
             {spelled}"
        );
        assert_eq!(
            documented, implemented,
            "the usage text and ExitCode disagree about what this command can exit with, \
             and the gate step reads its policy off the latter"
        );
    }

    #[test]
    fn Test_A_Root_Should_Default_To_Here()
    {
        assert_eq!(Parse(&[]).expect("no arguments is valid").root, PathBuf::from("."));
    }

    #[test]
    fn Test_A_Given_Root_Should_Win()
    {
        let arguments = vec!["--root".to_owned(), "somewhere".to_owned()];

        assert_eq!(
            Parse(&arguments).expect("--root is valid").root,
            PathBuf::from("somewhere")
        );
    }

    /// A mistyped flag must not be silently ignored into a default. `nomos check --rooot x`
    /// walking the current directory instead would report on the wrong tree and say
    /// nothing about it.
    #[test]
    fn Test_An_Unknown_Flag_Should_Refuse()
    {
        let arguments = vec!["--rooot".to_owned(), "x".to_owned()];

        let error = Parse(&arguments).expect_err("must refuse");

        assert!(error.contains("--rooot"), "{error}");
        assert!(error.contains("usage"), "{error}");
    }

    /// A tree that is not there is not a clean tree.
    #[test]
    fn Test_A_Missing_Root_Should_Be_Unreadable()
    {
        let command = CheckCommand {
            root: PathBuf::from("no-such-directory-anywhere"),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        assert_eq!(Run(&command, &mut stdout, &mut stderr), ExitCode::Unreadable);
    }

    #[test]
    fn Test_A_Blocking_Finding_Should_Exit_Nonzero()
    {
        let sources = vec![Source(
            "a.rs",
            "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
        )];
        let findings = Composed::Over(&sources).Findings(&sources);
        let mut stdout = Vec::new();

        assert_eq!(
            Report(&findings, Examined { files: 1, facts: 1 }, &mut stdout),
            ExitCode::Violations
        );
    }

    /// An advisory finding is reported and does not stop anybody. Twelve of them exist
    /// in this workspace today, and a gate that can never be green is one everybody
    /// learns to bypass.
    #[test]
    fn Test_An_Advisory_Finding_Should_Not_Fail_The_Run()
    {
        let sources = vec![Source("a.rs", "pub const T: &[&str] = &[];\n")];
        let findings = Composed::Over(&sources).Findings(&sources);
        let mut stdout = Vec::new();

        assert_eq!(
            Report(&findings, Examined { files: 1, facts: 1 }, &mut stdout),
            ExitCode::Ok
        );
        assert!(!findings.is_empty(), "there is something to report");
    }

    /// The counts are part of the result. Without them, a broken walk and a clean tree
    /// render the same line.
    #[test]
    fn Test_The_Report_Should_Say_How_Much_Was_Looked_At()
    {
        let mut stdout = Vec::new();

        let _code = Report(&[], Examined { files: 41, facts: 39 }, &mut stdout);

        let rendered = String::from_utf8(stdout).expect("output is utf-8");

        assert!(rendered.contains("41 file(s) examined"), "{rendered}");
        assert!(rendered.contains("39 with a syntax fact"), "{rendered}");
    }

    /// ---- the shipped binary consults a fact ----
    ///
    /// The assertion `P10-FACT-BYPASS` turns on. The parser this command registers really
    /// produces the fact, and the rule's verdict really depends on it: with the defining
    /// file's fact in the store the claim resolves and nothing is reported, and with it
    /// withheld the claim does not resolve. Same text, same rule, different store.
    #[test]
    fn Test_The_Composed_Command_Should_Resolve_A_Mirror_Through_A_Real_Fact()
    {
        let declaring = Source(
            "a.rs",
            "/// Mirrored by `Test_The_Real_Provider_Found_This`.\n\
             pub const T: &[&str] = &[];\n",
        );
        let checking = Source(
            "b.rs",
            "#[cfg(test)]\nmod tests\n{\n    #[test]\n    fn Test_The_Real_Provider_Found_This()\n    {\n    }\n}\n",
        );
        let whole = vec![declaring.clone(), checking.clone()];

        let resolved = Composed::Over(&whole).Findings(&whole);
        assert!(
            resolved.is_empty(),
            "the registered parser must find the check in b.rs: {resolved:?}"
        );

        // The store is told about the declaring file only; the rule is handed both.
        let short = Composed::Over(&[declaring]).Findings(&whole);
        assert!(
            short
                .iter()
                .any(|finding| return finding.subject_name == "T"),
            "with b.rs's fact withheld the claim must not resolve: {short:?}"
        );
    }

    /// A run that materialized nothing must not print a clean tree.
    ///
    /// Every file the walk found is one the provider refuses, so the store is empty. That
    /// is `Vacuous` — the answer is empty because something expected was not there — and
    /// not `Ok`.
    #[test]
    fn Test_A_Run_That_Materialized_No_Facts_Should_Not_Report_Clean()
    {
        let root = std::env::temp_dir().join("nomos-check-no-facts");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        std::fs::write(root.join("broken.rs"), "pub const ??? = ;").expect("writable");

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&root);

        assert_eq!(code, ExitCode::Vacuous);
        assert!(
            String::from_utf8_lossy(&stderr).contains("no syntax fact was materialized"),
            "{}",
            String::from_utf8_lossy(&stderr)
        );
    }

    /// And the control for it: a tree the provider *can* read exits on what the rule found
    /// rather than on vacuity. Without this the test above is satisfied by a command that
    /// always reports `Vacuous`.
    #[test]
    fn Test_A_Run_That_Materialized_Facts_Should_Judge_Rather_Than_Refuse()
    {
        let root = std::env::temp_dir().join("nomos-check-with-facts");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        std::fs::write(
            root.join("a.rs"),
            "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
        )
        .expect("writable");

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&root);

        assert_eq!(
            code,
            ExitCode::Violations,
            "{}",
            String::from_utf8_lossy(&stdout)
        );
    }

    /// ---- one unreadable file does not silence the rest of the tree ----
    ///
    /// The measurement `OD-RULES-002` was opened against, reproduced as a directory. Two
    /// files: one the real parser reads, declaring a mirror that resolves to nothing, and
    /// one the real parser refuses. Before that record this run exited `0` and printed the
    /// phantom as `[Advisory]`, because `broken.rs` set one incompleteness flag over the
    /// whole run — and this workspace always holds such a file, so the guard could never
    /// block on anything.
    ///
    /// Asserted through `Run` and not through the rule, because the thing that was wrong
    /// was the exit code of the shipped binary. The provider here is the registered one, so
    /// the refusal is a real refusal rather than a withheld fixture.
    #[test]
    fn Test_A_Phantom_Should_Block_Though_The_Tree_Holds_A_File_The_Parser_Refuses()
    {
        let root = std::env::temp_dir().join("nomos-check-phantom-beside-broken");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        std::fs::write(
            root.join("a.rs"),
            "/// Mirrored by `Test_Renamed_Away`.\npub const T: &[&str] = &[];\n",
        )
        .expect("writable");
        std::fs::write(root.join("broken.rs"), "pub const ??? = ;\n").expect("writable");

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = Run(&CheckCommand { root: root.clone() }, &mut stdout, &mut stderr);
        let rendered = String::from_utf8_lossy(&stdout).into_owned();

        let _ignored = std::fs::remove_dir_all(&root);

        assert_eq!(code, ExitCode::Violations, "{rendered}");
        assert_ne!(code.Value(), ExitCode::Ok.Value(), "{rendered}");
        assert!(
            rendered.contains("2 file(s) examined, 1 with a syntax fact"),
            "the run must still report what it could not read: {rendered}"
        );
        assert!(
            rendered.contains("1 of which can fail a build"),
            "the phantom is the finding that blocks: {rendered}"
        );
        assert!(
            rendered.contains("[Blocking]") && rendered.contains("Test_Renamed_Away"),
            "{rendered}"
        );
    }

    /// The floor is the rule's, and the run this command composes meets it.
    ///
    /// Asserted here because this is the only place in the workspace where the rule's
    /// requirement and a real provider's offer are both nameable. If it ever failed, every
    /// subject would be reported unread and the command would report that it could not run
    /// — which is honest, and is not what anybody installed it for.
    #[test]
    fn Test_The_Registered_Provider_Should_Satisfy_The_Rules_Floor()
    {
        let sources = vec![Source("a.rs", "pub const T: &[&str] = &[];\n")];
        let findings = Composed::Over(&sources).Findings(&sources);

        assert!(
            findings
                .iter()
                .all(|finding| return finding.applicability == Applicability::Supported),
            "the registered parser must serve nomos_rules::Syntax_Requirement: {findings:?}"
        );
        assert!(
            findings
                .iter()
                .all(|finding| return finding.gate == GateCategory::Advisory),
            "{findings:?}"
        );
    }
}
