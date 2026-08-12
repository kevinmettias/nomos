//! Which tests cannot run without a corpus, read out of the source.
//!
//! A corpus-gated test returns early when its corpus is absent, and cargo prints `ok`
//! for a test that returned early exactly as it does for one that asserted. That is the
//! same shape as the defect this workspace exists to prevent: the mechanism was present
//! and it did not run, and nothing said so. Counting the gates is what makes the silence
//! measurable.
//!
//! Derived, never declared. A hand-written list of gated tests would be right on the day
//! it was written and wrong the first time somebody added a gate without updating it —
//! and a stale inventory understates the hole, which is the direction that flatters.

use crate::reading::functions::{Function, Functions};
use crate::Workspace;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The environment variables that point a test at a corpus.
///
/// Authored here because there is nothing to infer them from: a corpus is a path this
/// repository does not contain, and the only thing naming it is the variable. Adding a
/// fourth corpus without adding it here makes its tests invisible to the count, so the
/// list is short on purpose and belongs next to the scanner that reads it.
///
/// Mirrored by `Test_The_Scanner_And_This_Table_Should_Name_The_Same_Variables`.
pub const CORPUS_VARIABLES: &[&str] =
    &["NOMOS_V14_CORPUS", "NOMOS_SPEC_ARCHIVES", "NOMOS_RUST_CORPUS"];

/// One test that reads a corpus.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CorpusGate
{
    /// The file holding the test, relative to the workspace root, in forward slashes.
    pub file: String,
    /// The test function's name.
    pub test: String,
    /// Every corpus variable the test reaches, directly or through a helper it calls.
    pub corpora: BTreeSet<String>,
}

/// Every corpus-gated test in the workspace, in a stable order.
///
/// # Panics
///
/// Panics if the dependency graph cannot be read, by way of [`Workspace::Load`].
#[must_use]
pub fn Corpus_Gates() -> Vec<CorpusGate>
{
    let workspace = Workspace::Load();
    let root = Workspace::Workspace_Root();
    let mut gates = Vec::new();

    for member in workspace.Members()
    {
        // The crate doing the counting names the variables in order to count them, so
        // scanning itself would report its own inventory as a gate. Nothing here reads a
        // corpus: these are assertions about the workspace, and they must hold on a runner
        // that has none.
        if member.name != "nomos-contract-tests"
        {
            let found = Gates_In_Member(&member.root, &root);
            gates.extend(found);
        }
    }

    gates.sort();
    return gates;
}

/// Every gate one member declares, from both of the roots a crate compiles from.
fn Gates_In_Member(member_root: &Path, root: &Path) -> Vec<CorpusGate>
{
    let mut gates = Vec::new();

    for directory in ["src", "tests"]
    {
        let source_root = member_root.join(directory);
        if source_root.is_dir()
        {
            let found = Gates_Under(&source_root, root);
            gates.extend(found);
        }
    }

    return gates;
}

/// Every gate the files under one source root declare.
fn Gates_Under(source_root: &Path, root: &Path) -> Vec<CorpusGate>
{
    use crate::reading::source_files::Source_Files;

    let mut gates = Vec::new();

    for file in Source_Files(source_root)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };
        let relative = Relative_To(root, &file);
        let found = Gates_In(&relative, &text);

        gates.extend(found);
    }

    return gates;
}

/// A path as it reads from the workspace root, in the one spelling this crate compares on.
fn Relative_To(root: &Path, file: &Path) -> String
{
    return file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string()
        .replace('\\', "/");
}

/// The gated tests in one file.
///
/// Two passes. The first records which functions name a corpus variable outright; the
/// second propagates that along calls until nothing changes, because the gate is almost
/// never in the test — it is in a `Corpus()` helper the test calls, sometimes through a
/// second helper such as `Both()`.
///
/// Scoped to one file. A helper shared across files would be missed, which is a false
/// negative rather than a false positive: the count would be too low and the guard would
/// notice the next time it was compared.
fn Gates_In(file: &str, text: &str) -> Vec<CorpusGate>
{
    let functions = Functions(text);
    let mut reach = Named_Outright(&functions);

    Propagate(&functions, &mut reach);

    return functions
        .iter()
        .filter(|function| return function.is_test)
        .filter_map(|function| {
            let corpora = reach.get(&function.name)?;
            return Some(CorpusGate {
                file: file.to_owned(),
                test: function.name.clone(),
                corpora: corpora.clone(),
            });
        })
        .collect();
}

/// The functions that name a corpus variable in their own body, which is where the second
/// pass starts from.
fn Named_Outright(functions: &[Function]) -> BTreeMap<String, BTreeSet<String>>
{
    let mut reach: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for function in functions
    {
        for variable in CORPUS_VARIABLES
        {
            if function.body.contains(variable)
            {
                reach
                    .entry(function.name.clone())
                    .or_default()
                    .insert((*variable).to_owned());
            }
        }
    }

    return reach;
}

/// Carries each function's corpus variables along the calls it makes, until a pass discovers
/// nothing new.
fn Propagate(functions: &[Function], reach: &mut BTreeMap<String, BTreeSet<String>>)
{
    loop
    {
        let discovered = Newly_Reached(functions, reach);
        if discovered.is_empty()
        {
            break;
        }

        for (name, variables) in discovered
        {
            reach.entry(name).or_default().extend(variables);
        }
    }
}

/// What each function would gain this pass, for the functions that would gain anything.
fn Newly_Reached(
    functions: &[Function],
    reach: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<(String, BTreeSet<String>)>
{
    let mut discovered = Vec::new();

    for function in functions
    {
        let gained = Gained(function, reach);
        let known = reach.get(&function.name);
        if gained
            .iter()
            .any(|variable| return known.is_none_or(|set| return !set.contains(variable)))
        {
            discovered.push((function.name.clone(), gained));
        }
    }

    return discovered;
}

/// The corpus variables one function reaches through the helpers its body calls.
///
/// A function does not gain from itself: a recursive call would otherwise report a variable
/// as newly reached on every pass and the fixpoint above would never settle.
fn Gained(function: &Function, reach: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String>
{
    let mut gained = BTreeSet::new();

    for (name, variables) in reach
    {
        if name == &function.name
        {
            continue;
        }
        if function.body.contains(&format!("{name}("))
        {
            gained.extend(variables.iter().cloned());
        }
    }

    return gained;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Test_Calling_A_Gated_Helper_Should_Be_Gated()
    {
        let source = r#"
fn Corpus() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_V14_CORPUS")?);
    return Some(root);
}

#[test]
fn Test_Something()
{
    let Some(root) = Corpus() else { return };
}
"#;

        let gates = Gates_In("example.rs", source);
        assert_eq!(gates.len(), 1);
        assert_eq!(gates.first().map(|gate| return gate.test.clone()), Some("Test_Something".to_owned()));
    }

    #[test]
    fn Test_A_Gate_Reached_Through_Two_Helpers_Should_Still_Be_Found()
    {
        let source = r#"
fn Archives() -> Option<PathBuf>
{
    return std::env::var_os("NOMOS_SPEC_ARCHIVES").map(PathBuf::from);
}

fn Both() -> Option<PathBuf>
{
    return Archives();
}

#[test]
fn Test_Compares_Two_Revisions()
{
    let Some(root) = Both() else { return };
}
"#;

        let gates = Gates_In("example.rs", source);
        assert_eq!(gates.len(), 1);
    }

    /// The case that makes naive brace matching wrong. `panic!("{}", ...)` closes a brace
    /// the scanner never opened, and every function after it is misattributed.
    #[test]
    fn Test_A_Brace_Inside_A_String_Should_Not_End_A_Body()
    {
        let source = r#"
#[test]
fn Test_Reads_A_Corpus()
{
    let root = std::env::var_os("NOMOS_V14_CORPUS");
    panic!("} unbalanced {");
}

#[test]
fn Test_Reads_Nothing()
{
    assert!(true);
}
"#;

        let gates = Gates_In("example.rs", source);
        assert_eq!(gates.len(), 1);
        assert_eq!(
            gates.first().map(|gate| return gate.test.clone()),
            Some("Test_Reads_A_Corpus".to_owned())
        );
    }

    /// A doc comment naming a variable is documentation, not a gate.
    #[test]
    fn Test_A_Variable_Named_Only_In_A_Comment_Should_Not_Gate()
    {
        let source = r"
/// Opt-in by NOMOS_V14_CORPUS.
#[test]
fn Test_Runs_Always()
{
    assert!(true); // NOMOS_SPEC_ARCHIVES is not read here
}
";

        assert!(Gates_In("example.rs", source).is_empty());
    }

    #[test]
    fn Test_An_Ungated_Test_Should_Not_Be_Reported()
    {
        let source = r"
#[test]
fn Test_Plain()
{
    assert_eq!(1 + 1, 2);
}
";

        assert!(Gates_In("example.rs", source).is_empty());
    }
}
