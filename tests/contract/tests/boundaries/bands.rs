//! The band table, and the walk two of the assertions need.
//!
//! `BANDS` is here because it is the subject of two different questions — whether the README
//! restates it and whether the dependency graph respects it — and a second copy of it would
//! let those two answers disagree.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The declared band of every workspace member.
///
/// Authored here rather than derived, because a band is a design decision and there is
/// nothing in the source to infer it from. Adding a crate without adding it here fails
/// [`crate::graph::Test_Every_Member_Should_Declare_A_Band`] — so a new crate cannot quietly
/// join the workspace outside the ordering.
///
/// Bands are scaled by ten so that a band can be inserted between two others without
/// renumbering everything, which is the sort of churn that turns a table nobody wants
/// to touch into a table nobody updates.
pub(crate) const BANDS: &[(&str, u32)] = &[
    ("nomos-contracts", 0),
    ("nomos-model", 10),
    ("nomos-store", 12),
    // The port and its implementations are not peers. The traits sit below, so that
    // swapping an implementation cannot recompile anything that only knows the port —
    // which is the entire reason the seam exists.
    ("nomos-platform", 15),
    ("nomos-platform-std", 16),
    // What the workspace currently is: above the document store it records into, below
    // everything that keys a fact on a snapshot or a build variant.
    ("nomos-workspace", 18),
    // Territory and VerificationPredicate, the two ledger-agnostic primitives
    // `OD-LEDGER-037` found underneath `nomos-agent-contracts`'s reuse of `nomos-ledger`.
    // Below the ledger and below `nomos-agent-contracts`, so both may depend on it.
    ("nomos-scope-verification", 19),
    ("nomos-ledger", 20),
    // The product substrate. Capability sits above the model and below everything
    // that resolves a provider through it.
    ("nomos-capability", 21),
    ("nomos-analysis", 22),
    // A capability contract sits above the registry that resolves it and below every
    // provider that offers against it. Not beside the providers: an agreement that lives
    // with one party to it is that party's to change, and the other cannot see the file.
    ("nomos-cap-syntax", 23),
    // The nomos.cap.dependency.edges contract, below both its provider and the rule that
    // reads it -- nomos-rules is a real second party from the day it was written, so this
    // did not wait beside nomos-lang-rust-cargo the way nomos.cap.module.index waits
    // inside nomos-lang-rust. `OD-CAPABILITY-002`.
    ("nomos-cap-dependency", 23),
    // The nomos.cap.controlflow.reachability contract, below both its provider
    // (nomos-lang-rust) and the rule that reads it (nomos-rules) for the identical
    // reason nomos-cap-dependency states above. `OD-RULES-008`.
    ("nomos-cap-controlflow", 23),
    // The nomos.cap.lint.diagnostics contract, below both its provider
    // (nomos-lang-rust-clippy) and the rule that reads it (nomos-rules) for the
    // identical reason nomos-cap-dependency states above. `OD-RULES-010`.
    ("nomos-cap-lint", 23),
    // The nomos.cap.dependency.policy contract, below both its provider
    // (nomos-lang-rust-deny) and the rule that reads it (nomos-rules) for the identical
    // reason nomos-cap-dependency states above. OD-RULES-010's second real ToolProvider
    // capability.
    ("nomos-cap-dependency-policy", 23),
    // The language-agnostic manifest core PKG-007's four version domains name, minus
    // any typed version-domain abstraction or provider allowlist a specific language
    // would supply. Below the language providers deliberately: it names none of them,
    // so a second language's package crate can depend on this one without also
    // depending on Rust's. `OD-PACKAGE-007`.
    ("nomos-package", 24),
    // Language providers sit above analysis because they produce the facts it stores,
    // and nothing sits above them but a composition root. They reach each other not at
    // all: two languages are two providers of one capability, and the registry is the
    // only thing that knows both.
    ("nomos-lang-rust", 25),
    // Its peer, deliberately at the same band. Two providers of one capability must not
    // be able to name each other: this suite's downward rule forbids an edge between
    // crates at one band, which is what stops the second answer from being derived from
    // the first. Two providers that shared a parser could not disagree.
    ("nomos-lang-rust-scan", 25),
    // The first real second-language provider of the same capability, over tree-sitter-go
    // in place of syn. Same band as its two Rust-reading siblings, for the same reason:
    // none of the three may name either of the others, and the registry is what lets a
    // caller resolve to any of them without knowing which.
    ("nomos-lang-go", 25),
    // The one provider of nomos.cap.dependency.edges. Different shape from its two
    // siblings here -- it is the only provider in this workspace that performs I/O -- but
    // the same band, for the same reason: nothing below it may be able to name it and
    // nothing beside it may either.
    ("nomos-lang-rust-cargo", 25),
    // The one provider of nomos.cap.lint.diagnostics. The same shape and the same
    // reasoning as nomos-lang-rust-cargo above: a subprocess-backed provider with I/O of
    // its own, at the same band, naming neither it nor either of its syntax-reading
    // siblings.
    ("nomos-lang-rust-clippy", 25),
    // The one provider of nomos.cap.dependency.policy. OD-RULES-010's second real
    // ToolProvider: a subprocess-backed provider with I/O of its own, at the same band as
    // nomos-lang-rust-cargo and nomos-lang-rust-clippy, naming neither them nor either of
    // its syntax-reading siblings.
    ("nomos-lang-rust-deny", 25),
    // A second provider of nomos.cap.dependency.edges, for a Go workspace. Reads
    // go.work/go.mod text directly rather than running a subprocess -- its own module doc
    // says why -- but the same band and the same rule as its three siblings above: none of
    // the four may name any other. `OD-CAPABILITY-009`.
    ("nomos-lang-go-modules", 25),
    // The Rust installable-unit manifest format: PackageId, PackageKind and PKG-007's
    // four version domains, given a reader that refuses what it cannot resolve. Wraps
    // nomos-package's generic core with RustEdition resolution. Above the two language
    // providers it registers -- it must be able to name them -- and well clear of
    // rules, which it does not touch. `OD-PACKAGE-001`, `OD-PACKAGE-007`.
    ("nomos-lang-rust-package", 26),
    // The first ModelBackendPackage/AgentExecutorPackage manifest maturity. Same band as
    // nomos-lang-rust-package -- a peer wrapping nomos-package's generic core for a different
    // PackageKind family, not a dependent of it. `OD-PACKAGE-010`.
    ("nomos-model-package", 26),
    // The first RulePackage manifest maturity. Same band as nomos-lang-rust-package and
    // nomos-model-package -- a third peer wrapping nomos-package's generic core, not a
    // dependent of either. `OD-PACKAGE-008`, `OD-ROADMAP-001`.
    ("nomos-rule-package", 26),
    // The first real second consumer of nomos-package's generic core: the Go
    // LanguagePackage manifest format. A fourth peer of the three above -- none of the
    // four names another. `OD-PACKAGE-006`, `OD-PACKAGE-007`.
    ("nomos-lang-go-package", 26),
    // The spec system sits beside the kernel, not above it. It reaches the product only
    // through a KnowledgeCapability, so nothing in the product may name it directly.
    ("nomos-spec-model", 11),
    ("nomos-spec-store", 12),
    // Bundle and ingest are peers over the store: one reads the corpus in, the other
    // writes it back out. Neither may name the other.
    ("nomos-spec-bundle", 13),
    ("nomos-spec-ingest", 13),
    ("nomos-spec-validate", 14),
    ("nomos-spec-project", 14),
    // Rules sit above everything they could ever need to judge and below the only thing
    // that runs them. Deliberately well clear of the language providers at 25: a rule is
    // a pure function from source text to findings and names no provider today, but the
    // moment one needs a parsed tree it must be able to reach a provider rather than
    // vendor a second parser — and a band below them would have forbidden that edge and
    // made the second parser the easy answer.
    ("nomos-rules", 30),
    // The step that applies what a rule found. Above rules deliberately, with room
    // between: a correction candidate may one day need to name a finding a rule
    // produced, and a band below rules would have forbidden that edge.
    ("nomos-corrections", 35),
    // AGT-001's TaskEnvelope and AGT-002's WorkResult, the typed input and output shape
    // an agent-assisted operation carries. Depends downward on nomos-scope-verification,
    // nomos-contracts and nomos-corrections. `OD-ROADMAP-001`, `OD-LEDGER-037`.
    ("nomos-agent-contracts", 36),
    // The first real AgentExecutor: dispatches a TaskEnvelope's goal to Claude Code as a
    // subprocess through nomos-platform's ProcessLauncher, bounded by OD-EXECUTOR-001's
    // structural capability boundary. Above nomos-agent-contracts. `OD-EXECUTOR-001`.
    ("nomos-agent-executor-claude-code", 37),
    // The second real AgentExecutor, independent of its sibling above: dispatches a
    // TaskEnvelope's goal to a local Ollama model as a subprocess through nomos-platform's
    // ProcessLauncher, bounded by OD-EXECUTOR-004's structural capability boundary,
    // measured against its own real mechanism. Same band as its sibling; neither may name
    // the other. `OD-EXECUTOR-004`.
    ("nomos-agent-executor-ollama", 37),
    // Runs a `nomos work` verb against a caller-chosen platform and hands back a typed
    // outcome, generic over the traits `nomos-platform` declares rather than over the
    // std implementation of them. Above the ledger it dispatches to; below every
    // composition root that could call it — `nomos-cli` today, and whatever a second
    // adapter is tomorrow. `OD-HOST-001`.
    ("nomos-work-orchestration", 40),
    // Composes the capability registry, ingests already-walked source into facts and
    // judges it, apart from choosing a platform, walking a tree or rendering the answer.
    // Same band as `nomos-work-orchestration`: both are the middle of a three-crate seam
    // between a composition root and the substrate it orchestrates. `OD-HOST-002`.
    ("nomos-check-orchestration", 40),
    // Assembles the specification store from the embedded governing records and a
    // caller-named corpus, and answers the `SpecCommand` verbs that touch no store or
    // only describe one, apart from choosing a platform or rendering the answer. Same
    // band as its two siblings above: the middle of a three-crate seam, increment 1 of 4
    // closing family 9's `SpecCommand` half. `OD-HOST-002`.
    ("nomos-spec-orchestration", 40),
    // The seam for the first-class Gate object `ARC-ROADMAP-001` names: `Plan` reaches
    // only `nomos-rules` (30) and `nomos-contracts` (0); `Run_Gate` reaches
    // `nomos-check-orchestration` (40) too, one band below, which is why this crate is
    // 41 rather than a fourth sibling at 40 -- a band may not depend on its own band.
    // `OD-HOST-001`, `P13-GATE-RUN-SEAM-CRATE`.
    ("nomos-gate-orchestration", 41),
    ("nomos-cli", 90),
    // A second real caller of `nomos-gate-orchestration`'s `Run_Gate`: walks a tree,
    // judges it exactly as `nomos gate run` would, and hands back a JSON-serializable
    // response rather than rendered text. Same band as `nomos-cli`: both are composition
    // roots over the same orchestration seams, neither depending on the other.
    // `P13-GATE-API-ADAPTER-FIRST-INCREMENT`.
    ("nomos-api", 90),
    // A report over this repository's own git history, not over the workspace's crate
    // graph — OD-STORE-002's Worked Case join between a crate's surface snapshot and
    // docs/records/, for a caller-given commit range. Above nomos-cli because it is a
    // second, unrelated composition root rather than something nomos-cli depends on;
    // below the observers at 100 because it depends on nomos-platform and
    // nomos-platform-std like any other host-band binary, not because anything below
    // it may name it.
    ("nomos-surface-provenance", 91),
    // The contract tests sit at the top: they observe the workspace and nothing
    // observes them.
    ("nomos-contract-tests", 100),
    // The vertical slice is their peer, not their superior. Both are terminal, and
    // sharing a band is what makes them unable to name each other: this suite's
    // downward rule forbids an edge between two crates at the same band. That is the
    // property wanted — an observer of the workspace that also participates in it
    // could no longer be trusted to report on it.
    ("nomos-integration-tests", 100),
];

pub(crate) fn Declared_Band(name: &str) -> Option<u32>
{
    return BANDS
        .iter()
        .find(|(crate_name, _)| *crate_name == name)
        .map(|(_, band)| *band);
}

/// The workspace root, from this crate's manifest directory.
pub(crate) fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
}

/// Every `.rs` file under a directory, recursively.
pub(crate) fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();
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
            Sort_One_Entry(&entry.path(), &mut pending, &mut found);
        }
    }

    return found;
}

/// A directory to descend into later, a Rust file to keep, or neither.
fn Sort_One_Entry(path: &Path, pending: &mut Vec<PathBuf>, found: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path.to_path_buf());
    }
    else if path.extension().is_some_and(|extension| extension == "rs")
    {
        found.push(path.to_path_buf());
    }
}

/// Every `mod` name declared anywhere under a source root.
///
/// Deliberately a flat set rather than a resolved tree. The precise version would walk
/// declarations from each root, and would need to handle `#[path]`, `#[cfg]` and inline
/// modules to avoid false positives. This approximation cannot report a false orphan —
/// it only misses the case where a module is declared in one place and the file lives in
/// an unrelated one, which is a naming problem rather than an invisibility problem.
pub(crate) fn Declared_Modules(source_root: &Path) -> BTreeSet<String>
{
    let mut declared = BTreeSet::new();
    for file in Source_Files(source_root)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };
        for line in text.lines()
        {
            let named = Module_Declared_By(line);

            declared.extend(named);
        }
    }

    return declared;
}

/// The file name one `mod` line declares.
///
/// A commented-out declaration is not a declaration. The sibling workspace names this
/// specifically as a decoy that made an orphan look declared. And `mod foo;` is a declaration
/// of a file where `mod foo {` is an inline module, which declares nothing on disk.
fn Module_Declared_By(line: &str) -> Option<String>
{
    let trimmed = line.trim();
    if trimmed.starts_with("//")
    {
        return None;
    }

    let rest = trimmed
        .strip_prefix("mod ")
        .or_else(|| trimmed.strip_prefix("pub mod "))
        .or_else(|| trimmed.strip_prefix("pub(crate) mod "))?;

    return rest.strip_suffix(';').map(|name| return name.trim().to_owned());
}
