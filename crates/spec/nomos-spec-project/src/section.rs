//! One section of a rendered projection.

use crate::projection::Item;
use crate::projection::Content;
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Section
{
    pub title: String,
    pub content: Content,
    pub items: Vec<Item>,
}
