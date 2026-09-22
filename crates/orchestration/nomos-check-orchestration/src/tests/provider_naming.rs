//! The guard on `OD-ROADMAP-005` decision item 1's own measurement: which files in this
//! crate are allowed to name an analysis provider crate.
//!
//! Before that item, thirty-odd provider symbols were spread across sixteen files -- every
//! materialization step named the crate it called, which is what made an external review
//! call this crate a composition root disguised as an application service. Putting them in
//! one place is only half the fix; the other half is that they stay there, and nothing about
//! the port types makes that automatic. A new capability wired the old way compiles, passes
//! every other test in this crate, and quietly restores the defect.
//!
//! So this file is the assertion that the rest of the item is checkable. It reads this
//! crate's own sources off disk and refuses any provider-crate identifier outside the
//! composition module.

use std::path::{Path, PathBuf};

/// The identifier prefixes a provider crate is named by, in the spelling Rust source uses.
///
/// Three prefixes rather than a list of crate names, deliberately: a list would have to be
/// extended by whoever adds the fourteenth provider, which is exactly the person this guard
/// is aimed at. `nomos_cap_` is absent because a capability *contract* crate is not a
/// provider and this crate legitimately declares contracts;
/// `nomos_cap_requirement_trace` bundles both and is named in the composition module, which
/// is allowed ground either way.
const PROVIDER_PREFIXES: [&str; 3] = ["nomos_lang_", "nomos_repo_policy", "nomos_connector_"];

/// The files allowed to name one, relative to `src/`, with forward slashes.
///
/// Two files and not one because `crates/orchestration/nomos-check-orchestration/src/
/// composition.rs` and its one submodule are one module in this repository's normal form --
/// the same `foo.rs` plus `foo/` shape `facts` and `run_context` already have. Merging them
/// would produce a single file past this workspace's own file-size review trigger, which is
/// a worse answer to "the naming lives in one place" than naming the two files here.
const COMPOSITION_MODULE: [&str; 2] = ["composition.rs", "composition/provider_table.rs"];

/// No file outside the composition module may name a provider crate, outside its own tests.
///
/// The test half of each file is excluded rather than judged: a test is a composition root
/// of its own and is entitled to name the provider whose behaviour it is asserting, which is
/// the same licence `crate::composition::provider_table`'s own test module takes. The
/// exclusion is by this repository's own convention -- a `#[cfg(test)]` module is last in a
/// file -- so everything from the first such attribute to the end of the file is dropped
/// before scanning, and a file that is nothing but tests is skipped by its path.
/// [`Judged_Code`] says why comments are dropped too.
#[test]
fn Test_Only_The_Composition_Module_Should_Name_A_Provider_Crate()
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
        if COMPOSITION_MODULE.contains(&relative.as_str()) || Is_Test_Only(&relative)
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
        "OD-ROADMAP-005 decision item 1: only the composition module may name a provider crate, and these name one: {offenders:?}. \
         A new capability is wired by adding a row to crate::composition::Composed_Providers and reaching it through a port, \
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
