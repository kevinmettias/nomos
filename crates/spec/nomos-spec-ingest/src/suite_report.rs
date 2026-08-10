//! What ingesting one sibling suite found.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SuiteReport
{
    pub suite: String,
    pub documents: u32,
    pub blocks: u32,
    /// Record identifiers, named rather than counted.
    pub records: Vec<String>,
    /// Schema node identifiers: the files declaring a shape.
    pub schemas: Vec<String>,
    /// The rest of the machine layer: instance documents, not schemas.
    pub machine_documents: Vec<String>,
    /// Identifiers another suite already owns, so this suite did not take them.
    ///
    /// Named rather than merged. Two suites declaring one identifier is a real ecosystem
    /// problem and reassigning the node would hide it by making the last writer right.
    pub contested: Vec<String>,
}
