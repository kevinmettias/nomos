//! Every format a projection can be rendered as, dispatched from the format it declares.
//!
//! Each format's writer lives in a submodule beside this one. The formats share their entry
//! point and almost nothing else: markdown writes tables where HTML writes articles, mermaid
//! writes a graph where a context pack writes a budgeted subset, and the two text transforms
//! all of them need are in `text`. What is left here is the dispatch, which is the one thing
//! that has to see every format at once.

mod html;
mod markdown;
mod mermaid;
mod pack;
mod serialized;
mod text;

use crate::Projection;
use crate::ProjectError;
use html::Render_Html;
use markdown::Render_Markdown;
use mermaid::Render_Mermaid;
use pack::Render_Contextpack;
use serialized::{Render_Json, Render_Yaml};

pub fn Render_Projection(projection: &Projection) -> Result<String, ProjectError>
{
    use crate::Format;

    return match projection.format
    {
        Format::Markdown => Ok(Render_Markdown(projection)),
        Format::Html => Ok(Render_Html(projection)),
        Format::Mermaid => Ok(Render_Mermaid(projection)),
        Format::Json => Render_Json(projection),
        Format::Yaml => Render_Yaml(projection),
        Format::Contextpack => Render_Contextpack(projection),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{Content, Format, Item, Name, Section, Value};

    #[test]
    fn Test_Render_Projection_Should_Dispatch_To_The_Format_The_Projection_Declares()
    {
        let mut projection = One_Section_Projection();
        let markdown = Render_Projection(&projection).expect("renders");

        assert!(markdown.contains("# One"), "{markdown}");
        assert!(markdown.contains("## Nodes"), "{markdown}");

        projection.format = Format::Json;
        let json = Render_Projection(&projection).expect("renders");

        assert!(json.contains("\"title\": \"One\""), "{json}");
    }

    fn One_Section_Projection() -> Projection
    {
        return Projection {
            profile: "one".to_owned(),
            title: "One".to_owned(),
            format: Format::Markdown,
            output: "one.md".to_owned(),
            sections: vec![Section {
                title: "Nodes".to_owned(),
                content: Content::Nodes,
                items: vec![Item::Of("CDM-ONE").With(Name("title"), Value("One"))],
            }],
            inputs: Vec::new(),
        };
    }
}
