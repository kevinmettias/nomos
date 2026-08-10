//! One section of a rendered projection.

use crate::item::Item;
use crate::content::Content;
use serde::Serialize;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Section
{
    pub title: String,
    pub content: Content,
    pub items: Vec<Item>,
}
