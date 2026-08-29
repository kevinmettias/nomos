#![forbid(unsafe_code)]
// Every fallible function in this crate returns `ProjectError` and nothing else. Each of its
// variants already carries the failure and the remedy — in a doc comment above the variant and
// in the `Display` arm below, which are what a caller actually reads. An `# Errors` section per
// function would be a second, unchecked copy of that one enumeration, so the lint is declined
// for the whole crate rather than answered file by file.
#![allow(clippy::missing_errors_doc)]

mod build;
mod catalogue;
mod determinism;
mod profile;
mod project_error;
mod projection;
mod render;
mod section;
mod select;

pub use build::{Build, Check, Freshness, SIDECAR_SUFFIX, Stamp};
pub use catalogue::{Catalogue, SHIPPED, Shipped};
pub use determinism::ProjectionOutput;
pub use profile::{Profile, SUBJECT};
// Named for the boundary rather than for its module, because two different types here are
// a Section and both are exported: `section::Section` is a section of a rendered projection,
// and this one is a section as a profile declares it. Aliasing at the facade would have made
// the exported name a thing no declaration says, so the declaration says it.
pub use profile::ProfileSection;
pub use project_error::ProjectError;
pub use projection::{Content, GENERATED_FILE_NOTICE, Filter, Format, Input, Item, Name, Output, Projection, Value};
pub use section::Section;
pub use render::Render_Projection;
pub use select::Select_Projection;
