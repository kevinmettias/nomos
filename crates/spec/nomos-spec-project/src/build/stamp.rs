use crate::{Format, Input, ProjectError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp
{
    pub profile: String,
    pub profile_digest: String,
    pub format: Format,
    pub output: String,
    pub content_digest: String,
    pub inputs_digest: String,
    pub sections: Vec<(String, u32)>,
    pub inputs: Vec<Input>,
}

impl Stamp
{
    pub fn Parse(text: &str) -> Result<Self, ProjectError>
    {
        return serde_json::from_str(text)
            .map_err(|error| return ProjectError::Malformed(error.to_string()));
    }

    pub fn Render(&self) -> Result<String, ProjectError>
    {
        let mut rendered = serde_json::to_string_pretty(self)
            .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
        rendered.push('\n');

        return Ok(rendered);
    }
}
