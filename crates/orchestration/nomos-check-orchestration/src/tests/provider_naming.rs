//! The guard on `OD-ROADMAP-005` decision item 1 as `OD-HOST-020` finished it: this crate's
//! production code names no analysis provider crate at all.
//!
//! Before that item, thirty-odd provider symbols were spread across sixteen files -- every
//! materialization step named the crate it called, which is what made an external review
//! call this crate a composition root disguised as an application service. A first step
//! gathered them into one composition module; `OD-HOST-020` moved that module out, so the
//! set is now selected by `nomos-composer-providers` and arrives here through `RunContext`.
//! The strong form is now the true one, and nothing about the port types makes it automatic:
//! a new capability wired the old way compiles, passes every other test in this crate, and
//! quietly restores the defect.
//!
//! So this file is the assertion that the rest of the item is checkable. It reads this
//! crate's own sources off disk and refuses any provider-crate identifier in production code,
//! anywhere. This crate's `Cargo.toml` enforces the same thing from the other side -- no
//! provider crate is in its `[dependencies]`, so production code that named one would not
//! compile -- and this guard exists so that the reason is a sentence a reader meets, rather
//! than a build error a reader has to diagnose.

use std::path::{Path, PathBuf};

/// The identifier prefixes a provider crate is named by, in the spelling Rust source uses.
///
/// Three prefixes rather than a list of crate names, deliberately: a list would have to be
/// extended by whoever adds the fourteenth provider, which is exactly the person this guard
/// is aimed at. `nomos_cap_` is absent because a capability *contract* crate is not a
/// provider and this crate legitimately declares contracts. `nomos_cap_requirement_trace`
/// bundles a contract with its one provider: this crate still names it to declare the
/// contract, and its provider offer moved to `nomos-composer-providers` with every other.
const PROVIDER_PREFIXES: [&str; 3] = ["nomos_lang_", "nomos_repo_policy", "nomos_connector_"];

/// The one file allowed to name a provider crate, relative to `src/`: the unit tests' local
/// provider table.
///
/// It names every provider because it is a copy of the standard set, and it is test code:
/// `composition.rs` declares it under a test gate, which a whole-file check cannot see, so it
/// is exempted here by name instead. `OD-HOST-020` section 5 records why it exists at all --
/// this crate's unit tests cannot use `nomos-composer-providers`, which depends back on this
/// crate and would hand them a `ComposedProviders` of a different type. `composition.rs`
/// itself is NOT exempt: it declares contracts and names no provider, and this guard proves it.
const TEST_ONLY_TABLE: &str = "composition/provider_table.rs";

/// No production file in this crate may name a provider crate.
///
/// The test half of each file is excluded rather than judged: a test is a composition root
/// of its own and is entitled to name the provider whose behaviour it is asserting, which is
/// the same licence the local table [`TEST_ONLY_TABLE`] takes as a whole. The
/// exclusion is by this repository's own convention -- a `#[cfg(test)]` module is last in a
/// file -- so everything from the first such attribute to the end of the file is dropped
/// before scanning, and a file that is nothing but tests is skipped by its path.
/// [`Judged_Code`] says why comments are dropped too.
#[test]
fn Test_No_Production_File_Should_Name_A_Provider_Crate()
{
    let sources = Crate_Sources();
    assert!(
        sources.len() > 10,
        "this guard reads this crate's own sources off disk; finding {} of them means it scanned the wrong tree and would pass vacuously",
        sources.len()
    );

    let mut offenders: Vec<String> = Vec::new();
    for source in &sources
    {
        let relative = Relative_To_The_Source_Directory(source);
        if relative == TEST_ONLY_TABLE || Is_Test_Only(&relative)
        {
            continue;
        }

        if Names_A_Provider(&Judged_Code(source))
        {
            offenders.push(relative);
        }
    }

    assert!(
        offenders.is_empty(),
        "OD-HOST-020: this crate's production code names no provider crate, and these name one: {offenders:?}. \
         A new capability is wired by adding a row and an offer to nomos-composer-providers and reaching it through a port, \
         not by calling the provider where the fact is filed"
    );
}

/// Every `.rs` file under this crate's own `src/`.
fn Crate_Sources() -> Vec<PathBuf>
{
    let mut found = Vec::new();
    Collect_Sources(&Source_Root(), &mut found);
    found.sort();

    return found;
}

/// This crate's own `src/`, from the manifest directory the compiler stamps in.
fn Source_Root() -> PathBuf
{
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
}

/// `directory`'s own `.rs` files and those of every directory below it, appended to `found`.
fn Collect_Sources(directory: &Path, found: &mut Vec<PathBuf>)
{
    let entries = std::fs::read_dir(directory).expect("this crate's own source tree is readable");
    for entry in entries
    {
        let path = entry.expect("a readable directory entry").path();
        if path.is_dir()
        {
            Collect_Sources(&path, found);
        }
        else if path.extension().is_some_and(|extension| return extension == "rs")
        {
            found.push(path);
        }
    }
}

/// `source`'s path relative to `src/`, with forward slashes so a row above reads the same on
/// every platform.
fn Relative_To_The_Source_Directory(source: &Path) -> String
{
    let root = Source_Root();
    let relative = source.strip_prefix(&root).expect("every scanned file sits under this crate's own src/");

    return relative.to_string_lossy().replace('\\', "/");
}

/// Whether `relative` names a file that is nothing but tests.
fn Is_Test_Only(relative: &str) -> bool
{
    return relative == "tests.rs" || relative.starts_with("tests/") || relative.ends_with("/tests.rs") || relative.ends_with("_tests.rs");
}

/// `source`'s code: its text with its own `#[cfg(test)]` module and its own comment lines
/// dropped.
///
/// Comments are dropped because this guard is about what the crate *calls*, not about what
/// it explains. Several modules here name a provider crate in prose to say what a port
/// stands for -- `facts::currency`'s own doc lists the three subprocess-backed providers by
/// name to explain why their facts carry no input digest -- and a guard that refused that
/// would push the explanations out of the files that need them, which is the opposite of
/// what the item is for. A line naming a provider in code never begins with `//`.
fn Judged_Code(source: &Path) -> String
{
    let text = std::fs::read_to_string(source).expect("a readable source file");
    let code = text.find("#[cfg(test)]").map_or(text.as_str(), |boundary| return text.get(..boundary).unwrap_or_default());

    return code.lines().filter(|line| return !line.trim_start().starts_with("//")).collect::<Vec<&str>>().join(LINE_BREAK);
}

/// What [`Judged_Code`] rejoins the lines it kept with. A constant rather than an escape at
/// the call site, because this file is written through tooling that has mangled a `\n`
/// literal here once already.
const LINE_BREAK: &str = "\n";

/// Whether `text` names any provider crate.
fn Names_A_Provider(text: &str) -> bool
{
    return PROVIDER_PREFIXES.iter().any(|prefix| return text.contains(prefix));
}
