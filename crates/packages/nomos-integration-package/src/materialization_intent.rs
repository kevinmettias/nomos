//! One placement an `IntegrationPackage` declares, and never performs.

use crate::ownership_class::OwnershipClass;
use crate::publication_scope::PublicationScope;

/// A declared placement: what lands where, under which ownership class and which
/// publication scope.
///
/// `OD-PACKAGE-003`'s "Declaring A Placement Is Not Performing One" names exactly these four
/// facts -- source, target path, `OD-PACKAGE-004`'s ownership class, `OD-PACKAGE-005`'s
/// publication scope -- as what an `IntegrationPackage` declares and hands to a generic
/// materializer. Both records place the classification with the thing doing the placing
/// rather than inside the target file, which is why the class and the scope are fields here
/// and not markers a materializer would read back out of the bytes it is about to overwrite.
///
/// The surface label is text on purpose. `OD-PACKAGE-003` leaves open whether its placement
/// table's rows should become a typed `IntegrationSurfaceKind`, and defers that to whoever
/// builds the second `IntegrationPackage`; this is the first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationIntent
{
    /// Which of `OD-PACKAGE-003`'s surfaces this places, as a free label -- "agent contract
    /// file", "per-agent adapter", "procedural skills" -- rather than a closed vocabulary.
    pub surface: String,
    /// Where the bytes come from, as the manifest spelled it. This crate does not interpret
    /// it: resolving a source to content is the materializer's concern, and `ARCH-006` (via
    /// `OD-PACKAGE-003`) says the content was authored or rendered by something other than
    /// the `IntegrationPackage` in any case.
    pub source: String,
    /// Where the bytes land, relative to the root of the repository being integrated.
    /// [`crate::Parse_Manifest`] refuses a value that is absolute or that climbs above that
    /// root before a value is constructed, and keeps the spelling the manifest used.
    pub target: String,
    /// What regeneration may do to the target once placed. `OD-PACKAGE-004`; an undeclared
    /// class resolves to [`OwnershipClass::UNDECLARED`].
    pub ownership_class: OwnershipClass,
    /// Whether the target may enter repository-distributed state. `OD-PACKAGE-005`; an
    /// undeclared scope resolves to [`PublicationScope::UNDECLARED`].
    pub publication_scope: PublicationScope,
}
