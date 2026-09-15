//! Every rule the ledger must satisfy, checked over a whole document.

use nomos_platform::Timestamp;
use std::collections::{BTreeMap, BTreeSet};

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
        Check_Widenings(item, &mut violations);
    }

    violations.extend(Dependency_Cycles(document));

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
            "{} is marked claimed and records no claim, so nothing can say who holds it or held it",
            item.id
        ));
    }
    if item.state == ItemState::Done && item.verified.is_none()
    {
        violations.push(format!(
            "{} is done with no recorded verification; done_when is prose, and prose is not a predicate",
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
            concat!(
                "{}'s territory lists `{first}` and `{second}`, which name the same subject; ",
                "whoever wrote it probably believed they were reserving two things",
            ),
            item.id,
            first = first,
            second = second
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
            concat!(
                "{}'s territory carries unexpanded pattern(s) {:?}, which compare as unknown ",
                "rather than as touching nothing and must not reach the board that way",
            ),
            item.id, item.territory.patterns
        ));
    }
}

/// A recorded widening naming a path the territory does not hold.
///
/// The two are written in one operation and there is no ordering of [`crate::LedgerItem::Widen`]
/// in which the row lands and the paths do not, so this cannot be reached by the verb. It is
/// here for the two ways it can be reached anyway: a hand edit, and a future writer that grows
/// the territory and records the enlargement as two statements rather than one.
///
/// What it protects is the measurement. These rows exist so that how often a predicted cone
/// escaped its reservation is answerable from the board, and a row naming a path nothing
/// reserves overstates the escape while looking exactly like a row that does not. Compared
/// after [`crate::Normalize_Path`], for the reason the verb compares that way: two spellings of
/// one path are one path.
fn Check_Widenings(item: &LedgerItem, violations: &mut Vec<String>)
{
    use crate::Normalize_Path;

    for widening in &item.widened
    {
        for added in &widening.added
        {
            let held = item
                .territory
                .paths
                .iter()
                .any(|path| return Normalize_Path(path) == Normalize_Path(added));

            if !held
            {
                violations.push(format!(
                    concat!(
                        "{} records a widening by {} that added `{added}`, which its territory does ",
                        "not reserve; the enlargement and the record it is kept in have come apart",
                    ),
                    item.id,
                    widening.holder,
                    added = added
                ));
            }
        }
    }
}

/// Sets of items that can only finish after one another, and so can never finish.
///
/// # Why `Check_Dependencies` does not already cover this
///
/// That check asks whether each named dependency is an item the ledger holds. Every member of
/// a ring names a real item, so a ring is a valid document by that check -- and `Load` refuses
/// only invalid documents, which means a cycle loads and nothing downstream looks again. What
/// follows is silence rather than an error: every item in the ring lists as `waiting`, the
/// label for a dependency that has not finished, and stays that way forever because no member
/// can finish before another member that cannot finish either. The `next:` line computed over
/// the whole board never names any of them, and `stranded` -- the one label that says a
/// dependency will never finish -- is reserved by `OD-LEDGER-020` for a *declined* dependency
/// and does not fire here.
///
/// # Why one violation per ring and not one per member
///
/// A caller has one thing to break. Reporting each member separately would read as several
/// defects and would still not say which items have to be considered together to repair any
/// of them.
///
/// # Why mutual reachability and not a walk that remembers what it has seen
///
/// A diamond -- two paths from one item down to another -- arrives at its bottom item twice,
/// so a check written on "have I been here before" reports it as a ring. That shape is
/// ordinary and common on a real board. What makes a ring a ring is that its members reach
/// *each other*, and `Test_An_Acyclic_Diamond_Should_Not_Be_Reported` is the guard that keeps
/// the two apart.
///
/// Put that way the self-dependency needs no case of its own: an item naming itself reaches
/// itself, which is what every member of every longer ring also does.
fn Dependency_Cycles(document: &LedgerDocument) -> Vec<String>
{
    let reaches = Reachability_Of(document);

    let mut rings: Vec<String> = Vec::new();

    Report_Rings(&reaches, &mut rings);

    rings.sort();

    return rings;
}

/// Every item each item can reach through `depends_on`, directly or at any remove.
///
/// A dependency naming nothing the ledger holds is `Check_Dependencies`' violation and is
/// skipped here rather than reported a second time in a second vocabulary.
fn Reachability_Of(document: &LedgerDocument) -> BTreeMap<&ItemId, BTreeSet<&ItemId>>
{
    let mut edges: BTreeMap<&ItemId, Vec<&ItemId>> = BTreeMap::new();
    for item in &document.items
    {
        edges.insert(&item.id, item.depends_on.iter().collect());
    }

    let mut reaches = BTreeMap::new();
    for item in &document.items
    {
        let reachable = Reachable_From(&edges, &item.id);
        reaches.insert(&item.id, reachable);
    }

    return reaches;
}

/// Every item `from` can reach through `edges`, directly or at any remove.
///
/// A dependency naming nothing the ledger holds stops the walk rather than being followed:
/// `Check_Dependencies` reports that one, and a second report in a second vocabulary would
/// read as a second defect.
fn Reachable_From<'a>(edges: &BTreeMap<&'a ItemId, Vec<&'a ItemId>>, from: &'a ItemId) -> BTreeSet<&'a ItemId>
{
    let mut downstream: BTreeSet<&ItemId> = BTreeSet::new();
    let mut pending: Vec<&ItemId> = edges.get(from).cloned().unwrap_or_default();

    while let Some(next) = pending.pop()
    {
        if !edges.contains_key(next) || !downstream.insert(next)
        {
            continue;
        }
        pending.extend(edges.get(next).cloned().unwrap_or_default());
    }

    return downstream;
}

/// Appends one violation per ring among `reaches`.
///
/// A ring already named by an earlier one is skipped rather than counted twice: two rings
/// sharing a member are one thing for a caller to break, and `already_reported` is what
/// keeps the second from reading as a second defect.
fn Report_Rings(reaches: &BTreeMap<&ItemId, BTreeSet<&ItemId>>, rings: &mut Vec<String>)
{
    let mut already_reported: BTreeSet<&ItemId> = BTreeSet::new();

    for (item, downstream) in reaches
    {
        if already_reported.contains(item) || !downstream.contains(item)
        {
            continue;
        }

        let ring: Vec<&ItemId> = downstream
            .iter()
            .copied()
            .filter(|other| return reaches.get(other).is_some_and(|from_other| return from_other.contains(item)))
            .collect();

        for member in &ring
        {
            already_reported.insert(member);
        }
        rings.push(Ring_Violation(&ring));
    }
}

/// One ring, said so a reader knows which items have to be considered together.
fn Ring_Violation(members: &[&ItemId]) -> String
{
    let Some(first) = members.first()
    else
    {
        // Unreachable: a ring always holds the item that was found to reach itself.
        return "an empty dependency cycle was reported, which is a defect in this check".to_owned();
    };

    if members.len() == 1
    {
        return format!("{first} depends on itself, which is a dependency cycle of one, so it can never finish");
    }

    let named: Vec<&str> = members.iter().map(|member| return member.As_Text()).collect();

    return format!("{} form a dependency cycle, so none of them can ever finish", named.join(", "));
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

    let intersection = item.territory.Intersect(&other.territory);

    return match intersection
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

#[cfg(test)]
#[path = "validation/tests.rs"]
mod tests;

/// Narrow, file-local proof for this file's own public function, addressed by name.
///
/// [`tests`] above is `validation/tests.rs`, a separate physical file whose behavioural
/// suite this does not repeat or replace. `check-test-coverage`'s Rust front end keys a
/// test's companion unit off the literal file it is textually written in, so a test living
/// in that separate file can never address a function declared here, however it is named —
/// this module gives [`Validate_Document`] the one-file address the check reads.
#[cfg(test)]
mod self_tests
{
    use super::*;

    #[test]
    fn Test_Validate_Document_Should_Find_Nothing_Wrong_With_An_Empty_Board()
    {
        let empty = LedgerDocument {
            schema_version: crate::SCHEMA_VERSION,
            items: Vec::new(),
        };

        let violations = Validate_Document(&empty, Timestamp::From_Unix_Seconds(0));

        assert!(violations.is_empty(), "{violations:?}");
    }
}
