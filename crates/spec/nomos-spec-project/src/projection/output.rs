//! One rendered file and the stamp beside it.

use crate::ProjectError;
use crate::Stamp;
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Format;

    fn Sample_Output() -> Output
    {
        return Output {
            path: "one.md".to_owned(),
            body: "# One\n".to_owned(),
            sidecar_path: "one.md.nomos-projection.json".to_owned(),
            stamp: Stamp {
                profile: "one".to_owned(),
                profile_digest: "sha256:aa".to_owned(),
                format: Format::Markdown,
                output: "one.md".to_owned(),
                content_digest: "sha256:bb".to_owned(),
                inputs_digest: "sha256:cc".to_owned(),
                sections: vec![("Nodes".to_owned(), 2)],
                inputs: Vec::new(),
            },
        };
    }

    #[test]
    fn Test_Sidecar_Should_Render_The_Stamp_As_Json()
    {
        let rendered = Sample_Output().Sidecar().expect("renders");

        assert!(rendered.contains("\"profile\": \"one\""), "{rendered}");
    }
}
