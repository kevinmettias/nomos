//! How much of a revision says nothing.

use crate::archive::template::Template;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FillerCensus
{
    pub templates: Vec<Template>,
    pub declared: Vec<String>,
    pub stubs: Vec<String>,
}

impl FillerCensus
{
    #[must_use]
    pub fn Undeclared(&self) -> Vec<&Template>
    {
        return self
            .templates
            .iter()
            .filter(|template| return template.declared.is_none())
            .collect();
    }

    #[must_use]
    pub fn Widest_Undeclared(&self) -> Option<&Template>
    {
        return self.Undeclared().into_iter().next();
    }
}
