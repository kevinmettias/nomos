//! Every value a rule reads from a row-shaped policy family, declared once.
//!
//! `OD-RULES-035` decided that a preference axis is a key, declared in this crate beside the
//! rules that read it and read through the one resolver its family has. Before this file the
//! same key and its default were written wherever a rule read it: `checks::structure` and
//! `checks::function_shape` each carried a private `Resolve_Limit`, every call site spelled its
//! key as a literal, and [`super::DeclaredParameter`] paired a key with a default a third time.
//! Now a rule names an axis, and the axis's family, key and undeclared meaning are written here
//! and nowhere else.
//!
//! # Only the row-shaped families
//!
//! The naming and limits contracts carry rows of scope, key and value, and neither interprets
//! its keys, so what a key means has to live with the rules that read it. A struct-shaped family
//! -- scripting, words, goals, test material -- already names its axes as the fields of its
//! payload type, and needs nothing here.
//!
//! # A case read through more than one axis
//!
//! A function's case is read under several keys -- the refinement its visibility selects, the
//! key that refines, and for a Go method the method keys ahead of both -- and which axis's
//! undeclared meaning applies when none is declared is part of the read, not of any one axis.
//! [`CaseRead`] carries both, and every such read the naming rules make is declared below, so
//! `OD-RULES-035` decisions 7 and 8 are stated here once rather than at each call site.

mod undeclared;

pub(crate) use undeclared::Undeclared;

use super::RequiredFact;
use nomos_cap_naming_policy::Case;

/// One value a rule reads from a row-shaped policy family.
///
/// The kind of value it takes is `Value` -- a count for limits, a [`Case`] for naming -- which is
/// what keeps a limits axis from being handed to the naming resolver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PolicyAxis<Value: 'static>
{
    /// The family a repository declares this axis in, and the capability its resolver reads.
    pub(crate) family: RequiredFact,
    /// The key a repository declares this axis's value under.
    pub(crate) key: &'static str,
    /// What a rule reading this axis does when the repository declares nothing for it.
    pub(crate) undeclared: Undeclared<Value>,
}

impl<Value: Copy> PolicyAxis<Value>
{
    /// The value a rule reading this axis for `language` is judged against when the repository
    /// declares nothing: that language's own default where the axis states one, and the
    /// repository-wide default otherwise -- or `None` when the axis is reported as undeclared,
    /// because then there is no value to judge against.
    pub(crate) fn Undeclared_Value(&self, language: Option<&str>) -> Option<Value>
    {
        return match self.undeclared
        {
            Undeclared::JudgedAgainstDefault { repository, languages } => Some(
                language
                    .and_then(|language| return languages.iter().find(|(named, _)| return *named == language))
                    .map_or(repository, |(_, value)| return *value),
            ),
            Undeclared::ReportedAsUndeclared => None,
        };
    }
}

/// The line count past which a file is a review candidate for splitting. code-standards' 500,
/// which Go shares, so Go states no default of its own.
pub(crate) const FILE_SIZE_REVIEW_LINES: PolicyAxis<u32> = PolicyAxis {
    family: RequiredFact::LimitsPolicy,
    key: "file-size-review-lines",
    undeclared: Undeclared::JudgedAgainstDefault { repository: 500, languages: &[] },
};

/// The line count past which a file must carry an explicit splitting justification.
/// code-standards' 1500, and Go's own lower 1000.
pub(crate) const FILE_SIZE_HARD_LINES: PolicyAxis<u32> = PolicyAxis {
    family: RequiredFact::LimitsPolicy,
    key: "file-size-hard-lines",
    undeclared: Undeclared::JudgedAgainstDefault { repository: 1500, languages: &[("go", 1000)] },
};

/// The value-parameter cap a function may declare. code-standards' four, which Go shares.
pub(crate) const PARAMETER_COUNT_MAX: PolicyAxis<u32> = PolicyAxis {
    family: RequiredFact::LimitsPolicy,
    key: "parameter-count-max",
    undeclared: Undeclared::JudgedAgainstDefault { repository: 4, languages: &[] },
};

/// The deepest control-flow nesting a function may reach. code-standards' three.
pub(crate) const NESTING_DEPTH_MAX: PolicyAxis<u32> = PolicyAxis {
    family: RequiredFact::LimitsPolicy,
    key: "nesting-depth-max",
    undeclared: Undeclared::JudgedAgainstDefault { repository: 3, languages: &[] },
};

/// The largest cyclomatic complexity a function may reach and not pass.
///
/// Reported as undeclared, the first axis to take that meaning. The other four limits are
/// code-standards' own numbers, carried as defaults because a repository running both tools is
/// already held to them. Nothing holds this one: code-standards judges no complexity, and the
/// figures in circulation -- ten, fifteen, twenty -- are conventions of particular tools rather
/// than a number this workspace could say a repository chose. `OD-RULES-035` section 3.
pub(crate) const CYCLOMATIC_COMPLEXITY_MAX: PolicyAxis<u32> = PolicyAxis {
    family: RequiredFact::LimitsPolicy,
    key: "cyclomatic-complexity-max",
    undeclared: Undeclared::ReportedAsUndeclared,
};

/// The case a function name takes.
///
/// Read behind the refinement a function's visibility selects, by every rule that judges a
/// function's name, and for a Go method behind the method keys as well. `function-naming-convention`
/// takes this axis's undeclared meaning; the two Go function rules take their refinement's.
pub(crate) const FUNCTION_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "function",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::UpperSnake, languages: &[] },
};

/// The case a module name takes.
pub(crate) const MODULE_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "module",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::LowerSnake, languages: &[] },
};

/// The case a field name takes.
pub(crate) const FIELD_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "field",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::LowerSnake, languages: &[] },
};

/// The case an exported function name takes.
///
/// Read ahead of [`FUNCTION_CASE`] for an exported function in any language. Its undeclared
/// meaning below is Go's, and only the exported Go function rule takes it.
pub(crate) const EXPORTED_FUNCTION_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "function.exported",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::UpperSnake, languages: &[] },
};

/// The case an unexported function name takes.
///
/// Read ahead of [`FUNCTION_CASE`] for every other function in any language. Its undeclared
/// meaning below is Go's mixed-snake, and only the unexported Go function rule takes it.
pub(crate) const UNEXPORTED_FUNCTION_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "function.unexported",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::MixedSnake, languages: &[] },
};

/// The case a method name takes.
///
/// Read for a Go method alone, after its refinement and ahead of every function key. Go nests a
/// function under nothing but the receiver it is declared on, so the syntax fact says which Go
/// functions are methods; for Rust it cannot, because it does not say which parameter is a
/// receiver, and `OD-RULES-035` decision 8 reads no method key there. No read takes the undeclared
/// meaning below: a method whose repository declares no method key is judged as a function is, so
/// the value is [`FUNCTION_CASE`]'s.
pub(crate) const METHOD_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "method",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::UpperSnake, languages: &[] },
};

/// The case an exported method name takes. Read for Go, as [`METHOD_CASE`] says; its undeclared
/// meaning is never taken, and is [`EXPORTED_FUNCTION_CASE`]'s.
pub(crate) const EXPORTED_METHOD_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "method.exported",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::UpperSnake, languages: &[] },
};

/// The case an unexported method name takes. Read for Go, as [`METHOD_CASE`] says; its undeclared
/// meaning is never taken, and is [`UNEXPORTED_FUNCTION_CASE`]'s.
pub(crate) const UNEXPORTED_METHOD_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "method.unexported",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::MixedSnake, languages: &[] },
};

/// One case a rule reads through more than one axis.
///
/// `OD-RULES-035` decisions 7 and 8. The first of `keys` the repository declares decides, each
/// looked up for the language before repository-wide, and `keys` name a refinement before the key
/// it refines and a method's keys before a function's: code-standards' own order for the same block
/// of the same file, so a declaration there means one thing to both tools that read it. With none
/// declared, `undeclared`'s undeclared meaning applies. That is the axis the rule read alone before
/// it read the others, so a repository that declares none of them is judged as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CaseRead<'axes>
{
    /// The axes the case is looked up under, most specific first.
    pub(crate) keys: &'axes [&'axes PolicyAxis<Case>],
    /// The axis whose undeclared meaning applies when the repository declares none of `keys`.
    pub(crate) undeclared: &'axes PolicyAxis<Case>,
}

/// The keys an exported function's case is read under, the refinement first.
const EXPORTED_FUNCTION_KEYS: &[&PolicyAxis<Case>] = &[&EXPORTED_FUNCTION_CASE, &FUNCTION_CASE];

/// The keys every other function's case is read under, the refinement first.
const UNEXPORTED_FUNCTION_KEYS: &[&PolicyAxis<Case>] = &[&UNEXPORTED_FUNCTION_CASE, &FUNCTION_CASE];

/// The keys an exported Go method's case is read under: its own, then an exported function's.
const EXPORTED_METHOD_KEYS: &[&PolicyAxis<Case>] = &[&EXPORTED_METHOD_CASE, &METHOD_CASE, &EXPORTED_FUNCTION_CASE, &FUNCTION_CASE];

/// The keys every other Go method's case is read under: its own, then an unexported function's.
const UNEXPORTED_METHOD_KEYS: &[&PolicyAxis<Case>] = &[&UNEXPORTED_METHOD_CASE, &METHOD_CASE, &UNEXPORTED_FUNCTION_CASE, &FUNCTION_CASE];

/// How `function-naming-convention` reads the case of a function declared exported -- in Rust,
/// exactly `pub` -- in whatever language it is written in, and upper-snake when nothing is declared.
pub(crate) const EXPORTED_FUNCTION_READ: CaseRead<'static> = CaseRead { keys: EXPORTED_FUNCTION_KEYS, undeclared: &FUNCTION_CASE };

/// How `function-naming-convention` reads every other function's case -- in Rust private,
/// `pub(crate)`, `pub(super)`, `pub(in path)`, or a trait declaration's own member.
pub(crate) const UNEXPORTED_FUNCTION_READ: CaseRead<'static> = CaseRead { keys: UNEXPORTED_FUNCTION_KEYS, undeclared: &FUNCTION_CASE };

/// How `function-naming-convention` reads an exported Go method's case.
pub(crate) const EXPORTED_METHOD_READ: CaseRead<'static> = CaseRead { keys: EXPORTED_METHOD_KEYS, undeclared: &FUNCTION_CASE };

/// How `function-naming-convention` reads every other Go method's case.
pub(crate) const UNEXPORTED_METHOD_READ: CaseRead<'static> = CaseRead { keys: UNEXPORTED_METHOD_KEYS, undeclared: &FUNCTION_CASE };

/// How the exported Go function rule reads an exported Go function's case, and upper-snake when
/// nothing is declared.
pub(crate) const GO_EXPORTED_FUNCTION_READ: CaseRead<'static> = CaseRead { keys: EXPORTED_FUNCTION_KEYS, undeclared: &EXPORTED_FUNCTION_CASE };

/// How the unexported Go function rule reads an unexported Go function's case, and Go's
/// mixed-snake when nothing is declared.
pub(crate) const GO_UNEXPORTED_FUNCTION_READ: CaseRead<'static> = CaseRead { keys: UNEXPORTED_FUNCTION_KEYS, undeclared: &UNEXPORTED_FUNCTION_CASE };

/// How the exported Go function rule reads an exported Go method's case.
pub(crate) const GO_EXPORTED_METHOD_READ: CaseRead<'static> = CaseRead { keys: EXPORTED_METHOD_KEYS, undeclared: &EXPORTED_FUNCTION_CASE };

/// How the unexported Go function rule reads an unexported Go method's case.
pub(crate) const GO_UNEXPORTED_METHOD_READ: CaseRead<'static> = CaseRead { keys: UNEXPORTED_METHOD_KEYS, undeclared: &UNEXPORTED_FUNCTION_CASE };

/// The case an exported type name takes. Read for Go.
pub(crate) const EXPORTED_TYPE_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "type.exported",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::UpperCamel, languages: &[] },
};

/// The case an unexported type name takes. Read for Go.
pub(crate) const UNEXPORTED_TYPE_CASE: PolicyAxis<Case> = PolicyAxis {
    family: RequiredFact::NamingPolicy,
    key: "type.unexported",
    undeclared: Undeclared::JudgedAgainstDefault { repository: Case::LowerCamel, languages: &[] },
};

/// Every limits axis above, which is every key a repository's `nomos-limits.json` may declare
/// and have a rule read.
///
/// `checks::undeclared_policy_key` reports any other key that file declares, so an axis added
/// above and left out of this list is reported as undeclared the first time a repository
/// writes it -- loud rather than silent, which is the direction `OD-RULES-035` decided a
/// mistake here should fail in. Naming has no list: its keys are read from `standards.json`, a
/// file another tool owns, and a key this workspace does not read there may be one that tool
/// does.
pub(crate) const LIMITS_AXES: &[&PolicyAxis<u32>] =
    &[&FILE_SIZE_REVIEW_LINES, &FILE_SIZE_HARD_LINES, &PARAMETER_COUNT_MAX, &NESTING_DEPTH_MAX, &CYCLOMATIC_COMPLEXITY_MAX];

#[cfg(test)]
mod tests
{
    use super::*;

    const LIMITS: &[&PolicyAxis<u32>] = LIMITS_AXES;

    const NAMING: [PolicyAxis<Case>; 10] = [
        FUNCTION_CASE,
        MODULE_CASE,
        FIELD_CASE,
        EXPORTED_FUNCTION_CASE,
        UNEXPORTED_FUNCTION_CASE,
        METHOD_CASE,
        EXPORTED_METHOD_CASE,
        UNEXPORTED_METHOD_CASE,
        EXPORTED_TYPE_CASE,
        UNEXPORTED_TYPE_CASE,
    ];

    /// One read a naming rule makes: the side of visibility it reads, whether it reads a method,
    /// and the key whose undeclared meaning it takes.
    struct ReadUnderTest
    {
        read: CaseRead<'static>,
        visibility: &'static str,
        method: bool,
        undeclared: &'static str,
    }

    /// Every read `function-naming-convention` and the two Go function rules make.
    const READS: [ReadUnderTest; 8] = [
        ReadUnderTest { read: EXPORTED_FUNCTION_READ, visibility: "exported", method: false, undeclared: "function" },
        ReadUnderTest { read: UNEXPORTED_FUNCTION_READ, visibility: "unexported", method: false, undeclared: "function" },
        ReadUnderTest { read: EXPORTED_METHOD_READ, visibility: "exported", method: true, undeclared: "function" },
        ReadUnderTest { read: UNEXPORTED_METHOD_READ, visibility: "unexported", method: true, undeclared: "function" },
        ReadUnderTest { read: GO_EXPORTED_FUNCTION_READ, visibility: "exported", method: false, undeclared: "function.exported" },
        ReadUnderTest { read: GO_UNEXPORTED_FUNCTION_READ, visibility: "unexported", method: false, undeclared: "function.unexported" },
        ReadUnderTest { read: GO_EXPORTED_METHOD_READ, visibility: "exported", method: true, undeclared: "function.exported" },
        ReadUnderTest { read: GO_UNEXPORTED_METHOD_READ, visibility: "unexported", method: true, undeclared: "function.unexported" },
    ];

    /// The resolver reads the capability an axis's `family` names, so an axis declared in the
    /// wrong family would be read from the wrong fact and fall back to its default in silence.
    #[test]
    fn Test_Every_Axis_Should_Be_Declared_In_The_Family_Its_Value_Kind_Is_Read_From()
    {
        assert!(LIMITS.iter().all(|axis| return axis.family == RequiredFact::LimitsPolicy), "{LIMITS:?}");
        assert!(NAMING.iter().all(|axis| return axis.family == RequiredFact::NamingPolicy), "{NAMING:?}");
    }

    /// Two axes under one key in one family would be two answers to one declaration.
    #[test]
    fn Test_No_Two_Axes_In_A_Family_Should_Share_A_Key()
    {
        let mut limits: Vec<&str> = LIMITS.iter().map(|axis| return axis.key).collect();
        let mut naming: Vec<&str> = NAMING.iter().map(|axis| return axis.key).collect();
        limits.sort_unstable();
        limits.dedup();
        naming.sort_unstable();
        naming.dedup();

        assert_eq!(limits.len(), LIMITS.len(), "{limits:?}");
        assert_eq!(naming.len(), NAMING.len(), "{naming:?}");
    }

    /// A language's own default applies to that language only: Go's hard trigger is 1000, and a
    /// rule reading the axis repository-wide, or for a language the axis lists nothing for, gets
    /// the repository-wide 1500.
    #[test]
    fn Test_Undeclared_Value_Should_Take_A_Languages_Own_Default_For_That_Language_Only()
    {
        assert_eq!(FILE_SIZE_HARD_LINES.Undeclared_Value(Some("go")), Some(1000));
        assert_eq!(FILE_SIZE_HARD_LINES.Undeclared_Value(None), Some(1500));
        assert_eq!(FILE_SIZE_HARD_LINES.Undeclared_Value(Some("rust")), Some(1500));
    }

    /// An axis reported as undeclared has no value to judge against, for any language.
    #[test]
    fn Test_Undeclared_Value_Should_Be_None_For_An_Axis_Reported_As_Undeclared()
    {
        assert_eq!(CYCLOMATIC_COMPLEXITY_MAX.Undeclared_Value(None), None);
        assert_eq!(CYCLOMATIC_COMPLEXITY_MAX.Undeclared_Value(Some("rust")), None);
    }

    /// A read names a refinement before the key it refines, and a method's keys before a
    /// function's, in one family, and the refinement is that key refined by the read's own side of
    /// visibility: `OD-RULES-035` decisions 7 and 8 read them in that order, which is
    /// code-standards' own, so a read listed any other way would let a plain key decide what a
    /// refinement declares differently.
    #[test]
    fn Test_A_Read_Should_Name_A_Refinement_Before_Its_Key_And_A_Methods_Keys_Before_A_Functions()
    {
        for tested in &READS
        {
            let keys: Vec<&str> = tested.read.keys.iter().map(|axis| return axis.key).collect();
            let function = [format!("{}.{}", FUNCTION_CASE.key, tested.visibility), FUNCTION_CASE.key.to_owned()];
            let method = [format!("{}.{}", METHOD_CASE.key, tested.visibility), METHOD_CASE.key.to_owned()];
            let expected: Vec<&str> =
                if tested.method { method.iter().chain(&function).map(String::as_str).collect() } else { function.iter().map(String::as_str).collect() };

            assert_eq!(keys, expected, "{:?}", tested.read);
            assert!(tested.read.keys.iter().all(|axis| return axis.family == FUNCTION_CASE.family), "{:?}", tested.read);
        }
    }

    /// A read with nothing declared is judged as its rule judged before it read more than one key:
    /// `function-naming-convention` against `function`'s upper-snake, and each Go function rule
    /// against its own refinement's Go default. Never a method axis's, because a method whose
    /// repository declares no method key is judged as a function is; and always one of the read's
    /// own keys, so the value the report names as undeclared is one the rule really read.
    #[test]
    fn Test_A_Read_Should_Take_The_Undeclared_Meaning_Of_The_Function_Axis_Its_Rule_Read_Before()
    {
        for tested in &READS
        {
            assert_eq!(tested.read.undeclared.key, tested.undeclared, "{:?}", tested.read);
            assert!(tested.read.keys.contains(&tested.read.undeclared), "{:?}", tested.read);
        }
    }

    /// The limits resolver reports an undeclared axis for whichever rule reads it; the naming
    /// resolver has no such path, because no naming axis has needed one. A naming axis declared
    /// reported-as-undeclared would reach a resolver with nothing to judge by, so this holds every
    /// naming axis to a default until the axis that needs otherwise brings the report path with it.
    #[test]
    fn Test_No_Naming_Axis_Should_Be_Reported_As_Undeclared_While_Its_Resolver_Cannot_Report()
    {
        let reported: Vec<&str> =
            NAMING.iter().filter(|axis| return axis.undeclared == Undeclared::ReportedAsUndeclared).map(|axis| return axis.key).collect();

        assert!(reported.is_empty(), "{reported:?}");
    }
}
