//! Parsing the doc-comment marker a struct declares a correspondence behind —
//! `OD-CAPABILITY-010`'s own choice, the identical marker shape `nomos-rules::universe`'s
//! own `Mirrored by` already uses for a declared mirror.
//!
//! A pure function of an already-read doc comment, grouped apart from `reading.rs` (how the
//! comment gets read at all) and `comparison.rs` (what happens once two structs are paired)
//! for the same reason `crate::naming::violations` and `crate::dependency::completeness`
//! both already are: testable against hand-built strings, no registry, no store, no reader.

/// The doc-comment marker a struct declares a correspondence behind — `OD-CAPABILITY-010`'s
/// own choice, the identical shape `nomos-rules::universe`'s `Mirrored by` already uses.
const CORRESPONDS_TO_MARKER: &str = "Corresponds to ";

/// The struct name one `documentation` observation declares a correspondence to, if it
/// declares one — the identical parsing `nomos_rules::universe`'s own `Claimed_Mirror`
/// already uses for `Mirrored by`, applied to this rule's own marker.
pub(super) fn Declared_Correspondence(documentation: Option<&str>) -> Option<String>
{
    for line in documentation?.lines()
    {
        let (_, after) = line.split_once(CORRESPONDS_TO_MARKER)?;
        let quoted = after.strip_prefix('`')?;
        let (name, _) = quoted.split_once('`')?;

        if !name.trim().is_empty()
        {
            return Some(name.trim().to_owned());
        }
    }

    return None;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Declared_Correspondence_Requires_A_Nonempty_Backticked_Name()
    {
        assert_eq!(Declared_Correspondence(Some("Corresponds to `Counter`.")), Some("Counter".to_owned()));
        assert_eq!(Declared_Correspondence(Some("Corresponds to nothing in particular.")), None);
        assert_eq!(Declared_Correspondence(Some("Corresponds to ``.")), None);
        assert_eq!(Declared_Correspondence(None), None);
    }
}
