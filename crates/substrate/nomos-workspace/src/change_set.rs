//! The name the rest of the workspace knows a change set by.

use crate::workspace_change_set::WorkspaceChangeSet;
/// The name this type is known by where the distinction from [`Change`] is already clear.
pub type ChangeSet = WorkspaceChangeSet;
