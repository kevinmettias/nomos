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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{NOT_APPLICABLE, Observation, PUBLIC};

    fn Item_With(visibility: &str, qualified_name: &str) -> PayloadItem
    {
        return PayloadItem {
            ordinal: 0,
            kind: "Function".to_owned(),
            visibility: visibility.to_owned(),
            qualified_name: qualified_name.to_owned(),
            documentation: Observation::Absent,
            shape: Observation::Absent,
        };
    }

    #[test]
    fn Test_Own_Name_Should_Be_The_Last_Segment_Of_A_Qualified_Name()
    {
        assert_eq!(Item_With(PUBLIC, "Table::All").Own_Name(), "All");
        assert_eq!(Item_With(PUBLIC, "All").Own_Name(), "All");
    }

    #[test]
    fn Test_Is_Public_Should_Be_True_Only_For_The_Public_Label()
    {
        assert!(Item_With(PUBLIC, "All").Is_Public());
        assert!(!Item_With("Private", "All").Is_Public());
        assert!(!Item_With(NOT_APPLICABLE, "All").Is_Public());
    }

    #[test]
    fn Test_Declares_No_Visibility_Should_Be_True_Only_For_The_Not_Applicable_Label()
    {
        assert!(Item_With(NOT_APPLICABLE, "All").Declares_No_Visibility());
        assert!(!Item_With(PUBLIC, "All").Declares_No_Visibility());
        assert!(!Item_With("Private", "All").Declares_No_Visibility());
    }
}
