use serde::{Deserialize, Serialize};

/// What kind of non-code entity a [`Resource`](super::Resource) is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind
{
    /// A network endpoint.
    Endpoint,
    /// A message channel, topic or queue.
    Channel,
    /// A database or table.
    DataStore,
    /// A file treated as data rather than as source.
    DataFile,
    /// A device.
    Device,
    /// A service outside the workspace.
    ExternalService,
}
