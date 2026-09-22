//! Rust source-text rules from code-standards.
//!
//! These checks deliberately stay conservative and text-local. They import standards whose
//! deciding evidence is visible in one Rust source file: panic primitive spelling, path
//! attributes, shared `Rc`/`Arc` plus `RefCell` ownership escapes, and four rules built on
//! the same "an attribute or construct carries no adjacent explanatory comment" shape —
//! `#[allow(...)]`, `unsafe` constructs, `#[inline(always)]`, and a bare `#[ignore]` with no
//! `= "reason"` value. Three of the four — `#[allow(...)]`, `#[inline(always)]` and the bare
//! `#[ignore]` — reuse `comment_block`'s `Has_A_Previous_Comment_Block`, the same "walk the
//! contiguous comment block immediately above this line" primitive `panic`'s
//! `Panic_Findings_In` and `interior_mutability`'s `Shared_Interior_Mutability_Findings_In`
//! already share — real repeat consumers, not a new abstraction invented for them. `unsafe`
//! is the fourth, and `OD-RULES-021` decided its own two constructs — a bare `unsafe {}`
//! block and an `unsafe fn`/`trait`/`impl` declaration — need two different artifacts rather
//! than one matcher applied uniformly: see `unsafe_justification`'s
//! `Has_Local_Safety_Justification` for why it does not reuse the shared primitive either
//! way.
//!
//! # Split by responsibility
//!
//! Eight rules in one file passed the size at which this crate extracts a submodule, and the
//! split follows the reasoning the family already carries. `comment_block` holds every
//! primitive that walks the contiguous comment block above a line; `unwrap_expect`, `panic`,
//! `path_attribute`, `interior_mutability` and `unsafe_justification` each hold one rule's
//! own readings, and `comment_justified` and `disabled_test` hold the readings of the three
//! rules that are now declared rather than written.
//!
//! What more than one of those needs stayed here: the eight rule ids, the self-exemption
//! below, the finding constructor every rule reports through, and `Detector` — the one shape
//! a rule with no shared primitive of its own is written over.
//!
//! # The three declared rules
//!
//! `every-allow-carries-a-justification`, `inline-always-requires-justification` and
//! `a-disabled-test-states-why` are no longer functions. `OD-RULES-034` censused this
//! workspace's seventy-one composed rules and decided a declarative form for the archetype
//! forty-four of them share — a per-line predicate over one file's raw text — so those three
//! are `declarations`' `DeclaredTextRule` literals, interpreted by
//! [`Judged_By_Declaration`] below. The interpreter is this module's own gate, traversal and
//! finding constructor with the parts a declaration states lifted out of it, which is why a
//! declared rule reports byte-identically to the function it replaced: nothing about how a
//! finding is built moved.
//!
//! The three had accumulated three separately-written copies of one identical justification
//! predicate between them, which is the authoring redundancy the record measured and the
//! reason those three went first.

use super::code_prefix::{Code_Prefix, Code_With_String_Bodies_Masked};
use crate::rule_descriptor::DeclaredTextRule;
use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

mod comment_block;
mod comment_justified;
mod declarations;
mod disabled_test;
mod interior_mutability;
mod panic;
mod path_attribute;
mod unsafe_justification;
mod unwrap_expect;

/// The three declarations this family's own `declarations` module holds, named where the
/// table can reach them. `DESCRIPTORS` is the only caller.
pub(crate) use declarations::{A_DISABLED_TEST_DECLARATION, EVERY_ALLOW_DECLARATION, INLINE_ALWAYS_DECLARATION};

/// The readings a declaration may name, hoisted to where `DeclaredDetector` and
/// `DeclaredJustification` resolve them. A vocabulary member stands for exactly one of
/// these, and nothing else in this crate may name them.
pub(crate) use comment_block::Has_An_Adjacent_Non_Empty_Comment;
pub(crate) use comment_justified::{Has_Allow_Attribute, Has_Inline_Always_Attribute};
pub(crate) use disabled_test::Has_Bare_Ignore_Attribute;

pub use comment_justified::{Check_Every_Allow_Carries_A_Justification, Check_Inline_Always_Justification};
pub use disabled_test::Check_A_Disabled_Test_States_Why;
pub use interior_mutability::Check_Shared_Interior_Mutability_Says_Why;
pub use panic::Check_Panics_Are_Justified_Documented_And_Validated;
pub use path_attribute::Check_A_Rust_Path_Stays_Within_Its_Own_Subtree;
pub use unsafe_justification::Check_Unsafe_Justification;
pub use unwrap_expect::Check_Unwrap_Expect_Discipline;

/// The code-standards `unwrap`/`expect` discipline rule id.
pub const UNWRAP_EXPECT_DISCIPLINE: &str = "unwrap-expect-discipline";
/// The code-standards explicit-panic justification rule id.
pub const PANICS_ARE_JUSTIFIED_DOCUMENTED_AND_VALIDATED: &str = "panics-are-justified-documented-and-validated";
/// The code-standards Rust path-attribute rule id.
pub const A_RUST_PATH_STAYS_WITHIN_ITS_OWN_SUBTREE: &str = "a-rust-path-stays-within-its-own-subtree";
/// The code-standards shared-interior-mutability rule id.
pub const SHARED_INTERIOR_MUTABILITY_SAYS_WHY: &str = "shared-interior-mutability-says-why";
/// The code-standards `#[allow(...)]` justification rule id.
pub const EVERY_ALLOW_CARRIES_A_JUSTIFICATION: &str = "every-allow-carries-a-justification";
/// The code-standards `unsafe` justification rule id.
pub const UNSAFE_JUSTIFICATION: &str = "unsafe-justification";
/// The code-standards `#[inline(always)]` justification rule id.
pub const INLINE_ALWAYS_JUSTIFICATION: &str = "inline-always-requires-justification";
/// The code-standards disabled-test justification rule id.
pub const A_DISABLED_TEST_STATES_WHY: &str = "a-disabled-test-states-why";

/// `rule` and `message` are both `&str`; without a distinct type per position, a call site
/// like `Unjustified_Construct_Findings_In(source, rule, message, ...)` reads as two
/// interchangeable strings and a swap compiles silently.
struct Rule<'a>(&'a str);
struct Message<'a>(&'a str);

/// Recognizes the construct this rule judges, named so it reads as a collaborator with one
/// documented operation rather than a bare stored callable.
struct ConstructDetector(fn(&str) -> bool);

impl ConstructDetector
{
    fn Is_Detecting(&self, code: &str) -> bool
    {
        return (self.0)(code);
    }
}

/// Recognizes a construct's local justification, named for the same reason as
/// [`ConstructDetector`].
struct JustificationDetector(fn(&[&str], usize) -> bool);

impl JustificationDetector
{
    fn Is_Detecting(&self, lines: &[&str], index: usize) -> bool
    {
        return (self.0)(lines, index);
    }
}

/// How to recognize the construct this rule judges and how to recognize its local
/// justification, grouped so the six call sites above and this function stay under the
/// parameter-count ceiling.
struct Detector
{
    has_construct: ConstructDetector,
    has_local_justification: JustificationDetector,
}

/// The shape `panic`'s `Panic_Findings_In`, `interior_mutability`'s
/// `Shared_Interior_Mutability_Findings_In`, `unsafe_justification`'s `Unsafe_Findings_In`,
/// `disabled_test`'s `Disabled_Test_Findings_In`, and `comment_justified`'s
/// `Allow_Findings_In` and `Inline_Always_Findings_In` all reduce to: a construct-matching
/// predicate, an unless-locally-justified predicate, and one finding message. This is the one
/// place that shape is written down.
fn Unjustified_Construct_Findings_In(source: &SourceFile, rule: Rule<'_>, message: Message<'_>, detector: Detector) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        // `P69-SELF-MATCH-VIA-STRING-LITERALS-FIVE-MORE-RULES`: every detector here looks
        // for real attribute/keyword syntax, which is never legitimately written inside a
        // string literal -- masking a literal's own body before the search can only drop a
        // false positive (prose quoting the construct it describes), never hide a real one.
        let code = Code_With_String_Bodies_Masked(&Code_Prefix(line));
        if detector.has_construct.Is_Detecting(&code) && !detector.has_local_justification.Is_Detecting(&lines, index)
        {
            let finding = Finding_For_Line(source, rule.0, Line_Number(index), message.0);
            findings.push(finding);
        }
    }

    return findings;
}

/// Runs one declared rule over `sources`, and the whole of what a declaration means.
///
/// `OD-RULES-034`'s interpreter. Every line of it was already in this module: the gate is
/// the one the six linked rules above wrote out per rule, the traversal and the finding
/// construction are [`Unjustified_Construct_Findings_In`]'s, and the ordering is the
/// `subject_name` sort every rule in this family ends with. What a declaration supplies is
/// the four things those bodies differed in — the detector, the justification, the message
/// and the identity — which is why a declared rule reports byte-identically to the function
/// it replaced rather than merely equivalently.
///
/// The reader is read only where the declaration says test material is excluded. That is not
/// an optimization: `FactReader::Require` records an unmet read against the run's own claim,
/// so a declaration that is not test-material sensitive must not ask, exactly as the linked
/// rules it replaces never asked.
pub(crate) fn Judged_By_Declaration(declaration: &DeclaredTextRule, sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let declared_fixture_locations = Declared_Fixture_Locations_For(declaration, facts);
    let mut findings = Vec::new();

    for source in sources
    {
        if Is_Judged_By(declaration, source, &declared_fixture_locations)
        {
            findings.extend(Declared_Findings_In(declaration, source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// The repository's own declared fixture locations, read once for the whole run and only
/// where the declaration's own criterion needs them.
fn Declared_Fixture_Locations_For(declaration: &DeclaredTextRule, facts: &mut dyn FactReader) -> Vec<String>
{
    if !declaration.Is_Excluding_Test_Material()
    {
        return Vec::new();
    }

    return super::Resolve_Declared_Fixture_Locations(facts);
}

/// Whether this declaration's subject includes `source`: its language, its test-material
/// criterion and its self-exemption, in the order the rules it replaces asked them in.
fn Is_Judged_By(declaration: &DeclaredTextRule, source: &SourceFile, declared_fixture_locations: &[String]) -> bool
{
    if declaration.language.is_some_and(|language| return !source.Is_Written_In(language))
    {
        return false;
    }

    if declaration.Is_Excluding_Test_Material() && super::Is_Test_Or_Example_Source(source, declared_fixture_locations)
    {
        return false;
    }

    return !Is_Exempt_Implementation_File(source, declaration.self_exemption);
}

/// One judged source's findings, through the same engine the linked rules of this family go
/// through — the declaration resolved to the two predicates that engine takes.
fn Declared_Findings_In(declaration: &DeclaredTextRule, source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(declaration.id),
        Message(declaration.message),
        Detector {
            has_construct: ConstructDetector(declaration.detector.As_Predicate()),
            has_local_justification: JustificationDetector(Justification_Predicate(declaration)),
        },
    );
}

/// The justification a declaration names, or one that never holds where it names none.
///
/// A declaration with no justification clause is a rule with no local escape, which is a
/// real shape rather than a missing field — so it renders as a predicate that answers `false`
/// rather than as an empty one the engine would have to special-case.
fn Justification_Predicate(declaration: &DeclaredTextRule) -> fn(&[&str], usize) -> bool
{
    return match declaration.justification
    {
        Some(justification) => justification.As_Predicate(),
        None => Never_Locally_Justified,
    };
}

fn Never_Locally_Justified(_lines: &[&str], _index: usize) -> bool
{
    return false;
}

/// Whether `source` is one of the implementation modules a declaration exempts.
fn Is_Exempt_Implementation_File(source: &SourceFile, modules: &[&str]) -> bool
{
    return modules.iter().any(|module| return Is_Under_Module(source, module));
}

/// This rule family's own module root, checked with the same normalized-slash comparison
/// [`super::Is_Test_Or_Example_Source`] already uses. Every rule under it that reads a
/// construct's own spelling (`unsafe {`, `#[allow(`, `Rc::new(RefCell::new(`, `#[path`)
/// exempts this module: its own test fixtures and each rule's own detection-pattern string
/// necessarily spell out the exact syntax the rule looks for, so these are the files in the
/// workspace guaranteed to look like violations of every rule they implement, regardless of
/// whether the match is a fixture or the pattern-matching code itself.
///
/// The exemption names the module rather than the single file it originally named because
/// the split above moved those fixtures and patterns into the children — a child holding one
/// rule's detection patterns is exactly as self-matching as the file they were written in.
/// It stays a real boundary rather than a suffix: a stranger's repository can end the same
/// way without earning the exemption.
///
/// `#![forbid(unsafe_code)]` at the crate root makes the `unsafe-justification` instance of
/// this provably safe forever; the others are safe today (checked: no real `#[allow(...)]`/
/// `Rc<RefCell<...>>`/escaping `#[path]` usage exists under this module outside its own
/// fixtures and patterns), and trade a theoretical future in-module violation going unflagged
/// for not building the per-line, string-literal-aware self-reference tracking this crate has
/// so far declined to build — the same tradeoff every other path-based exemption here already
/// makes.
const OWN_IMPLEMENTATION_MODULE: &str = "crates/rules/nomos-rules/src/checks/rust_text";

fn Lines_Of(source: &SourceFile) -> Vec<&str>
{
    return source.text.lines().collect();
}

fn Finding_For_Line(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number} {because}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Is_Own_Implementation_File(source: &SourceFile) -> bool
{
    return Is_Under_Module(source, OWN_IMPLEMENTATION_MODULE);
}

/// Whether `source` is `module`'s own file or sits under its directory, with slashes
/// normalized first.
///
/// One comparison for the linked rules' fixed exemption and for a declaration's stated one,
/// because they are the same criterion: a second copy is how a declared rule would start
/// exempting a file its linked predecessor judged, which the byte-identical comparison in
/// `declarations`' own tests exists to catch and this exists to make impossible.
fn Is_Under_Module(source: &SourceFile, module: &str) -> bool
{
    let normalized = source.path.replace('\\', "/");
    let Some(rest) = normalized.strip_prefix(module)
    else
    {
        return false;
    };

    return rest == ".rs" || rest.starts_with('/');
}

#[cfg(test)]
mod tests;
