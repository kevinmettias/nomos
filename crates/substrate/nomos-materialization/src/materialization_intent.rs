//! One placement a package declares, and that this crate performs.

use crate::owned_region::OwnedRegion;
use crate::ownership_class::OwnershipClass;
use crate::publication_scope::PublicationScope;

/// A declared placement: what lands where, under which ownership class and which
/// publication scope.
///
/// `OD-PACKAGE-003`'s "Declaring A Placement Is Not Performing One" names exactly these
/// facts -- source, target path, `OD-PACKAGE-004`'s ownership class, `OD-PACKAGE-005`'s
/// publication scope -- as what a package declares and hands to a generic materializer.
/// Both records place the classification with the thing doing the placing rather than inside
/// the target file, which is why the class and the scope are fields here and not markers
/// [`crate::Materialize`] would read back out of the bytes it is about to overwrite.
///
/// Nothing here says which kind of package declared it, and nothing in this crate can ask.
/// A second package kind that materializes something fills in this same shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationIntent
{
    /// Which surface this places, as a free label -- "agent contract file", "per-agent
    /// adapter", "procedural skills" -- rather than a closed vocabulary. Carried through to
    /// the report so a multi-target run is legible; nothing here reads it.
    pub surface: String,
    /// Where the bytes come from, relative to the source root the caller names. Read as one
    /// file: a source naming a directory is refused rather than walked.
    pub source: String,
    /// Where the bytes land, relative to the target root the caller names. A value that is
    /// absolute or that climbs above that root is refused before any write.
    pub target: String,
    /// What regeneration may do to the target. `OD-PACKAGE-004`; an undeclared class
    /// resolves to [`OwnershipClass::UNDECLARED`].
    pub ownership_class: OwnershipClass,
    /// Whether the target may enter repository-distributed state. `OD-PACKAGE-005`; an
    /// undeclared scope resolves to [`PublicationScope::UNDECLARED`]. Reported and never
    /// acted on here.
    pub publication_scope: PublicationScope,
    /// Where the owned region sits, for an [`OwnershipClass::Composed`] target and only for
    /// one.
    ///
    /// Required there and refused elsewhere, both for the same reason: a `Composed` intent
    /// with no region tells the mechanism nothing about which bytes are its own, and a
    /// region declared against a class that has no region is a misdeclaration whose silent
    /// reading would overwrite a whole file that somebody meant to protect half of.
    pub owned_region: Option<OwnedRegion>,
}
