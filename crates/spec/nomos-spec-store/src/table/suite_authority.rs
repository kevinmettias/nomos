//! Whether a suite is this repository's own specification or one it merely references.

mod suite_authority;
mod suite_id;
mod suite_title;

pub use suite_authority::SuiteAuthority;
pub use suite_id::SuiteId;
pub use suite_title::SuiteTitle;
