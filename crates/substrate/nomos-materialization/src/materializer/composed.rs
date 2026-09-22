//! Replacing a `Composed` target's owned region without touching the free one.
//!
//! `OD-PACKAGE-004`: "Where a renderer exists, it writes only the owned region and leaves
//! the rest byte-for-byte untouched." Byte-for-byte is meant literally here, so the splice
//! is built by *keeping* the parts it must not change rather than by rewriting them: the
//! text before the opening marker, both markers themselves, and the text after the closing
//! marker are carried through as the strings they already were. Only what lies strictly
//! between the two markers is replaced.
//!
//! # Every refusal is the same refusal
//!
//! Four conditions refuse, and each is the case where performing the write anyway would
//! cost the free region:
//!
//! - **A marker that is not there.** The region has no boundary, so the only write available
//!   is the whole file — which is the free region gone.
//! - **A marker that is there twice.** Which occurrence bounds the region is undecidable,
//!   and the wrong choice swallows whatever sits between the two occurrences. Guessing the
//!   first is a guess that looks like a rule.
//! - **A closing marker before an opening one.** The pair names no region.
//! - **A target that does not exist at all** — refused one level up, in [`super::plan`],
//!   because there is no free region to preserve and no markers to write between, and
//!   creating the file would make every byte of it this mechanism's.
//!
//! That is the record's own asymmetry applied to the one class where a write is partial:
//! refusing costs a rerun, and a wrong splice eats prose somebody wrote with no diff to
//! recover it from.

use crate::owned_region::OwnedRegion;
use crate::refusal::Refusal;

/// The target's whole contents with its owned region replaced by `source`.
///
/// # Errors
///
/// Returns the [`Refusal`] naming whichever of the marker conditions above holds.
pub(crate) fn Spliced(previous: &str, region: &OwnedRegion, source: &str) -> Result<String, Refusal>
{
    let opening = region.opening_marker.as_str();
    let closing = region.closing_marker.as_str();

    Sole_Occurrence(previous, opening)?;
    Sole_Occurrence(previous, closing)?;

    let (before, after_opening) = previous
        .split_once(opening)
        .ok_or_else(|| return Refusal::AbsentRegionMarker { marker: opening.to_owned() })?;
    // The closing marker occurs exactly once in the whole text, so failing to find it after
    // the opening one means it sits before it -- the pair names no region.
    let (_, after_closing) = after_opening.split_once(closing).ok_or(Refusal::InvertedRegionMarkers)?;

    return Ok(format!("{before}{opening}{source}{closing}{after_closing}"));
}

/// A marker appears in the target exactly once, or the region it would bound is not one.
fn Sole_Occurrence(previous: &str, marker: &str) -> Result<(), Refusal>
{
    return match previous.matches(marker).count()
    {
        0 => Err(Refusal::AbsentRegionMarker { marker: marker.to_owned() }),
        1 => Ok(()),
        _ => Err(Refusal::RepeatedRegionMarker { marker: marker.to_owned() }),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    const BEFORE: &str = "prose a person wrote\n";
    const OPENING: &str = "<!-- nomos:begin -->";
    const CLOSING: &str = "<!-- nomos:end -->";
    const AFTER: &str = "\nmore prose a person wrote\n";

    fn Region() -> OwnedRegion
    {
        return OwnedRegion::New(OPENING, CLOSING);
    }

    fn Target(owned: &str) -> String
    {
        return format!("{BEFORE}{OPENING}{owned}{CLOSING}{AFTER}");
    }

    #[test]
    fn Test_Only_The_Owned_Region_Should_Change()
    {
        let spliced = Spliced(&Target("\nold table\n"), &Region(), "\nnew table\n").expect("both markers, in order, once each");

        assert_eq!(spliced, Target("\nnew table\n"));
    }

    #[test]
    fn Test_The_Free_Region_Should_Survive_Byte_For_Byte()
    {
        let spliced = Spliced(&Target("\nold\n"), &Region(), "").expect("an empty owned region is still a region");

        assert!(spliced.starts_with(BEFORE), "the text before the opening marker was rewritten: {spliced:?}");
        assert!(spliced.ends_with(AFTER), "the text after the closing marker was rewritten: {spliced:?}");
    }

    #[test]
    fn Test_A_Missing_Opening_Marker_Should_Refuse()
    {
        let target = format!("{BEFORE}{CLOSING}{AFTER}");

        let refusal = Spliced(&target, &Region(), "new").expect_err("there is no region to write into");

        assert_eq!(refusal, Refusal::AbsentRegionMarker { marker: OPENING.to_owned() });
    }

    #[test]
    fn Test_A_Missing_Closing_Marker_Should_Refuse()
    {
        let target = format!("{BEFORE}{OPENING}{AFTER}");

        let refusal = Spliced(&target, &Region(), "new").expect_err("the region has no end");

        assert_eq!(refusal, Refusal::AbsentRegionMarker { marker: CLOSING.to_owned() });
    }

    #[test]
    fn Test_A_Repeated_Marker_Should_Refuse_Rather_Than_Take_The_First()
    {
        let target = format!("{BEFORE}{OPENING}a{CLOSING}b{OPENING}c{CLOSING}{AFTER}");

        let refusal = Spliced(&target, &Region(), "new").expect_err("which pair bounds the region is undecidable");

        assert_eq!(refusal, Refusal::RepeatedRegionMarker { marker: OPENING.to_owned() });
    }

    #[test]
    fn Test_A_Closing_Marker_Before_The_Opening_One_Should_Refuse()
    {
        let target = format!("{BEFORE}{CLOSING}between{OPENING}{AFTER}");

        let refusal = Spliced(&target, &Region(), "new").expect_err("the pair names no region");

        assert_eq!(refusal, Refusal::InvertedRegionMarkers);
    }

    /// The prose between two markers in the wrong order is exactly what a splice that
    /// ignored the order would have eaten, so the refusal above is measured by what survives.
    #[test]
    fn Test_A_Refused_Splice_Should_Return_No_Text_At_All()
    {
        let target = format!("{BEFORE}{CLOSING}between{OPENING}{AFTER}");

        assert!(Spliced(&target, &Region(), "new").is_err());
        assert!(target.contains("between"), "the refusal must not have touched the caller's text");
    }
}
