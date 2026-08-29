//! A source document, named the way a person names one.

use serde::{Deserialize, Serialize};

/// A source document, named the way a person names one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRef
{
    pub path: String,
    pub revision: String,
}
