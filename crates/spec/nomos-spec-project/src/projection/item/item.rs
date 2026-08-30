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

        return ContentHash::Of(&material).As_String_Slice().to_owned();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Of_Should_Create_An_Item_Bearing_Only_Its_Identity()
    {
        let item = Item::Of("CDM-ONE");

        assert_eq!(item.identity, "CDM-ONE");
        assert!(item.fields.is_empty());
        assert!(item.body.is_none());
    }

    #[test]
    fn Test_With_Should_Attach_A_Name_And_Value_To_The_Item()
    {
        let item = Item::Of("CDM-ONE").With(Name("title"), Value("One"));

        assert_eq!(item.Field("title"), Some("One"));
    }

    #[test]
    fn Test_Carrying_Should_Set_The_Item_Body()
    {
        let item = Item::Of("CDM-ONE").Carrying("prose");

        assert_eq!(item.body.as_deref(), Some("prose"));
    }

    #[test]
    fn Test_Field_Should_Return_None_For_A_Name_Nothing_Set()
    {
        let item = Item::Of("CDM-ONE").With(Name("title"), Value("One"));

        assert_eq!(item.Field("missing"), None);
    }

    #[test]
    fn Test_Digest_Should_Change_When_A_Field_Value_Changes()
    {
        let base = Item::Of("CDM-ONE").With(Name("title"), Value("One"));
        let changed = Item::Of("CDM-ONE").With(Name("title"), Value("Two"));

        assert_ne!(base.Digest(), changed.Digest());
    }
}
