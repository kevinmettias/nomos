//! Where one restored node or statement came from in the source.

use serde::{Deserialize, Serialize};

use crate::OrdinalRef;
use crate::TableRowRef;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage
{
    pub source_block: Option<OrdinalRef>,
    pub source_heading: Option<OrdinalRef>,
    pub source_table_row: Option<TableRowRef>,
    pub disposition: String,
    pub target_node_id: Option<String>,
    pub target_statement_id: Option<String>,
}
