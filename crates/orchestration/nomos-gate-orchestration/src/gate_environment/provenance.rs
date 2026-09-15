//! What a run records about the inputs that judged it.
//!
//! Every digest here answers one question -- *did this input change?* -- and each is separate because
//! each input changes for a different reason and a comparison has to be able to tell them apart. The
//! one rule they all keep is that a value which did not change must hash to the same digest on every
//! run, so the ordering a walker or a set happens to produce can never be read as a difference in the
//! repository.

use nomos_contracts::Digest128;
use nomos_platform::Timestamp;
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;

use crate::policy::{BaselineDebt, GatePolicyFile, RuleCalibration, Suppression};
use crate::{CoveragePolicy, GateCommand, SuppressionDisposition};

/// Every file this run judged, by path and content.
///
/// Sorted by path before hashing, so the walker's own ordering cannot decide the identity.
/// Two runs over identical content must agree here or a comparison reads a difference in
/// directory iteration as a difference in the repository, which is the exact failure
/// `OD-GATE-031` exists to stop -- and it would be the worst possible instance of it,
/// because nothing about the tree would have changed at all.
///
/// An unwalked root is a distinct identity rather than an empty one. `CheckOutcome` already
/// keeps `Unreadable` and `NoSource` apart, and folding them together here would let a tree
/// that could not be read match one that held nothing.
pub(super) fn Source_Digest(walked: Option<&[SourceFile]>) -> Digest128
{
    let Some(sources) = walked
    else
    {
        return Digest_Of_Owned(&[b"unwalked".to_vec()]);
    };

    let mut ordered: Vec<(&str, &str)> = sources.iter().map(|source| return (source.path.as_str(), source.text.as_str())).collect();
    ordered.sort_unstable();

    let mut parts: Vec<Vec<u8>> = vec![b"walked".to_vec()];
    for (path, text) in ordered
    {
        parts.push(path.as_bytes().to_vec());
        parts.push(text.as_bytes().to_vec());
    }

    return Digest_Of_Owned(&parts);
}

/// The policy this run judged under, in authoring order.
///
/// Authoring order rather than sorted, unlike [`Selection_Digest`], and the difference is
/// not a style choice: `SuppressionPolicy::Suppressing` and `BaselinePolicy::Tolerating`
/// both take the *first* entry that applies, so reordering two entries that match the same
/// finding changes which one wins. An identity that ignored order would call two genuinely
/// different policies the same.
pub(super) fn Policy_Digest(policy: &GatePolicyFile) -> Digest128
{
    let mut parts = Suppression_Parts(&policy.suppressions.suppressions);
    parts.extend(Baseline_Parts(&policy.baseline.debt));
    parts.extend(Calibration_Parts(&policy.adoption.calibrated));
    parts.push(Coverage_Tag(policy.coverage).as_bytes().to_vec());

    return Digest_Of_Owned(&parts);
}

/// Every field of one suppression that decides which finding it covers, in policy order.
///
/// A function rather than a loop inside [`Policy_Digest`] because the three entry kinds
/// answer the same question with different fields, and a reader comparing them should be
/// able to compare three named lists rather than three blocks in one body.
fn Suppression_Parts(entries: &[Suppression]) -> Vec<Vec<u8>>
{
    let mut parts: Vec<Vec<u8>> = Vec::new();

    for suppression in entries
    {
        parts.push(b"suppression".to_vec());
        parts.push(suppression.rule.As_Str().as_bytes().to_vec());
        parts.push(suppression.subject.Digest().Bytes().to_vec());
        parts.push(Disposition_Tag(suppression.disposition).as_bytes().to_vec());
        parts.push(suppression.rationale.as_bytes().to_vec());
        parts.push(suppression.owner.as_bytes().to_vec());
        parts.push(Expiry_Bytes(suppression.expiry));
    }

    return parts;
}

/// A suppression disposition as a stable tag.
///
/// Written out rather than derived from `Debug`, so that renaming a variant cannot silently
/// change every recorded policy identity, and exhaustive so that adding one fails to compile
/// here rather than hashing to whatever the last arm happened to be.
const fn Disposition_Tag(disposition: SuppressionDisposition) -> &'static str
{
    return match disposition
    {
        SuppressionDisposition::InlineSuppression => "inline-suppression",
        SuppressionDisposition::RepositoryPolicyException => "repository-policy-exception",
        SuppressionDisposition::TemporaryWaiver => "temporary-waiver",
        SuppressionDisposition::AcceptedBaselineDebt => "accepted-baseline-debt",
        SuppressionDisposition::FalsePositiveDisposition => "false-positive",
        SuppressionDisposition::FormalRiskAcceptance => "formal-risk-acceptance",
    };
}

/// A waiver's expiry as bytes, with absence distinct from any moment.
///
/// An empty part rather than a sentinel number: `nomos_model::Digest_Of_Parts` length-prefixes,
/// so an empty part and an eight-byte one cannot collide, and no real timestamp has to be
/// reserved to mean "none".
fn Expiry_Bytes(expiry: Option<Timestamp>) -> Vec<u8>
{
    return match expiry
    {
        Some(moment) => moment.Unix_Seconds().to_le_bytes().to_vec(),
        None => Vec::new(),
    };
}

/// Every field of one baseline entry that decides which finding it tolerates, in policy order.
fn Baseline_Parts(entries: &[BaselineDebt]) -> Vec<Vec<u8>>
{
    let mut parts: Vec<Vec<u8>> = Vec::new();

    for debt in entries
    {
        parts.push(b"baseline".to_vec());
        parts.push(debt.rule.As_Str().as_bytes().to_vec());
        parts.push(debt.subject.Digest().Bytes().to_vec());
        parts.push(debt.rationale.as_bytes().to_vec());
    }

    return parts;
}

/// Every field of one calibration that decides which rule it covers, in policy order.
fn Calibration_Parts(entries: &[RuleCalibration]) -> Vec<Vec<u8>>
{
    let mut parts: Vec<Vec<u8>> = Vec::new();

    for calibration in entries
    {
        parts.push(b"calibration".to_vec());
        parts.push(calibration.rule.As_Str().as_bytes().to_vec());
        parts.push(calibration.rationale.as_bytes().to_vec());
    }

    return parts;
}

/// A coverage floor as a stable tag, exhaustive for the same reason [`Disposition_Tag`] is.
const fn Coverage_Tag(coverage: CoveragePolicy) -> &'static str
{
    return match coverage
    {
        CoveragePolicy::Unset => "unset",
        CoveragePolicy::RequireCompleteness => "require-completeness",
    };
}

/// Which rules were allowed to count and which paths were in scope, sorted.
///
/// Sorted rather than in authoring order, unlike [`Policy_Digest`], because
/// `RuleSelector::Is_Included` and `ScopeSelector::Is_In_Scope` both answer with `any`, so
/// two selections listing the same things in different orders select identically. Hashing
/// the order would report a difference where a caller made none, and a stated difference
/// nobody caused is how a reader learns to stop reading them.
pub(super) fn Selection_Digest(command: &GateCommand) -> Digest128
{
    let mut rules: Vec<&str> = command.rules.include.iter().map(|rule| return rule.As_Str()).collect();
    let mut included: Vec<&str> = command.scope.include.iter().map(String::as_str).collect();
    let mut excluded: Vec<&str> = command.scope.exclude.iter().map(String::as_str).collect();
    rules.sort_unstable();
    included.sort_unstable();
    excluded.sort_unstable();

    let mut parts: Vec<Vec<u8>> = vec![b"rules".to_vec()];
    parts.extend(rules.iter().map(|rule| return rule.as_bytes().to_vec()));
    parts.push(b"include".to_vec());
    parts.extend(included.iter().map(|prefix| return prefix.as_bytes().to_vec()));
    parts.push(b"exclude".to_vec());
    parts.extend(excluded.iter().map(|prefix| return prefix.as_bytes().to_vec()));

    return Digest_Of_Owned(&parts);
}

/// What did the judging: this build's variant, and the rule set it carries.
///
/// The rule set comes from `nomos_rules::DESCRIPTORS` rather than from
/// [`crate::Registered`], which copies the same three fields into its offers. Reading the
/// table directly costs no registry composition and, more to the point, introduces no second
/// failure path into [`super::Run_Gate`]: `Registered` returns a `Result`, and a run that
/// could not name its own instrument would need a whole answer for that, for a case a static
/// table cannot produce.
pub(super) fn Instrument_Digest(variant: &BuildVariant) -> Digest128
{
    let mut parts: Vec<Vec<u8>> = vec![
        b"variant".to_vec(),
        variant.target.as_bytes().to_vec(),
        variant.profile.as_bytes().to_vec(),
        variant.toolchain.as_bytes().to_vec(),
    ];
    // `features` is a `BTreeSet`, so this is already in a stable order.
    parts.extend(variant.features.iter().map(|feature| return feature.as_bytes().to_vec()));

    parts.push(b"rules".to_vec());
    for descriptor in nomos_rules::DESCRIPTORS
    {
        parts.push(descriptor.Rule().As_Str().as_bytes().to_vec());
        parts.push(descriptor.contract_record.as_bytes().to_vec());
        parts.push(descriptor.contract_record_version.to_le_bytes().to_vec());
    }

    return Digest_Of_Owned(&parts);
}

/// The digest of an ordered sequence of owned parts.
///
/// `nomos_model::Digest_Of_Parts` borrows, and every component above builds material that
/// does not outlive its own call -- a version number as bytes, a timestamp, a subject's own
/// digest. Owning the parts and borrowing once here is what lets each component read as a
/// list of what it covers rather than as lifetime plumbing.
fn Digest_Of_Owned(parts: &[Vec<u8>]) -> Digest128
{
    let borrowed: Vec<&[u8]> = parts.iter().map(|part| return part.as_slice()).collect();

    return nomos_model::Digest_Of_Parts(&borrowed);
}
