//! Every value a rule reads from a family a repository may leave undeclared, one constant each.
//!
//! `OD-RULES-011` version 3 decision 5 decided that which optional values a rule reads is declared
//! on its descriptor as the one statement its body also resolves by. These constants are those
//! statements. A row in [`crate::DESCRIPTORS`] names the ones it reads with `Reading`, and the
//! rule's body hands the same constants to its family's resolver, so the row and the body cannot
//! state two different reads -- and a test runs every row's body and holds what it resolved to what
//! its row declares.
//!
//! A limit or a case is read for a language, or repository-wide, and that is part of the read: Go's
//! hard file-size trigger and the repository-wide one read one key for two languages, and are two
//! reads. The four struct-shaped families each have one read, on [`OptionalRead`] itself.

use crate::rule_descriptor::policy_axis::{
    CYCLOMATIC_COMPLEXITY_MAX, EXPORTED_FUNCTION_READ, EXPORTED_METHOD_READ, EXPORTED_TYPE_READ, FIELD_READ, FILE_SIZE_HARD_LINES,
    FILE_SIZE_REVIEW_LINES, GO_EXPORTED_FUNCTION_READ, GO_EXPORTED_METHOD_READ, GO_UNEXPORTED_FUNCTION_READ, GO_UNEXPORTED_METHOD_READ,
    MODULE_READ, NESTING_DEPTH_MAX, PARAMETER_COUNT_MAX, UNEXPORTED_FUNCTION_READ, UNEXPORTED_METHOD_READ, UNEXPORTED_TYPE_READ,
};
use crate::rule_descriptor::{LimitRead, NamingRead, ReadLanguage};
use crate::GO_LANGUAGE;

pub(crate) use crate::rule_descriptor::OptionalRead;

/// The review trigger, repository-wide: `500-lines`.
pub(crate) const FILE_SIZE_REVIEW: LimitRead = LimitRead { axis: &FILE_SIZE_REVIEW_LINES, language: None };

/// The justification trigger, repository-wide: `1500-lines`.
pub(crate) const FILE_SIZE_HARD: LimitRead = LimitRead { axis: &FILE_SIZE_HARD_LINES, language: None };

/// The review trigger, for Go: `five-hundred-line-review-trigger`.
pub(crate) const GO_FILE_SIZE_REVIEW: LimitRead = LimitRead { axis: &FILE_SIZE_REVIEW_LINES, language: Some(GO_LANGUAGE) };

/// The justification trigger, for Go: `one-thousand-line-hard-trigger`.
pub(crate) const GO_FILE_SIZE_HARD: LimitRead = LimitRead { axis: &FILE_SIZE_HARD_LINES, language: Some(GO_LANGUAGE) };

/// The value-parameter cap, repository-wide: `parameter-count`.
pub(crate) const PARAMETER_COUNT: LimitRead = LimitRead { axis: &PARAMETER_COUNT_MAX, language: None };

/// The value-parameter cap, for Go: `go-helpers-package-five-inputs`.
pub(crate) const GO_PARAMETER_COUNT: LimitRead = LimitRead { axis: &PARAMETER_COUNT_MAX, language: Some(GO_LANGUAGE) };

/// The deepest nesting a function may reach, repository-wide: `nesting-depth`.
pub(crate) const NESTING_DEPTH: LimitRead = LimitRead { axis: &NESTING_DEPTH_MAX, language: None };

/// The largest cyclomatic complexity, repository-wide: `cyclomatic-complexity`.
pub(crate) const CYCLOMATIC_COMPLEXITY: LimitRead = LimitRead { axis: &CYCLOMATIC_COMPLEXITY_MAX, language: None };

/// An exported function's case, in each source's own language: `function-naming-convention`.
pub(crate) const EXPORTED_FUNCTION: NamingRead = NamingRead { read: EXPORTED_FUNCTION_READ, language: ReadLanguage::OfEachSource };

/// Every other function's case, in each source's own language: `function-naming-convention`.
pub(crate) const UNEXPORTED_FUNCTION: NamingRead = NamingRead { read: UNEXPORTED_FUNCTION_READ, language: ReadLanguage::OfEachSource };

/// An exported Go method's case: `function-naming-convention`.
///
/// Declared in each source's own language like its function read, and resolved by the body for a Go
/// source alone. A method read's keys hold its function read's, so asked of a language whose methods
/// are not told apart it names nothing its function read does not.
pub(crate) const EXPORTED_METHOD: NamingRead = NamingRead { read: EXPORTED_METHOD_READ, language: ReadLanguage::OfEachSource };

/// Every other Go method's case: `function-naming-convention`, as [`EXPORTED_METHOD`] says.
pub(crate) const UNEXPORTED_METHOD: NamingRead = NamingRead { read: UNEXPORTED_METHOD_READ, language: ReadLanguage::OfEachSource };

/// A module's case, repository-wide: `data-names-stay-lower-snake`.
pub(crate) const MODULE: NamingRead = NamingRead { read: MODULE_READ, language: ReadLanguage::Repository };

/// A field's case, repository-wide: `data-names-stay-lower-snake`.
pub(crate) const FIELD: NamingRead = NamingRead { read: FIELD_READ, language: ReadLanguage::Repository };

/// An exported Go function's case: `exported-functions-use-upper-snake-case`.
pub(crate) const GO_EXPORTED_FUNCTION: NamingRead = NamingRead { read: GO_EXPORTED_FUNCTION_READ, language: ReadLanguage::Language(GO_LANGUAGE) };

/// An exported Go method's case: `exported-functions-use-upper-snake-case`.
pub(crate) const GO_EXPORTED_METHOD: NamingRead = NamingRead { read: GO_EXPORTED_METHOD_READ, language: ReadLanguage::Language(GO_LANGUAGE) };

/// An unexported Go function's case: `unexported-functions-lowercase-only-the-first-letter`.
pub(crate) const GO_UNEXPORTED_FUNCTION: NamingRead = NamingRead { read: GO_UNEXPORTED_FUNCTION_READ, language: ReadLanguage::Language(GO_LANGUAGE) };

/// An unexported Go method's case: `unexported-functions-lowercase-only-the-first-letter`.
pub(crate) const GO_UNEXPORTED_METHOD: NamingRead = NamingRead { read: GO_UNEXPORTED_METHOD_READ, language: ReadLanguage::Language(GO_LANGUAGE) };

/// An exported Go type's case: `types-use-upper-camel-case-lower-camel-case`.
pub(crate) const GO_EXPORTED_TYPE: NamingRead = NamingRead { read: EXPORTED_TYPE_READ, language: ReadLanguage::Language(GO_LANGUAGE) };

/// An unexported Go type's case: `types-use-upper-camel-case-lower-camel-case`.
pub(crate) const GO_UNEXPORTED_TYPE: NamingRead = NamingRead { read: UNEXPORTED_TYPE_READ, language: ReadLanguage::Language(GO_LANGUAGE) };

/// `standards.json`'s `scripting.tooling_language`: `declared-tooling-language-for-scripts`.
pub(crate) const TOOLING_LANGUAGE: OptionalRead = OptionalRead::TOOLING_LANGUAGE;

/// `standards.json`'s goals and their ceiling: `goals-and-parts-line-up`.
pub(crate) const GOALS: OptionalRead = OptionalRead::GOALS;

/// The roots `nomos-standards-corpus.json` declares: `standards-corpus`.
pub(crate) const STANDARDS_CORPUS: OptionalRead = OptionalRead::STANDARDS_CORPUS;

/// The corpus under `tests/contract/requirements/`: `requirement-trace-staleness`.
pub(crate) const REQUIREMENT_TRACE: OptionalRead = OptionalRead::REQUIREMENT_TRACE;
