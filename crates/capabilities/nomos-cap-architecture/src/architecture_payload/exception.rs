//! One named pair the component order alone cannot express.

/// One `this package may name that one` statement, for two packages in the same component.
///
/// Deliberately a separate type from [`super::permission::Permission`] although the two carry
/// the same two strings. A permission is between components and an exception is between
/// packages, and the identical reasoning `OD-CAPABILITY-008` gives for not sharing an encoder
/// across two providers applies to a shape: making them one type would make a caller that
/// confused the two compile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exception
{
    pub from: String,
    pub to: String,
}
