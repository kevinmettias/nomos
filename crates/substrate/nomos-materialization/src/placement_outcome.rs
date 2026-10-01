//! What a completed run did to one target.

/// The three things a successful placement can have been.
///
/// Named after what happened to the target rather than after the class that decided it, so a
/// reader of a report can see that a `GeneratedOwned` intent overwrote something without
/// having to re-derive it from the class beside it. The three are not interchangeable: an
/// overwrite destroyed bytes and a creation did not, and a region replacement left bytes in
/// the file that the run did not author.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlacementOutcome
{
    /// The target did not exist and now holds the source.
    Created,
    /// The target existed and now holds the source instead of whatever it held.
    Overwritten,
    /// The target existed, its declared owned region now holds the source, and everything
    /// outside that region is unchanged byte for byte.
    OwnedRegionReplaced,
}
