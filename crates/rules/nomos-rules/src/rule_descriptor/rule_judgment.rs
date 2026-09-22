//! The one shape every rule's own judgment is called through.
//!
//! Its own file rather than a second type beside [`super::RuleDescriptor`]: the one-public-type
//! rule wants one file-home type per file, and the callable is what keeps [`super::DESCRIPTORS`]
//! a `const` table rather than a list of identifiers needing a second table to carry the code.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

use super::DeclaredTextRule;

/// A rule's own judgment, in the one shape every rule can be called through.
///
/// A named type with one private field rather than a bare `fn` alias, and the reason has
/// outlived the alias twice over. The alias named the decision and stopped there: the two
/// parameters stayed positions a caller could transpose, and there was nowhere to state what
/// a rule's judgment may do. [`Self::Judges`] is that somewhere, and it is the one place a
/// run calls a rule through -- `nomos_check_orchestration`'s `run_context` names it, and
/// nothing else may.
///
/// # The two arms
///
/// A rule is linked or declared, and the field carries which. A linked rule is a `fn`
/// pointer, which is the only callable a `const` can hold and is what `OD-RULES-027` decided
/// makes the table below the whole declaration of a rule rather than half of one. A declared
/// rule is a `&'static` reference to a [`DeclaredTextRule`], which is data and so is
/// `const`-constructible for the same reason a pointer is.
///
/// Both arms being `const` is the whole of why there is no second list. `OD-RULES-034`
/// measured the alternative -- a separate table of declared rules beside this one -- against
/// what `OD-GATE-020` measured twice: two artifacts declaring one thing go silently out of
/// step. A second arm on one field cannot.
///
/// # Why nothing can tell the two apart
///
/// The arm is private, there is no accessor, and [`Self::Judges`] answers identically for
/// both, so no consumer of a [`super::RuleDescriptor`] has a way to ask which one it holds --
/// including `nomos_check_orchestration`'s `declared_rules`, which derives a `RulePackage`
/// from every descriptor and never reads this field at all. That is a conclusion rather than
/// an omission: `Judgment::Mechanical` means a linked implementation decides the rule, and a
/// declared form interpreted by a linked interpreter is precisely that.
///
/// [`core::fmt::Debug`] is written by hand for the same reason. A derived one would print the
/// arm, which would make the distinction observable through a formatter after every other
/// path to it had been closed -- and what it would print is a `fn` pointer's address, which
/// the compiler is free to merge across distinct functions and duplicate across codegen
/// units, so the derive was already answering a question it could not answer.
///
/// # What both arms are handed
///
/// Both parameters are taken by every rule and read by most. A rule that judges only source
/// text is widened here with a closure that ignores the reader, and the two rules whose
/// whole subject arrives through the reader are widened with one that ignores the sources;
/// neither wrapper decides anything, which is why they are spelled inline in the table
/// rather than given names of their own. A declared rule needs no wrapper: the interpreter
/// takes both and reads the reader only where the declaration says it must.
///
/// The sources a rule is handed are not always the walked ones. Six rules read a capability
/// family's own materialized slice instead, and which six is `nomos_check_orchestration`'s
/// to say, not this table's: a capability slice is an orchestration concept, and a
/// descriptor naming one would be a lower band describing an upper band's shape.
/// `OD-RULES-027` decided that split and why the mapping that remains there is a
/// declaration rather than the demand planner `OD-RULES-009` declines.
#[derive(Clone, Copy)]
pub struct RuleJudgment(Arm);

/// Which of the two shapes a judgment is carried in.
///
/// Private, with no accessor and no `Debug`, which is what makes the choice invisible to
/// every consumer rather than merely undocumented.
#[derive(Clone, Copy)]
enum Arm
{
    /// A rule written as a Rust function and compiled into this crate.
    Linked(fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>),
    /// A rule of the archetype `OD-RULES-034` measured, declared and interpreted here.
    Declared(&'static DeclaredTextRule),
}

impl RuleJudgment
{
    /// A linked rule's judgment, carried as the one pointer shape a `const` table can hold.
    ///
    /// `const` so that [`super::DESCRIPTORS`] stays a table rather than becoming a run-time
    /// build step: a constructor that allocated would be this same declaration with the
    /// property that made it useful taken away -- `OD-RULES-027`.
    #[must_use]
    pub const fn New(judge: fn(&[SourceFile], &mut dyn FactReader) -> Vec<Finding>) -> Self
    {
        return Self(Arm::Linked(judge));
    }

    /// A declared rule's judgment: the declaration itself, interpreted when it is run.
    ///
    /// `const` for the identical reason [`Self::New`] is, and `pub(crate)` because a
    /// declaration is authored in this crate beside the vocabulary it names. Nothing outside
    /// composes a descriptor, so exporting this would offer a second authoring path to
    /// callers that have no vocabulary to name -- the accretion the one table exists to
    /// prevent.
    pub(crate) const fn Declaring(declaration: &'static DeclaredTextRule) -> Self
    {
        return Self(Arm::Declared(declaration));
    }

    /// Runs this rule's judgment over `sources`, reading whatever facts it needs through `reader`.
    ///
    /// The rule's only call path, for either arm. Both arguments are handed to every rule
    /// whether it reads them or not, which is what lets one signature serve a fact-free rule,
    /// a source-free rule and a rule that reads both -- see the module doc for why that
    /// widening has no decision in it.
    ///
    /// The signature is unchanged by the second arm, and that is load-bearing rather than
    /// incidental: this is called from exactly one place in the whole workspace,
    /// `nomos_check_orchestration`'s `run_context/judging.rs`, so a declared rule joins a run
    /// with no change anywhere above this crate. `OD-RULES-034` says an item that needed one
    /// would be evidence the seam was chosen wrongly.
    #[must_use]
    pub fn Judges(&self, sources: &[SourceFile], reader: &mut dyn FactReader) -> Vec<Finding>
    {
        return match self.0
        {
            Arm::Linked(judge) => judge(sources, reader),
            Arm::Declared(declaration) => declaration.Judges(sources, reader),
        };
    }
}

impl core::fmt::Debug for RuleJudgment
{
    /// Prints the type and nothing else, for either arm.
    ///
    /// Hand-written rather than derived because a derive would print the arm, and the arm is
    /// the one thing about a rule's judgment that no consumer may branch on -- see the type's
    /// own doc. What is withheld is a `fn` pointer's address or a declaration's fields,
    /// neither of which a reader of a descriptor's `Debug` output can act on.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str("RuleJudgment");
    }
}
