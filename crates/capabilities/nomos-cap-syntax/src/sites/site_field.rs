//! One field a kind declares.

use super::SiteValueType;

/// A field of a kind: its name and the type its value holds.
///
/// Declared once, in the kind's own declaration, and read from there by the writer, the reader
/// and the typed view a rule uses -- so no provider and no rule spells a field a second time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SiteField
{
    pub name: &'static str,
    pub value: SiteValueType,
}
