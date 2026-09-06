//! No member manifest describes itself by a numeric band any more.
//!
//! `OD-RULES-020` replaced numeric bands with named zones, and `P41-ZONES-MIGRATION-3`
//! carried the migration through `README.md`, `nomos-rules`' own `ZONES` and the
//! dependency-graph tests that read them. It never touched the one place a band number
//! also lived: a workspace member's own `Cargo.toml` `description`, which every crate had
//! opened with a sentence like `"Band 40. ..."` since before zones existed. Fifty-seven of them
//! still did, restating an ordering the rest of the workspace had already stopped
//! believing in — exactly the kind of stale architectural language `AGENTS.md` names as
//! something a reader should not have to mentally translate.
//!
//! This is a phrase check, the same shape `band_zero.rs` already uses for a narrower
//! claim: it cannot tell whether a *new* crate's description is well written, only that
//! none of them opens with a number the architecture no longer orders by.

/// The prefix a description can no longer open with.
const STALE_PREFIX: &str = "description = \"Band ";

#[test]
fn Test_No_Member_Manifest_Should_Open_Its_Description_With_A_Numeric_Band()
{
    use nomos_contract_tests::Workspace;

    let workspace = Workspace::Load();
    let mut offending = Vec::new();
    for member in workspace.Members()
    {
        let manifest = member.root.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest.display()));

        if Opens_With_A_Numeric_Band(&text)
        {
            offending.push(member.name.clone());
        }
    }

    offending.sort();

    assert!(
        offending.is_empty(),
        "{offending:?} still describe themselves by a numeric band. Zones replaced bands \
         everywhere else this workspace states its own architecture; a manifest that still \
         opens with one is the stale label `OD-RULES-020` and `P41-ZONES-MIGRATION-3` were \
         meant to retire, left where nothing read it back."
    );
}

/// Whether `manifest_text`'s `description` line opens with `STALE_PREFIX` followed by a
/// digit — a real band number, not merely the word appearing somewhere in prose.
fn Opens_With_A_Numeric_Band(manifest_text: &str) -> bool
{
    return manifest_text
        .lines()
        .any(|line| line.starts_with(STALE_PREFIX) && line[STALE_PREFIX.len()..].starts_with(|character: char| character.is_ascii_digit()));
}

#[cfg(test)]
mod tests
{
    use super::Opens_With_A_Numeric_Band;

    #[test]
    fn Test_A_Numeric_Band_Opening_Should_Be_Detected()
    {
        assert!(Opens_With_A_Numeric_Band("description = \"Band 40. Composes something.\"\n"));
    }

    #[test]
    fn Test_A_Lettered_Band_Opening_Should_Still_Be_Detected()
    {
        assert!(Opens_With_A_Numeric_Band("description = \"Band 1p. Port traits.\"\n"));
    }

    #[test]
    fn Test_A_Cleaned_Description_Should_Not_Be_Detected()
    {
        assert!(!Opens_With_A_Numeric_Band("description = \"Composes something.\"\n"));
    }

    #[test]
    fn Test_The_Word_Band_Elsewhere_In_Prose_Should_Not_Be_Detected()
    {
        assert!(!Opens_With_A_Numeric_Band(
            "description = \"Reads a Band 40 crate's own facts.\"\n"
        ));
    }
}
