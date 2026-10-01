//! [`DiscardedValuesPayload`], one file's answer.

use super::discarded_value::DiscardedValue;

/// Every value one Go file assigns to `_`, in position order.
///
/// A file that discards nothing answers with no value -- a real, examined answer, not the same fact
/// as a file nobody examined.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscardedValuesPayload
{
    pub values: Vec<DiscardedValue>,
}
