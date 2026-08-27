//! One row of a rendered projection.

use serde::Serialize;

use super::{Name, Value};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Item
{
    pub identity: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<(String, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl Item
{
    #[must_use]
    pub fn Of(identity: &str) -> Self
    {
        return Self {
            identity: identity.to_owned(),
            fields: Vec::new(),
            body: None,
        };
    }

    #[must_use]
    pub fn With(mut self, name: Name<'_>, value: Value<'_>) -> Self
    {
        self.fields.push((name.0.to_owned(), value.0.to_owned()));

        return self;
    }

    #[must_use]
    pub fn Carrying(mut self, body: &str) -> Self
    {
        self.body = Some(body.to_owned());

        return self;
    }

    #[must_use]
    pub fn Field(&self, name: &str) -> Option<&str>
    {
        return self
            .fields
            .iter()
            .find(|(field, _)| return field == name)
            .map(|(_, value)| return value.as_str());
    }

    #[must_use]
    pub fn Digest(&self) -> String
    {
        use nomos_spec_model::ContentHash;

        let mut material = self.identity.clone();
        for (name, value) in &self.fields
        {
            material.push('\u{1f}');
            material.push_str(name);
            material.push('\u{1f}');
            material.push_str(value);
        }
        if let Some(body) = &self.body
        {
            material.push('\u{1e}');
            material.push_str(body);
        }

        return ContentHash::Of(&material).As_Str().to_owned();
    }
}
