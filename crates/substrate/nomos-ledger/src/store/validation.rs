//! Every rule the ledger must satisfy, checked over a whole document.

use nomos_model::Intersection;
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
pub fn Validate(document: &LedgerDocument, now: Timestamp) -> Vec<String>
{
    let mut violations = Duplicate_Identifiers(document);
    let declared: Vec<&ItemId> = document.items.iter().map(|item| return &item.id).collect();

    for item in &document.items
    {
        Check_Dependencies(item, &declared, &mut violations);
        Check_State(item, &mut violations);
        Check_Predicate(item, &mut violations);
        Check_Territory(item, &mut violations);
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
    let holder = Holder_Of(item);
    let other_holder = Holder_Of(other);

    return match item.territory.Intersect(&other.territory)
    {
        Intersection::Disjoint => None,
        Intersection::Overlaps(shared) => Some(format!(
            "{} (held by {holder}) and {} (held by {other_holder}) both claim {} overlapping \
             subject(s)",
            item.id,
            other.id,
            shared.len()
        )),
        Intersection::Unknown(reason) => Some(format!(
            "{} (held by {holder}) and {} (held by {other_holder}) cannot be shown \
             independent: {}",
            item.id,
            other.id,
            reason.Describe()
        )),
    };
}

/// Who holds a claim, for a message that has to name somebody either way.
fn Holder_Of(item: &LedgerItem) -> &str
{
    return item
        .claim
        .as_ref()
        .map_or("someone", |claim| return claim.holder.as_str());
}
