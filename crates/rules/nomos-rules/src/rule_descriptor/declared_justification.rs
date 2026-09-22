//! The closed vocabulary of local justifications a declaration may name.
//!
//! Its own file for the reason [`super::DeclaredDetector`] is in its own: it carries the
//! same refusal -- a declaration names a justification and never contains one -- and the
//! refusal belongs to the vocabulary rather than to the declaration naming a member of it.
//!
//! This is the half `checks::rust_text`'s `Unjustified_Construct_Findings_In` already takes:
//! "a construct-matching predicate, an unless-locally-justified predicate, and one finding
//! message", in that engine's own words. A declaration that names no member of this
//! vocabulary is a rule with no local escape at all, which the engine renders as a
//! justification that never holds rather than as a missing field.

/// One shape of local justification a declared rule accepts.
///
/// One member, and that is the measurement rather than a placeholder: the three rules
/// declared through this form carried three separately-written copies of the identical
/// predicate -- `Has_Local_Allow_Justification`, `Has_Local_Inline_Always_Justification` and
/// `Has_Local_Ignore_Justification`, byte-for-byte the same body under three names -- which
/// is precisely the authoring redundancy `OD-RULES-034` decided a declared form removes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclaredJustification
{
    /// Any non-empty comment on the construct's own line, or in the contiguous comment
    /// block immediately above it.
    AnyAdjacentComment,
}

impl DeclaredJustification
{
    /// The reading this member names, in the shape the shared engine drives.
    ///
    /// `index` is the construct's own line and `lines` the whole file, because the engine's
    /// look-back is the contiguous block above one line and nothing wider. Block state,
    /// scope and any other file are outside what this signature can see, which is
    /// `OD-RULES-034`'s block-and-scope refusal expressed as a type rather than as a note.
    pub(crate) fn As_Predicate(self) -> fn(&[&str], usize) -> bool
    {
        return match self
        {
            Self::AnyAdjacentComment => crate::checks::Has_An_Adjacent_Non_Empty_Comment,
        };
    }
}
