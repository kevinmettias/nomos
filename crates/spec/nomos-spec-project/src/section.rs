//! One section of a rendered projection.

use crate::Item;
use crate::Content;
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Section
{
    pub title: String,
    pub content: Content,
    pub items: Vec<Item>,
}
