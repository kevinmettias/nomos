//! [`Demanded_Families`]' own contract, exercised: a selection demands exactly what its
//! rules declare, and nothing else.

use super::{Demanded_Families, Materialize_Compiler_Section, COPY_CLONES_FAMILY};
use crate::composed_providers::SubjectFact;
use crate::run_context::MaterializationEnvironment;
use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, RuleId, SnapshotId};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::{RequiredFact, DESCRIPTORS};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A rule's declared family is demanded when that rule is selected.
///
/// `NESTING_DEPTH` is the case, not an example. Its descriptor declares
/// `RequiredFact::LimitsPolicy` and the hand-written guard this derivation replaced named
/// five rules that did not include it, so selecting it alone materialized no limits-policy
/// fact and `Check_Nesting_Depth` fell back to `MAX_NESTING_DEPTH`'s built-in default
/// instead of the limit the repository configured -- silently, with no finding and no
/// `MissingCapability`. A full run hid it, because its five siblings were selected too.
#[test]
fn Test_A_Rule_Selected_Alone_Should_Demand_The_Family_It_Declares()
{
    let demanded = Demanded_Families(&[RuleId::New(nomos_rules::NESTING_DEPTH)]);

    assert!(
        demanded.contains(&RequiredFact::LimitsPolicy),
        "NESTING_DEPTH declares RequiredFact::LimitsPolicy and selecting it alone demanded \
         {demanded:?}. This is the drift the derivation exists to close: a rule that runs \
         without the fact it declared judges against a built-in default and says nothing."
    );
}

/// A family no selected rule declares is not demanded.
///
/// The converse control. Without it the assertion above is satisfied by a derivation that
/// demands everything always, which would be correct and useless -- every run would pay
/// for every provider, and the selection `RuleSelector` exists to express would buy
/// nothing.
#[test]
fn Test_A_Family_No_Selected_Rule_Declares_Should_Not_Be_Demanded()
{
    let demanded = Demanded_Families(&[RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE)]);

    assert!(
        demanded.is_empty(),
        "NO_TRAILING_WHITESPACE declares no required fact -- its descriptor's requires is \
         empty -- and selecting it alone demanded {demanded:?}. A derivation that demands \
         a family nobody asked for makes every narrowed run pay for every provider."
    );
}

/// The demand is exactly the union over the selected rules' declarations.
///
/// Checked against `DESCRIPTORS` itself rather than against a list written here, so a
/// rule whose declaration changes moves this with it and no second statement of the
/// relation can appear for the first one to drift against.
#[test]
fn Test_The_Demand_Should_Be_The_Union_Of_What_The_Selected_Rules_Declare()
{
    for descriptor in DESCRIPTORS
    {
        let demanded = Demanded_Families(&[RuleId::New(descriptor.id)]);

        for family in descriptor.requires
        {
            assert!(
                demanded.contains(family),
                "{} declares {family:?} and selecting it demanded {demanded:?}",
                descriptor.id
            );
        }

        assert!(
            demanded.len() == descriptor.requires.len(),
            "{} declares {:?} and selecting it alone demanded {demanded:?} -- a family \
             nothing selected asked for",
            descriptor.id,
            descriptor.requires
        );
    }
}

/// Selecting everything demands every family any rule declares, once each.
#[test]
fn Test_Selecting_Everything_Should_Demand_Each_Family_Once()
{
    let demanded = Demanded_Families(&[]);

    for descriptor in DESCRIPTORS
    {
        for family in descriptor.requires
        {
            assert!(
                demanded.contains(family),
                "{} declares {family:?} and a run selecting everything demanded {demanded:?}",
                descriptor.id
            );
        }
    }

    Assert_Each_Family_Demanded_Once(&demanded);
}

/// The second half of the claim above: no family appears twice, since one run considers each
/// family's section exactly once. Extracted rather than left inline because it is the whole
/// of what that test asserts after its first loop, and the loop it replaces there was longer
/// than the assertion it carried.
fn Assert_Each_Family_Demanded_Once(demanded: &[RequiredFact])
{
    let mut seen = Vec::new();
    for family in demanded
    {
        assert!(
            !seen.contains(&family),
            "{family:?} appears twice in {demanded:?}, so its section would be considered \
             more than once for one run"
        );
        seen.push(family);
    }
}

/// How many times the counting provider below was asked to answer.
///
/// A `static` and not a captured counter because the port it stands in for is a plain `fn`
/// pointer, which captures nothing -- the same reason
/// `crate::tests::materialization::CountingLauncher` can hold a `Cell` and this cannot. One
/// test reads and writes it, so nothing races it.
static COMPILER_PROVIDER_CALLS: AtomicUsize = AtomicUsize::new(0);

/// A compiler-backed provider that records it was asked and answers nothing.
///
/// It refuses rather than answering, because the question this stands in for is whether it
/// was *reached at all* -- and a refusal is cheap, where the real provider behind this port
/// loads a sysroot and every crate the analyzed root resolves to.
fn Counting_Compiler_Provider(_root: &Path, _context: &Context, _environment: &StdEnvironment) -> Result<SubjectFact, String>
{
    COMPILER_PROVIDER_CALLS.fetch_add(1, Ordering::SeqCst);

    return Err("the counting provider never answers; it only records that it was asked".to_owned());
}

/// A run whose selection declares no compiler family never reaches the compiler-backed
/// provider, and one that does reaches it exactly once.
///
/// This is `P123`'s own economic claim, and findings cannot prove it: a family that was never
/// materialized and a family that was materialized and found nothing render identically over
/// a finding list. Counting the provider's own calls is the distinguishing evidence, the same
/// shape `Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_Metadata` already uses
/// for a subprocess -- a counting launcher cannot serve here, because this provider launches
/// no process: it links `ra_ap_hir` and calls it in-process.
///
/// Both halves are one test rather than two because the counter is a `static` shared by the
/// whole test binary, and two tests reading it would be two tests racing it.
#[test]
fn Test_A_Selection_Declaring_No_Compiler_Family_Should_Never_Reach_Its_Provider()
{
    let without = Demanded_Families(&[RuleId::New(nomos_rules::NO_TRAILING_WHITESPACE)]);
    let with = Demanded_Families(&[RuleId::New(nomos_rules::COPY_CLONES)]);

    let before = COMPILER_PROVIDER_CALLS.load(Ordering::SeqCst);
    Run_Compiler_Section(&without);

    assert_eq!(
        COMPILER_PROVIDER_CALLS.load(Ordering::SeqCst),
        before,
        "NO_TRAILING_WHITESPACE declares no required fact, so a run selecting it alone must          never load a compiler frontend -- demanded {without:?}"
    );

    Run_Compiler_Section(&with);

    assert_eq!(
        COMPILER_PROVIDER_CALLS.load(Ordering::SeqCst),
        before.saturating_add(1),
        "the control: COPY_CLONES declares RequiredFact::CopyClones, so selecting it must          reach the provider exactly once -- demanded {with:?}"
    );
}

/// Runs the copy-clones section over `demanded` with [`Counting_Compiler_Provider`] in place
/// of the composed one, and discards what it produced: this test reads the counter, not the
/// answer.
fn Run_Compiler_Section(demanded: &[RequiredFact])
{
    let providers = crate::composition::Composed_Providers::<StdProgramLauncher, StdFileSystem, StdEnvironment>();
    let mut store = MemoryFactStore::New();
    let context = Counting_Context();
    let root = std::env::temp_dir();
    let mut env = MaterializationEnvironment {
        root: &root,
        context: &context,
        store: &mut store,
        launcher: &StdProgramLauncher,
        filesystem: &StdFileSystem,
        environment: &StdEnvironment,
        providers: &providers,
    };

    let _discarded = Materialize_Compiler_Section(&mut env, demanded, COPY_CLONES_FAMILY, Counting_Compiler_Provider);
}

/// Fill bytes distinct enough that this context's three digests differ from one another;
/// each value carries no meaning beyond "not equal to the others", and nothing here reads
/// them -- the provider this drives refuses before it ever files a fact.
const SNAPSHOT_FILL: u8 = 1;
const VARIANT_FILL: u8 = 2;
const CONFIGURATION_FILL: u8 = 3;

fn Counting_Context() -> Context
{
    return Context {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_FILL; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_FILL; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}
