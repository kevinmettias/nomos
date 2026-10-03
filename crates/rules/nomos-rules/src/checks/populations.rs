//! The population every narrowing rule judges, one constant each.
//!
//! `OD-ANALYSIS-012` version 2 decided that a rule's population is declared on its descriptor
//! as the one statement its body also filters by. These constants are those statements. A
//! narrowing row in [`crate::DESCRIPTORS`] names one with `Judging`, and the rule's body tests
//! each source it is handed against the same constant, so the row and the body cannot state two
//! different populations. A rule whose norm is about any file names none and judges
//! [`Population::Every`].
//!
//! One constant serves several rules only where one filter already served them: the three
//! ordering rules share `concurrency_text`'s, the two scalar-range rules `scalar_range`'s, and so
//! on. Rules in one module that judge different populations get one constant each.
//!
//! What a body leaves out within its population -- its own implementation file, test material, a
//! Go file that is not a test -- is judgment and not population, and stays in the body.

use crate::rule_descriptor::Population;
use crate::{GO_LANGUAGE, RUST_LANGUAGE};

/// Rust, the only language `a-rust-path-stays-within-its-own-subtree`'s `#[path]` attribute exists in.
pub(crate) const PATH_ATTRIBUTE_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose interior-mutability types `shared-interior-mutability-says-why` reads.
pub(crate) const INTERIOR_MUTABILITY_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose `unsafe` blocks `unsafe-justification` reads.
pub(crate) const UNSAFE_JUSTIFICATION_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose test attributes `zero-flake-policy` reads for a retry-until-green one.
pub(crate) const RETRY_UNTIL_GREEN_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust and Go, each with its own sleep vocabulary: `sleep-based-synchronization` judges every
/// Rust source and, within Go, only a test file.
pub(crate) const SLEEP_POPULATION: Population = Population::Languages(&[RUST_LANGUAGE, GO_LANGUAGE]);

/// Rust, whose atomic orderings the three ordering rules read through one filter.
pub(crate) const CONCURRENCY_TEXT_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose field types the two scalar-range rules read through one filter.
pub(crate) const SCALAR_RANGE_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose enum variants `named-fields-over-positional-variant-payloads` reads.
pub(crate) const ENUM_SHAPE_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose error messages the three error-text rules read.
pub(crate) const ERROR_TEXT_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose function bodies `no-single-line-function-bodies` reads.
pub(crate) const SINGLE_LINE_BODY_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose borrowed parameter types `parameters-borrow-unless-ownership-is-taken` reads.
pub(crate) const BORROWED_CONTAINER_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose lifetimes the two lifetime-discipline rules read.
pub(crate) const LIFETIME_DISCIPLINE_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose procedural macros `prefer-macro-rules-over-procedural-macros` reads.
pub(crate) const PROCEDURAL_MACRO_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose function nesting `nesting-depth` measures.
pub(crate) const NESTING_DEPTH_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, the language the complexity family measures.
pub(crate) const CYCLOMATIC_COMPLEXITY_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, whose closure bounds the two closure-bound rules read through one filter.
pub(crate) const CLOSURE_BOUNDS_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Rust, the language a crate's module tree reaches its files in.
pub(crate) const ORPHAN_MODULES_POPULATION: Population = Population::Language(RUST_LANGUAGE);

/// Go, whose `t.Skip` calls `a-skipped-test-states-why` reads.
pub(crate) const SKIPPED_TEST_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Go, whose `//go:build ignore` lines `an-excluded-file-says-why` reads.
pub(crate) const EXCLUDED_FILE_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Go, whose `//nolint` and workspace-marker comments the two marker rules read.
pub(crate) const GO_MARKERS_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Go, whose package clauses `a-package-is-named-after-its-directory` reads.
pub(crate) const PACKAGE_PLACEMENT_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Rust and Go, each with its own import syntax: `no-wildcard-imports` reads both.
pub(crate) const WILDCARD_IMPORT_POPULATION: Population = Population::Languages(&[RUST_LANGUAGE, GO_LANGUAGE]);

/// Go, whose constants and variables the two Go data-name rules read through one filter.
pub(crate) const GO_DATA_NAMES_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Go, whose functions the two Go function-name rules read through one filter.
pub(crate) const GO_FUNCTION_NAMES_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Go, whose type names `types-use-upper-camel-case-lower-camel-case` reads.
pub(crate) const GO_TYPE_NAMES_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Go, whose functions `go-helpers-package-five-inputs` counts the parameters of.
pub(crate) const GO_PARAMETER_COUNT_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// Go, whose files the two Go file-size triggers measure.
pub(crate) const GO_FILE_SIZE_POPULATION: Population = Population::Language(GO_LANGUAGE);

/// A script: a source whose first line is a shebang naming an absolute interpreter path, whatever
/// its extension -- the population the three script rules share, recognized by
/// `script_discipline`'s own predicate.
pub(crate) const SCRIPT_POPULATION: Population = Population::Kind { name: "shebang scripts", holds: super::script_discipline::Is_Shebang_Script };
