//! Every rule the ledger must satisfy, checked over a whole document.

use nomos_platform::Timestamp;

use crate::LedgerItem;
use crate::ItemId;
use crate::ItemState;
use crate::LedgerDocument;

/// Every way a ledger can be internally inconsistent.
///
/// Returns all violations rather than the first. An author fixing one at a time and
/// re-running is an author who stops running it.
#[must_use]
pub fn Validate_Document(document: &LedgerDocument, now: Timestamp) -> Vec<String>
{
    let mut violations = Duplicate_Identifiers(document);
    let declared: Vec<&ItemId> = document.items.iter().map(|item| return &item.id).collect();

    for item in &document.items
    {
        Check_Dependencies(item, &declared, &mut violations);
        Check_State(item, &mut violations);
        Check_Predicate(item, &mut violations);
        Check_Territory(item, &mut violations);
        Check_Pattern(item, &mut violations);
    }

    let overlapping = Overlapping_Claims(document, now);
    violations.extend(overlapping);

    return violations;
}

/// Identifiers that appear more than once, which makes every lookup ambiguous.
fn Duplicate_Identifiers(document: &LedgerDocument) -> Vec<String>
{
    let mut violations = Vec::new();
    let mut seen: Vec<&ItemId> = Vec::new();

    for item in &document.items
    {
        if seen.contains(&&item.id)
        {
            violations.push(format!("{} appears more than once", item.id));
        }
        seen.push(&item.id);
    }

    return violations;
}

/// Dependencies naming an item the ledger does not hold.
fn Check_Dependencies(item: &LedgerItem, declared: &[&ItemId], violations: &mut Vec<String>)
{
    for dependency in &item.depends_on
    {
        if !declared.contains(&dependency)
        {
            violations.push(format!(
                "{} depends on {dependency}, which is not in the ledger",
                item.id
            ));
        }
    }
}

/// What a state must be able to say about itself.
///
/// The claimed case is deliberately `claim.is_none()` and not `!Has_Active_Claim(now)`.
///
/// A lease expiring is `Claimed` with an inactive claim, and it is the normal end of an
/// agent that died rather than a corruption. Validating against the clock made a document's
/// validity a function of when it was read: one written valid stopped being valid on its
/// own, `Load` refuses an invalid document, and every operation loads first — so one lapsed
/// lease refused every claim on the board, including items sharing no territory with it.
/// `MAXIMUM_LEASE` exists to stop a crashed agent holding territory until somebody edits the
/// file, and the lease expiring was causing exactly what the lease exists to prevent.
///
/// What remains is the invariant that does not move: an item claimed by nobody records who
/// claimed it. That is a real corruption — nothing can say whose work was abandoned — and it
/// cannot arrive by the passage of time.
fn Check_State(item: &LedgerItem, violations: &mut Vec<String>)
{
    if item.state == ItemState::Blocked && item.blocked.is_none()
    {
        violations.push(format!("{} is blocked without saying why", item.id));
    }
    if item.state == ItemState::Claimed && item.claim.is_none()
    {
        violations.push(format!(
            "{} is marked claimed and records no claim, so nothing can say who holds it or \
             held it",
            item.id
        ));
    }
    if item.state == ItemState::Done && item.verified.is_none()
    {
        violations.push(format!(
            "{} is done with no recorded verification; done_when is prose, and prose is not \
             a predicate",
            item.id
        ));
    }

}

/// A predicate nothing could run, which makes `done_when` unenforceable.
fn Check_Predicate(item: &LedgerItem, violations: &mut Vec<String>)
{
    if let Some(predicate) = &item.verification
        && !predicate.Is_Runnable()
    {
        violations.push(format!(
            "{} carries a verification predicate that cannot be run",
            item.id
        ));
    }
}

/// What a territory must be able to exclude.
///
/// An item somebody can pick up must say what it touches. An empty territory is disjoint
/// from every other territory, so two agents working an unstated item are told they may both
/// proceed — the ledger answers the exclusion question confidently and wrongly. Silence
/// about territory is not a claim of touching nothing.
fn Check_Territory(item: &LedgerItem, violations: &mut Vec<String>)
{
    for (first, second) in item.territory.Ambiguous_Paths()
    {
        violations.push(format!(
            "{}'s territory lists `{first}` and `{second}`, which name the same subject; \
             whoever wrote it probably believed they were reserving two things",
            item.id
        ));
    }

    if matches!(item.state, ItemState::Ready | ItemState::Claimed) && item.territory.Is_Empty()
    {
        violations.push(format!(
            "{} is workable but reserves nothing, so it excludes nobody",
            item.id
        ));
    }
}

/// A territory that still carries an unexpanded pattern.
///
/// `OD-LEDGER-013` withdrew `work add --territory-pattern`, the only authoring surface that
/// ever put a value in [`crate::Territory::patterns`], and kept the field itself so a
/// document that arrives with one anyway — hand-edited, or written by some future authoring
/// surface — still fails closed: every comparison touching a pattern answers
/// [`nomos_model::Intersection::Unknown`] rather than comparing as excluding nothing. That
/// guard covers claiming, but a pattern sitting in a `Ready` or `Claimed` item was never
/// refused by `Validate_Document` itself, which is the gap the record named and left open. This closes
/// it: any non-empty `patterns` is reported, regardless of state, because the field is only
/// ever non-empty by a hand edit that this check exists to catch.
fn Check_Pattern(item: &LedgerItem, violations: &mut Vec<String>)
{
    if !item.territory.patterns.is_empty()
    {
        violations.push(format!(
            "{}'s territory carries unexpanded pattern(s) {:?}, which compare as unknown \
             rather than as touching nothing and must not reach the board that way",
            item.id, item.territory.patterns
        ));
    }
}

/// Every pair of concurrently-claimed items whose territories are not provably disjoint.
///
/// This is the invariant the whole ledger exists to hold. Two active claims on
/// overlapping territory means two agents editing the same files, and the first one to
/// write wins silently.
fn Overlapping_Claims(document: &LedgerDocument, now: Timestamp) -> Vec<String>
{
    let active: Vec<&LedgerItem> = document
        .items
        .iter()
        .filter(|item| item.Has_Active_Claim(now))
        .collect();

    let mut violations = Vec::new();

    for (index, item) in active.iter().enumerate()
    {
        for other in active.iter().skip(index.saturating_add(1))
        {
            let contested = Not_Provably_Disjoint(item, other);
            violations.extend(contested);
        }
    }

    return violations;
}

/// What one pair of active claims has to answer for, if anything.
///
/// `Unknown` is reported alongside `Overlaps` rather than passed over. The ledger's promise
/// is that two holders were shown independent, and a comparison that could not decide has
/// not shown it.
fn Not_Provably_Disjoint(item: &LedgerItem, other: &LedgerItem) -> Option<String>
{
    use nomos_model::Intersection;

    let first = Claimant { item, holder: Holder_Of(item) };
    let second = Claimant { item: other, holder: Holder_Of(other) };

    return match item.territory.Intersect(&other.territory)
    {
        Intersection::Disjoint => None,
        Intersection::Overlaps(shared) => Some(Overlap_Message(first, second, shared.len())),
        Intersection::Unknown(reason) => Some(Unknown_Message(first, second, &reason)),
    };
}

/// One claimant in a pair being compared, and who holds it — grouped so the message
/// functions below stay under this crate's own parameter-count ceiling.
struct Claimant<'a>
{
    item: &'a LedgerItem,
    holder: &'a str,
}

/// Who holds a claim, for a message that has to name somebody either way.
fn Holder_Of(item: &LedgerItem) -> &str
{
    return item
        .claim
        .as_ref()
        .map_or("someone", |claim| return claim.holder.as_str());
}

/// Two claimants whose territory provably shares `shared_count` subject(s).
fn Overlap_Message(first: Claimant<'_>, second: Claimant<'_>, shared_count: usize) -> String
{
    return format!(
        "{} (held by {}) and {} (held by {}) both claim {shared_count} overlapping subject(s)",
        first.item.id, first.holder, second.item.id, second.holder
    );
}

/// Two claimants whose territory could not be shown independent, and why.
fn Unknown_Message(first: Claimant<'_>, second: Claimant<'_>, reason: &nomos_model::UnknownReason) -> String
{
    return format!(
        "{} (held by {}) and {} (held by {}) cannot be shown independent: {}",
        first.item.id,
        first.holder,
        second.item.id,
        second.holder,
        reason.Describe()
    );
}
