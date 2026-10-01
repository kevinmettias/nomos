//! Zone: Provider — which branches of a C# file's `#if` chains a named build compiles, for
//! `nomos.cap.csharp.conditional_compilation`.
//!
//! # The question, and why this one
//!
//! `nomos-lang-csharp` reads C# on its face, and its own guarantee names where that stops: a
//! `#if`/`#elif`/`#else` chain holds every branch's declarations at once, and nothing in one file
//! says which the compiler reads, so that provider declines to read inside any of them and
//! declares its completeness `Unknown` for it. Which branch compiles depends on the preprocessor
//! symbols the build defines -- a configuration's `DEBUG`, a framework's `NET8_0_OR_GREATER`, a
//! project's `DefineConstants` -- and a parse tree cannot know them.
//!
//! That is the question a rule asks first: before any rule can say whether a declaration exists
//! in the build that ships, or that a branch no declared build ever compiles is dead, something
//! has to say which branches a build compiles. It is chosen for that reason and not because it
//! was the easiest reachable one. Its answer is also what would let the syntax provider read
//! inside the compiled branches of a named build and revisit its completeness -- a consequence
//! this crate states and does not perform. Resolved symbols, effective accessibility and partial
//! completion are the next questions of the same kind, each its own capability when a rule asks.
//!
//! # The mechanism, named
//!
//! Not Roslyn, and not a language server. Two things, each the authority for its half:
//!
//! - **`MSBuild`** evaluates the build's definition set and the files it compiles, run as
//!   `dotnet msbuild` with `-getProperty` and `-getItem` ([`Evaluate_Build`]). It reads the
//!   project with its imports, `Directory.Build.props`, the SDK's configuration defaults and the
//!   target framework's implicit symbols, exactly as a build does. It restores nothing, builds
//!   nothing and writes nothing.
//! - **This crate** judges each file's directives against that set by the C# specification's
//!   rules ([`Read_Conditionals`]), recognizing directives only where the compiler does
//!   (`crate::lexer`).
//!
//! # What the host needs
//!
//! A .NET SDK recent enough for `-getProperty`, which `MSBuild` 17.8 added (the .NET 8 SDK
//! onward), and an SDK-style project, whose SDK supplies the `AddImplicitDefineConstants` target.
//! A host with no SDK is reported [`EvaluationFailure::DotnetUnavailable`], which a rule reports as
//! `DependencyUnavailable` -- never answered as though no symbol were defined, which would call
//! every `#if DEBUG` branch skipped. A project the SDK refuses, including an old-style one, is
//! [`EvaluationFailure::Refused`] with `MSBuild`'s own words.
//!
//! # What it does not do
//!
//! It decides no build. Which builds a repository cares about is the repository's to declare, and
//! the composition root reads that declaration and hands each build here; this crate evaluates
//! exactly the build it is asked about. It answers which files that build compiles, because
//! `MSBuild` does and a directory cannot, but it does not choose among them: a file is judged under
//! whatever build it is handed with, and a caller that hands it a file the build does not compile
//! gets an answer about a compilation that never happens. [`BuildEvaluation::Compiles`] is how a
//! caller avoids that.

#![forbid(unsafe_code)]

mod conditional_fact_production;
mod conditional_reading;
mod build_evaluation;
mod build_set;
mod conditional_walk;
mod definition_set;
mod evaluation_error;
mod evaluation_failure;
mod evaluation_request;
mod fact_context;
mod guarantee;
mod lexer;
mod materialization;
mod msbuild;
mod parse_failure;
mod preprocessor_expression;
mod production;

pub use build_evaluation::BuildEvaluation;
pub use build_set::{BuildPorts, BuildSetAnswer, FileToJudge, JudgedFile, Materialize_Build_Set, Unjudged};
pub use conditional_fact_production::ConditionalFactProduction;
pub use conditional_reading::{ConditionalReading, Read_Conditionals};
pub use definition_set::DefinitionSet;
pub use evaluation_error::EvaluationError;
pub use evaluation_failure::EvaluationFailure;
pub use evaluation_request::EvaluationRequest;
pub use fact_context::FactContext;
pub use guarantee::{Declared_Guarantee, PROVIDER, Provider_Offer};
pub use materialization::Materialization;
pub use msbuild::Evaluate_Build;
pub use parse_failure::ParseFailure;
pub use production::{Build_Subject, Materialize_Conditional_Fact};
