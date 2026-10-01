//! A table that cannot be read as one.

/// A table that cannot be read as one.
///
/// Reported rather than repaired. A run of pipe lines with no delimiter is not a table,
/// and one with two delimiters has a second header nothing names — either way, guessing
/// which rows carry content is how a content row gets typed out of the preservation
/// rule's view without leaving the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Defect
{
    NoSeparator
    {
        table_ordinal: u32,
        rows: u32,
    },
    ManySeparators
    {
        table_ordinal: u32,
        separators: u32,
    },
}

impl core::fmt::Display for Defect
{
    // `fmt` is the fixed method name `std::fmt::Display` mandates; it is not a free choice
    // of abbreviation and cannot be spelled out without ceasing to implement the trait.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::NoSeparator { table_ordinal, rows } => write!(
                formatter,
                "table {table_ordinal} has {rows} row(s) and no delimiter, so no row is a header"
            ),
            Self::ManySeparators {
                table_ordinal,
                separators,
            } => write!(
                formatter,
                "table {table_ordinal} has {separators} delimiters, so where its header stops is undecided"
            ),
        };
    }
}
