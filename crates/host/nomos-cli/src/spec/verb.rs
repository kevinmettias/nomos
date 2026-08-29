//! One module per `nomos spec` verb, each doing the work its request describes.
//!
//! Grouped rather than flat because this level was mixing the frame with the verbs: parsing
//! arguments, the exit codes, the dispatch and the refusal reporting are what every verb
//! runs inside, and the seven below are what runs. A reader looking for what `render` does
//! had to pick it out of a list that also held how a refusal becomes a line.
//!
//! Each still reads the frame by naming it -- `crate::spec::Channels`, `crate::spec::ExitCode`
//! -- rather than through a `super` that now means something narrower than it used to.

mod editing;
mod freshness;
mod listing;
mod markdown;
mod record;
mod render;
mod table;

pub(super) use editing::{Commit_Edit, Preview_Edit, Report_Edit_Error};
pub(super) use freshness::Freshness_Of;
pub(super) use listing::{Empty_Section, EmptySection, List_Profiles, List_Sources};
pub(super) use markdown::Render_Markdown;
pub(super) use record::Read_Record;
pub(super) use render::{No_Such_Profile, Render_Profile, Report_Build_Error};
pub(super) use table::Read_Table;
