//! One declaration in a syntax payload.

use crate::Observation;
/// One `item` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadItem
{
    /// Position in source order, from zero.
    pub ordinal: u32,
    /// The form the provider recognised. See the module doc: the vocabulary is open.
    pub kind: String,
    /// The visibility the item declares, as the provider observed it.
    pub visibility: String,
    /// The name as written, qualified by syntactic nesting.
    pub qualified_name: String,
    /// The item's documentation, and whether the provider could look for it.
    pub documentation: Observation,
    /// What the item declares, beyond its name — see the module doc's table.
    pub shape: Observation,
}

impl PayloadItem
{
    /// The item's own name, without the nesting it is qualified by.
    ///
    /// The last `::` segment. Matching a qualified form against anything else resolves
    /// nothing, because almost every declaration worth naming lives inside something.
    #[must_use]
    pub fn Own_Name(&self) -> &str
    {
        return self
            .qualified_name
            .rsplit("::")
            .next()
            .unwrap_or(&self.qualified_name);
    }

    /// Whether the item declares itself public.
    #[must_use]
    pub fn Is_Public(&self) -> bool
    {
        use crate::PUBLIC;

        return self.visibility == PUBLIC;
    }

    /// Whether the provider observed a form with no visibility to declare.
    ///
    /// True is an observation. False is not the opposite of it — see the module doc.
    #[must_use]
    pub fn Declares_No_Visibility(&self) -> bool
    {
        use crate::NOT_APPLICABLE;

        return self.visibility == NOT_APPLICABLE;
    }
}
