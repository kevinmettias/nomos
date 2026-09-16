//! How much of a revision says nothing.

use crate::Template;
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The reach every template this module's fixtures build carries. No assertion here reads
    /// it — these tests are about `declared`, not about reach — so it is the fixture's own
    /// choice of a count, held in one place rather than repeated per fixture.
    const SECTIONS_IN_A_FIXTURE_TEMPLATE: u32 = 4;

    fn Template_Named(text: &str, declared: Option<&'static str>) -> Template
    {
        return Template {
            text: text.to_owned(),
            sections: SECTIONS_IN_A_FIXTURE_TEMPLATE,
            documents: vec!["a.md".to_owned()],
            declared,
        };
    }

    #[test]
    fn Test_Undeclared_Should_Keep_Only_Templates_With_No_Declared_Pattern()
    {
        let census = FillerCensus {
            templates: vec![
                Template_Named("known filler", Some("known filler")),
                Template_Named("mystery block", None),
            ],
            declared: Vec::new(),
            stubs: Vec::new(),
        };

        let undeclared = census.Undeclared();

        assert_eq!(undeclared.len(), 1);
        assert_eq!(undeclared.first().map(|template| template.text.as_str()), Some("mystery block"));
    }

    #[test]
    fn Test_Widest_Undeclared_Should_Return_The_First_Undeclared_Template()
    {
        let census = FillerCensus {
            templates: vec![Template_Named("first widest", None), Template_Named("second", None)],
            declared: Vec::new(),
            stubs: Vec::new(),
        };

        assert_eq!(
            census.Widest_Undeclared().map(|template| template.text.as_str()),
            Some("first widest")
        );

        let empty = FillerCensus::default();
        assert!(empty.Widest_Undeclared().is_none());
    }
}
