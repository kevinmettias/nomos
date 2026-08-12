//! One record as the manifest names it.

use nomos_contracts::SchemaId;
use serde::{Deserialize, Serialize};

use crate::document::DocumentId;
use crate::document::DocumentKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference
{
    pub kind: DocumentKind,
    pub schema: SchemaId,
    pub document: DocumentId,
}
