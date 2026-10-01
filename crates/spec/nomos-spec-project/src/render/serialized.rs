//! JSON and YAML: the projection serialized whole, behind the generated-file marker.

use crate::GENERATED_FILE_NOTICE;
use crate::ProjectError;
use crate::Projection;
use serde::Serialize;

/// The projection, wrapped in the two fields every generated artifact opens with.
#[derive(Serialize)]
// `clippy::struct_field_names` objects to `nomos_generated` repeating the type's own name. The
// field names here are the serialized keys, not internal names: `nomos_generated: true` is the
// marker every generated artifact opens with — `spec/domain-specification.md` and
// `diagrams/relations.mmd` both carry it, and it is asserted by that exact spelling in
// `read_surface.rs` and `renderers.rs`. Renaming the field to satisfy the lint would rewrite
// every projection this workspace ships.
#[allow(clippy::struct_field_names)]
struct Generated<'a>
{
    nomos_generated: bool,
    do_not_edit: &'a str,
    #[serde(flatten)]
    projection: &'a Projection,
}

pub(super) fn Render_Json(projection: &Projection) -> Result<String, ProjectError>
{
    let generated = Generated {
        nomos_generated: true,
        do_not_edit: GENERATED_FILE_NOTICE,
        projection,
    };
    let mut rendered = serde_json::to_string_pretty(&generated)
        .map_err(|error| return ProjectError::Malformed(error.to_string()))?;
    rendered.push('\n');

    return Ok(rendered);
}

pub(super) fn Render_Yaml(projection: &Projection) -> Result<String, ProjectError>
{
    let generated = Generated {
        nomos_generated: true,
        do_not_edit: GENERATED_FILE_NOTICE,
        projection,
    };

    return serde_yaml_ng::to_string(&generated)
        .map_err(|error| return ProjectError::Malformed(error.to_string()));
}
