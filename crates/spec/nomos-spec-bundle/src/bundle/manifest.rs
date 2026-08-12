//! What a bundle claims to hold, and the digest that proves it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest
{
    pub records: u32,
    /// Rows per table. A `BTreeMap` because a `HashMap` would order the bundle by
    /// whatever the hasher felt like, and the bundle's whole value is that it diffs.
    pub counts: BTreeMap<String, u32>,
    /// Over the header and every record line, each terminated by a newline.
    pub digest: String,
}
