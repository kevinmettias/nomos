//! Why a run was refused before it wrote anything.

/// One reason a declared intent cannot be performed.
///
/// Every variant is a condition under which writing would either touch a path this
/// mechanism has no claim on, or lose bytes it cannot prove belong to the declared source.
/// `OD-PACKAGE-004` fixes the direction for all of them: refusing costs a rerun, and
/// overwriting costs work that cannot be recovered.
///
/// A refusal ends the whole run, not one target of it. A multi-target intent that wrote
/// three files and refused the fourth would leave the tree in a state no declaration
/// describes, and `UserOwned` specifically means "never written, under any circumstance" —
/// which a partial run would satisfy for its own target and violate for the reader who has
/// to work out what actually happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal
{
    /// The target is `UserOwned`. `OD-PACKAGE-004`: "Never happens, under any circumstance.
    /// No package, no installer, no renderer, no profile writes this path." This is also the
    /// class an intent that declared none resolves to, so an unrecognized asset lands here.
    UserOwnedTarget,
    /// The target is anchored outside the tree being written: a leading separator or a drive
    /// prefix.
    AbsoluteTarget,
    /// The target climbs above the root it is relative to.
    EscapingTarget,
    /// The source is anchored outside the tree being read.
    AbsoluteSource,
    /// The source climbs above the root it is relative to.
    EscapingSource,
    /// The source did not read: absent, denied, or a directory rather than a file.
    UnreadableSource
    {
        /// What the filesystem said.
        cause: String,
    },
    /// The target exists and did not read, so what a rollback would have to restore is
    /// unknown and the write cannot be undone.
    UnreadableTarget
    {
        /// What the filesystem said.
        cause: String,
    },
    /// The target is `Composed` and declares no owned region, so nothing says which bytes
    /// the declared source owns.
    UndeclaredOwnedRegion,
    /// An owned region is declared for a target whose class has no owned region. Reading it
    /// anyway would mean guessing which of the two declarations was meant.
    OwnedRegionOnAnUncomposedTarget,
    /// The target is `Composed` and does not exist. There is no free region to preserve and
    /// no markers to write between; creating the file would make every byte of it this
    /// mechanism's, which is the one thing `Composed` says it is not.
    AbsentComposedTarget,
    /// A declared marker does not appear in the target. Writing would have to replace the
    /// whole file, which is the free region lost.
    AbsentRegionMarker
    {
        /// The marker that was not found.
        marker: String,
    },
    /// A declared marker appears in the target more than once, so which occurrence bounds
    /// the owned region is undecidable and a wrong choice swallows prose either side of it.
    RepeatedRegionMarker
    {
        /// The marker that appears more than once.
        marker: String,
    },
    /// The closing marker appears before the opening one, so the region they name is not a
    /// region.
    InvertedRegionMarkers,
}

impl core::fmt::Display for Refusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::UserOwnedTarget => formatter.write_str("its ownership class is UserOwned, which is never written"),
            Self::AbsoluteTarget => formatter.write_str("its target is absolute rather than relative to the tree being written"),
            Self::EscapingTarget => formatter.write_str("its target climbs above the tree being written"),
            Self::AbsoluteSource => formatter.write_str("its source is absolute rather than relative to the tree being read"),
            Self::EscapingSource => formatter.write_str("its source climbs above the tree being read"),
            Self::UnreadableSource { cause } => write!(formatter, "its source did not read: {cause}"),
            Self::UnreadableTarget { cause } => write!(formatter, "its target exists and did not read, so no write could be undone: {cause}"),
            Self::UndeclaredOwnedRegion => formatter.write_str("it is Composed and declares no owned region"),
            Self::OwnedRegionOnAnUncomposedTarget => formatter.write_str("it declares an owned region and is not Composed"),
            Self::AbsentComposedTarget => formatter.write_str("it is Composed and its target does not exist"),
            Self::AbsentRegionMarker { marker } => write!(formatter, "its target carries no `{marker}`"),
            Self::RepeatedRegionMarker { marker } => write!(formatter, "its target carries `{marker}` more than once"),
            Self::InvertedRegionMarkers => formatter.write_str("its closing marker appears before its opening one"),
        };
    }
}
