//! The property every label-bearing enum in this crate pins, asserted in one place.
//!
//! Each such enum keeps its own `Test_Label_Should_Spell_Every_Variant_Distinctly` in its
//! own module, beside the variants it names; what moved here is the sequence all six of
//! them spelled out with nothing changed but the type.

use alloc::vec::Vec;

/// Asserts that no two of `labels` share a spelling, naming `what` in the failure message.
pub(crate) fn Assert_Labels_Distinct<'label>(labels: impl IntoIterator<Item = &'label str>, what: &str)
{
    let mut labels: Vec<&str> = labels.into_iter().collect();
    let count = labels.len();
    labels.sort_unstable();
    labels.dedup();

    assert_eq!(labels.len(), count, "two {what} share a wire spelling");
}
