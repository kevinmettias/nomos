use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportReport
{
    pub records: u32,
    pub counts: BTreeMap<String, u32>,
}
