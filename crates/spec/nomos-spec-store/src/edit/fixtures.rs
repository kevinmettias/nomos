//! A shared setup for the two test modules that each build one edit and then diverge to
//! test their own side of it: `authoring::commit` applies the preview, `edit::staged_edit`
//! asserts on the preview itself.

use crate::{EditError, EditPreview, Seed_Governing_Records, SpecificationStore};

/// The synthetic record both callers stage an edit against.
pub(crate) const CANONICAL: &str = "---\nid: D-900\ntype: decision\ntitle: A synthetic record\n\
                                     status: accepted\nversion: 1\n\
                                     authority: canonical-normative-record\ntags:\n  - testing\n\
                                     relations:\n  - target: D-129\n    type: relates-to\n---\n\n\
                                     # A synthetic record\n\n## Decision\n\nFirst paragraph.\n\n\
                                     ## Rationale\n\nSecond paragraph.\n";
/// Where [`CANONICAL`] is authored.
pub(crate) const PATH: &str = "docs/records/D-900-a-synthetic-record.md";

/// A store already holding [`CANONICAL`] as authored at [`PATH`], paired with one edit
/// already staged and previewed against it — named rather than a bare pair so a caller
/// cannot swap the two.
pub(crate) struct PreviewedEdit
{
    pub(crate) store: SpecificationStore,
    pub(crate) preview: EditPreview,
}

/// A store seeded with governing records, holding [`CANONICAL`] as authored at [`PATH`], with
/// one edit already staged and previewed against it.
///
/// Every fallible step returns rather than unwrapping: this helper is compiled under
/// `cfg(test)` and both callers depend on it, so a refusal is better reported as the value
/// the caller asked for than as a panic raised inside the fixture.
///
/// # Errors
///
/// Returns [`EditError`] if the in-memory schema or the governing seed fails, if the
/// synthetic record's markdown does not parse as a record, if `D-900` does not resolve to
/// exactly one held record, or if the staged edit's target path is not free.
pub(crate) fn Previewed_Edit_Of_D900() -> Result<PreviewedEdit, EditError>
{
    use crate::store::AUTHORED;

    let mut store = SpecificationStore::In_Memory()?;
    Seed_Governing_Records(&mut store)?;
    store.Put_Record(PATH, AUTHORED, CANONICAL)?;
    let edited = CANONICAL.replace("First paragraph.", "First paragraph, edited.");

    let preview = store
        .Claim_For_Edit("D-900", None)?
        .Stage(&edited, None)?
        .Preview(&store)?;

    return Ok(PreviewedEdit { store, preview });
}
