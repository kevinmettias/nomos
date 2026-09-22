//! What one pass over a file has accumulated so far.
//!
//! Two numbers travel together through every recorder: the items found, and how many
//! conditional regions were declined. They are carried as one value rather than two `&mut`
//! parameters threaded through a dozen functions, so that a recorder which forgets the second
//! cannot compile — an undercounted header is exactly the shape of a completeness claim
//! nobody could falsify.

use super::super::Facts;
use super::super::Item;
use super::support::ItemRecord;

/// The items and the region count one reading has produced so far.
pub(super) struct Recording
{
    items: Vec<Item>,
    unexpanded: u32,
}

impl Recording
{
    pub(super) fn New() -> Self
    {
        return Self { items: Vec::new(), unexpanded: 0 };
    }

    /// Records one declaration, with what this provider observed about it.
    ///
    /// `shape` is `None` where the form has no shape to describe rather than where none could
    /// be seen. This provider parses, so everything it does not record is an absence it looked
    /// for — the distinction the payload spells `.` rather than `-`.
    pub(super) fn Push(&mut self, record: ItemRecord)
    {
        let ordinal = u32::try_from(self.items.len()).unwrap_or(u32::MAX);

        self.items.push(Item {
            ordinal,
            kind: record.kind,
            scope: record.scope,
            name: record.name,
            visibility: record.visibility,
            documentation: record.documentation,
            shape: record.shape,
        });
    }

    /// Notes one preprocessor conditional region this reading did not enter.
    ///
    /// Saturating, and the saturation is not a formality: a file with more than four billion
    /// conditional regions cannot be counted in the header's `u32`, and reporting a wrapped
    /// count would be reporting a smaller gap than the one that exists.
    pub(super) fn Note_Conditional_Region(&mut self)
    {
        self.unexpanded = self.unexpanded.saturating_add(1);
    }

    pub(super) fn Into_Facts(self) -> Facts
    {
        return Facts { items: self.items, unexpanded: self.unexpanded };
    }
}
