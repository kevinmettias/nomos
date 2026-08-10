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

use crate::arguments::Named_Value;
use nomos_analysis::{Context, MemoryFactStore, Reader};
use nomos_capability::Registry;
use nomos_contracts::{ConfigurationId, Finding, Guarantee, SubjectId};
use nomos_lang_rust::{FactContext, Materialization};
use nomos_model::Content_Digest;
use nomos_rules::{Check_Completeness_Mirrors, SourceFile};
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What the process exits with.
///
/// The numbers are shared with every other group on this binary: an exit code means one
/// thing per binary rather than one thing per group. `3` and `4` are `work`'s claim
/// codes and are not reused here, and `5` and `6` carry the meanings `spec` gave them —
/// "could not be read at all" and "the answer is empty because something expected was
/// not there".
///
/// [`ExitCode::Vacuous`] is the one that earns its own code rather than folding into
/// `Ok`. A run that judged nothing and a run that judged everything and approved are the
/// two states this binary must never render the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitCode
{
    /// The rules ran and nothing they found can fail a build.
    Ok = 0,
    /// At least one finding can fail a build.
    Violations = 1,
    /// The command line was wrong.
    Usage = 2,
    /// The tree could not be read at all.
    Unreadable = 5,
    /// Nothing was judged: the walk found no source, or no fact was materialized for any
    /// of the source it found. Either way a clean result would mean nothing.
    Vacuous = 6,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// What to check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckCommand
{
    /// The tree to judge.
    pub root: PathBuf,
}

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
    if !command.root.is_dir()
    {
        let _ignored = writeln!(
            stderr,
            "cannot read `{}`: not a directory",
            command.root.display()
        );
        return ExitCode::Unreadable;
    }

    let sources = Read_Sources(&command.root);

    if sources.is_empty()
    {
        let _ignored = writeln!(
            stderr,
            "no Rust source found under `{}`, so nothing was judged.\n\
             A clean result here would mean only that the walk found nothing.",
            command.root.display()
        );
        return ExitCode::Vacuous;
    }

    let registry = Registered();
    let configuration = Resolved_Configuration(&registry);
    let variant = Host_Variant();
    let mut workspace = Workspace::Empty(variant.clone(), configuration);

    let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
    for source in &sources
    {
        checkout = checkout.Present(source.path.clone(), source.text.clone());
    }

    // The whole walk arrives as one change set because a walk is one event. Applying a
    // file at a time would produce one generation per file, and every intermediate one
    // would describe a tree that never existed.
    let applied = match workspace.Apply(&checkout)
    {
        Ok(applied) => applied,
        Err(error) =>
        {
            let _ignored = writeln!(
                stderr,
                "the walk of `{}` could not be ingested as a workspace state: {error:?}",
                command.root.display()
            );
            return ExitCode::Unreadable;
        }
    };

    let context = Context {
        snapshot: applied.Snapshot(),
        variant: variant.Id(),
        configuration,
        generation: applied.Generation(),
    };

    let mut store = MemoryFactStore::New();
    let facts = Materialize_Syntax(&sources, &context, &mut store);

    if facts == 0
    {
        let _ignored = writeln!(
            stderr,
            "{} file(s) were read under `{}` and no syntax fact was materialized for any \
             of them, so no mirror claim could be resolved.\n\
             A clean result here would mean only that the analysis never ran.",
            sources.len(),
            command.root.display()
        );
        return ExitCode::Vacuous;
    }

    let mut reader = Reader::On(&store, &registry, context);
    let findings = Check_Completeness_Mirrors(&sources, &mut reader);

    return Report(
        &findings,
        Examined {
            files: sources.len(),
            facts,
        },
        stdout,
    );
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

        for offer in registry.Offers(&contract.id)
        {
            rendered.push_str("offer\t");
            rendered.push_str(contract.id.As_Str());
            rendered.push('\t');
            rendered.push_str(offer.provider.As_Str());
            rendered.push('\t');
            rendered.push_str(&offer.version.to_string());
            rendered.push('\t');
            rendered.push_str(&Rendered_Guarantee(offer.guarantee));
            rendered.push('\n');
        }
    }

    return ConfigurationId::From_Digest(Content_Digest(rendered.as_bytes()));
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
    let production = FactContext {
        snapshot: context.snapshot,
        variant: context.variant,
        configuration: context.configuration,
        generation: context.generation,
    };
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

    // The counts of what was looked at are part of the result, not decoration. "0
    // findings" over 4 files and "0 findings" over 400 are different claims, and so are
    // "400 files" and "400 files, 12 of which produced a fact" — a reader who cannot tell
    // them apart cannot tell a clean tree from a broken walk.
    let _ignored = writeln!(
        stdout,
        "\n{} file(s) examined, {} with a syntax fact, {} finding(s), {blocking} of which \
         can fail a build",
        examined.files,
        examined.facts,
        findings.len()
    );

    return if blocking > 0
    {
        ExitCode::Violations
    }
    else
    {
        ExitCode::Ok
    };
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

            if path.is_dir()
            {
                let skipped = path
                    .file_name()
                    .is_some_and(|name| return name == "target" || name == ".git");

                if !skipped
                {
                    pending.push(path);
                }
                continue;
            }

            if path.extension().is_some_and(|extension| return extension == "rs")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                let relative = Relative(root, &path);
                let subject = Subject_Of_Path(&relative);
                sources.push(SourceFile::New(relative, subject, text));
            }
        }
    }

    sources.sort_by(|left, right| return left.path.cmp(&right.path));
    return sources;
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

/// The identity of the subject a tree-relative path denotes.
///
/// This root files a fact under it and hands the same value to the rule on
/// [`SourceFile::subject`], so the two cannot disagree about addressing. Separators are
/// unified, `.` segments dropped, and case folded — because `Main.rs` and `main.rs` are one
/// file on two of the three platforms this runs on, and treating them as two subjects would
/// let one edit invalidate neither.
///
/// # Why this normalization is written here
///
/// It is the third copy in the workspace. `nomos_ledger::Subject_Of` computes a subject for
/// work *territory* and additionally folds a record filename onto its identifier;
/// `tests/integration/src/corpus.rs` computes one for a *fact* and says in its own doc
/// comment that the duplication "is worth converging behind one home the moment a third
/// caller appears". This is that third caller, and converging is not this item's: it would
/// mean editing a crate this item does not hold, and importing the ledger's would couple
/// what a fact is about to what a claim is about — two vocabularies that agree today and
/// have no reason to stay agreed. `P10-SUBJECT-HOME` holds the convergence.
fn Subject_Of_Path(path: &str) -> SubjectId
{
    let normalized = path
        .trim()
        .replace('\\', "/")
        .split('/')
        .filter(|segment| return !segment.is_empty() && *segment != ".")
        .collect::<Vec<&str>>()
        .join("/")
        .to_lowercase();

    return SubjectId::From_Digest(Content_Digest(normalized.as_bytes()));
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
            let configuration = Resolved_Configuration(&registry);
            let variant = Host_Variant();
            let mut workspace = Workspace::Empty(variant.clone(), configuration);

            let mut checkout = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
            for source in sources
            {
                checkout = checkout.Present(source.path.clone(), source.text.clone());
            }
            let applied = workspace.Apply(&checkout).expect("the fixture is a valid tree");

            let context = Context {
                snapshot: applied.Snapshot(),
                variant: variant.Id(),
                configuration,
                generation: applied.Generation(),
            };
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
