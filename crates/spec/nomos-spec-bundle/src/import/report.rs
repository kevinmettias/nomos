use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report
{
    pub records: u32,
    pub counts: BTreeMap<String, u32>,
}
