//! One rendered file and the stamp beside it.

use crate::ProjectError;
use crate::build::Stamp;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output
{
    pub path: String,
    pub body: String,
    pub sidecar_path: String,
    pub stamp: Stamp,
}

impl Output
{
    pub fn Sidecar(&self) -> Result<String, ProjectError>
    {
        return self.stamp.Render();
    }
}
