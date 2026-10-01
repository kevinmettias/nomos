use serde::{Deserialize, Serialize};

use super::{EntityId, ResourceKind};

/// A non-code entity a rule or metric can address.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource
{
    /// Stable identity.
    pub id: EntityId,
    /// What kind of resource this is.
    pub kind: ResourceKind,
    /// How the resource is addressed, in its own domain's terms.
    pub locator: String,
}
