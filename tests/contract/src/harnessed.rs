//! Which declarations the integration harness actually measures.
//!
//! The third derivation, and the one that reads the question from the test side. A
//! declaration this set does not contain is a promise with no test behind it, and a member
//! of this set that no crate declares is a test measuring something that no longer exists —
//! both directions matter, and neither can be seen from inside the harness, which is why
//! the observer reads it from outside.

use crate::Workspace;
use std::collections::BTreeSet;

/// The strategies the integration harness actually holds to their declarations.
///
/// Read from the harness's own source, by the turbofish it registers a domain with.
///
/// # Panics
///
/// Panics if the dependency graph cannot be read, by way of [`Workspace::Load`].
#[must_use]
pub fn Harnessed_Strategies() -> BTreeSet<String>
{
    use crate::reading::source_files::Source_Files;

    let workspace = Workspace::Load();
    let mut found = BTreeSet::new();
    let Some(harness) = workspace.Get("nomos-integration-tests")
    else
    {
        return found;
    };
    for file in Source_Files(&harness.root.join("tests"))
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };
        let registered = Registered_In(&text);

        found.extend(registered);
    }

    return found;
}

/// Every `Check::<Strategy>` in one file.
///
/// Assembled from its parts for the reason the fact type is: this file would otherwise
/// find itself if the harness and the observer ever shared a directory, and the two have
/// no rule keeping them apart.
fn Registered_In(text: &str) -> Vec<String>
{
    let marker = concat!("Check", "::<");
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(marker)
    {
        let Some(after) = rest.get(start.saturating_add(marker.len())..)
        else
        {
            break;
        };
        let Some((inside, remainder)) = after.split_once('>')
        else
        {
            break;
        };
        let named = Strategy_Name(inside);

        found.extend(named);
        rest = remainder;
    }

    return found;
}

/// The strategy a registration names, if it names one.
///
/// A single-letter name is the generic parameter of the registering function itself, not a
/// domain. Reporting `S` as a strategy would put a name in the harnessed set that no crate
/// can ever declare.
fn Strategy_Name(inside: &str) -> Option<String>
{
    let name = inside.trim();
    let named = name.len() > 1 && name.chars().all(|letter| return letter.is_alphanumeric());

    return named.then(|| return name.to_owned());
}
