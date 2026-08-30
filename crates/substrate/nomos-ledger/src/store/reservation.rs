//! What refuses an item before it can join the board, decided over its territory.

use nomos_model::Intersection;

use crate::AddRefusal;
use crate::LedgerItem;
use crate::LedgerDocument;
use crate::{Normalize_Path, Territory};

/// What an item declares about record identifiers, as one value.
///
/// The two territories are one thing to everybody who reads them. The verb that receives
/// them only forwards them; the refusal below takes both and begins by comparing one against
/// the other, because `amending` is meaningful only as a subset of `published`. Loose, they
/// are also two arguments of one type sitting next to each other, which a call site can
/// transpose without the compiler noticing.
///
/// Borrowed rather than owned: this is a grouping of arguments already borrowed from the
/// caller, and copying two territories to satisfy the grouping would be paying for the shape.
pub(super) struct RecordDeclaration<'a>
{
    /// The record files this repository has already published.
    pub(super) published: &'a Territory,
    /// The subset the item declares it is editing rather than allocating.
    pub(super) amending: &'a Territory,
}

/// Refuses an item whose territory reserves a record identifier that is already spent.
///
/// Two comparisons, in the order an author can act on. A published identifier is spent
/// forever and the remedy is unconditional; a reserved one may be released, so being told
/// about it second is being told about the one that might still resolve itself.
///
/// Both are decided by [`Territory::Intersect`], one authored path at a time so the refusal
/// can name which. That is not a second containment rule beside the first: `Intersect` folds
/// a record filename onto the identifier it carries, which is what makes
/// `docs/records/OD-LEDGER-025` and `docs/records/OD-LEDGER-025-a-slug.md` one subject
/// without anything here knowing the grammar. `OD-LEDGER-016` is that decision and this is
/// the second caller to rely on it — and the reason a declaration may be spelled either way
/// too, since the same fold decides whether a declaration covers a reservation.
///
/// Only the *published* comparison is conditional. An amendment and an allocation both edit
/// one file, so two open items reserving one record still refuse each other whichever act
/// each intends: that is territory doing its ordinary job, and exempting amendments from it
/// would put two writers on one record with nothing between them.
pub(super) fn Refuse_A_Spent_Record(
    item: &LedgerItem,
    document: &LedgerDocument,
    declared: &RecordDeclaration,
) -> Result<(), AddRefusal>
{
    Refuse_An_Unpublished_Amendment(declared.amending, declared.published)?;

    for reserved in Record_Reservations(&item.territory)
    {
        let mine = Territory::Of_Files([reserved.clone()]);

        if !matches!(mine.Intersect(declared.amending), Intersection::Overlaps(_))
        {
            Refuse_If_Published(&mine, &reserved, declared.published)?;
        }

        Refuse_If_Reserved(&mine, &reserved, document)?;
    }

    return Ok(());
}

/// The authored paths in a territory that name a record identifier rather than a file.
///
/// Scoped deliberately. `add` does not refuse overlapping territory in general and must not
/// start: items overlap constantly and claims are what serialize them. What is being guarded
/// is the one reservation an author cannot recover from mid-claim, because there is no
/// `work edit` to move a record identifier once somebody else has published it.
///
/// Recognised through [`Normalize_Path`] rather than by re-reading the grammar here. A path
/// names an identifier when folding lands it directly inside the record directory: both
/// `docs/records/OD-LEDGER-025` and `docs/records/OD-LEDGER-025-a-slug.md` fold to
/// `docs/records/od-ledger-025`, and anything nested deeper is some other thing that happens
/// to live there. The directory itself is not an identifier — reserving all of it is a
/// different problem, and `P10-RECORD-LOCK` is where it was answered.
fn Record_Reservations(territory: &Territory) -> Vec<String>
{
    return territory
        .paths
        .iter()
        .filter(|path| {
            let folded = Normalize_Path(path);
            let Some(within) = folded.strip_prefix(RECORD_DIRECTORY_PREFIX)
            else
            {
                return false;
            };

            return !within.is_empty() && !within.contains('/');
        })
        .cloned()
        .collect();
}

/// Refuses an identifier some committed record already carries.
///
/// Reached only for a reservation the item did not declare as an amendment, so arriving here
/// already means the item is allocating. The refusal still names both acts, because the
/// commonest way to be here is having meant the other one.
fn Refuse_If_Published(
    mine: &Territory,
    reserved: &str,
    published: &Territory,
) -> Result<(), AddRefusal>
{
    for file in &published.paths
    {
        let theirs = Territory::Of_Files([file.clone()]);

        if matches!(mine.Intersect(&theirs), Intersection::Overlaps(_))
        {
            return Err(AddRefusal::RecordPublished {
                identifier: Normalize_Path(reserved),
                file: file.clone(),
            });
        }
    }

    return Ok(());
}

/// Refuses an identifier another open item is already holding.
///
/// Only open items reserve. A closed item's territory is history, and refusing against it
/// would make every finished item a permanent claim on its number — which would refuse the
/// whole board, since almost every item ever written reserved a record.
fn Refuse_If_Reserved(
    mine: &Territory,
    reserved: &str,
    document: &LedgerDocument,
) -> Result<(), AddRefusal>
{
    use crate::ItemState;

    for other in &document.items
    {
        if !matches!(other.state, ItemState::Ready | ItemState::Claimed)
        {
            continue;
        }

        if matches!(mine.Intersect(&other.territory), Intersection::Overlaps(_))
        {
            return Err(AddRefusal::RecordReserved {
                identifier: Normalize_Path(reserved),
                item: other.id.clone(),
            });
        }
    }

    return Ok(());
}

/// Refuses a declared amendment of a record this repository has not published.
///
/// Checked before the reservations rather than among them, because it is a statement about
/// the declaration itself and holds whether or not the territory reserves anything. An author
/// who declared the wrong identifier is told that here, once, instead of being told nothing
/// and getting an item that allocates while claiming to amend.
pub(super) fn Refuse_An_Unpublished_Amendment(
    amending: &Territory,
    published: &Territory,
) -> Result<(), AddRefusal>
{
    for declared in &amending.paths
    {
        let mine = Territory::Of_Files([declared.clone()]);

        if !matches!(mine.Intersect(published), Intersection::Overlaps(_))
        {
            return Err(AddRefusal::AmendmentNotPublished {
                identifier: Normalize_Path(declared),
            });
        }
    }

    return Ok(());
}

/// The folded record directory, with its separator, as [`Normalize_Path`] leaves it.
const RECORD_DIRECTORY_PREFIX: &str = "docs/records/";

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ItemId, ItemKind, ItemOrigin, ItemState};

    fn Item_Reserving(id: &str, path: &str) -> LedgerItem
    {
        return LedgerItem {
            id: ItemId::New(id),
            title: "an item".to_owned(),
            why: "because".to_owned(),
            done_when: "when it is done".to_owned(),
            kind: ItemKind::Decision,
            origin: ItemOrigin::Proposed,
            territory: Territory::Of_Files([path.to_owned()]),
            state: ItemState::Ready,
            depends_on: Vec::new(),
            blocked: None,
            claim: None,
            verification: None,
            verified: None,
            abandoned: Vec::new(),
            displaced: Vec::new(),
            declined: None,
        };
    }

    fn Document_Of(items: Vec<LedgerItem>) -> LedgerDocument
    {
        return LedgerDocument { schema_version: crate::SCHEMA_VERSION, items };
    }

    #[test]
    fn Test_Refuse_A_Spent_Record_Should_Refuse_An_Undeclared_Reservation_Of_A_Published_Identifier()
    {
        let published = Territory::Of_Files(["docs/records/OD-LEDGER-999-a-slug.md"]);
        let document = Document_Of(Vec::new());

        let undeclared = Item_Reserving("NEW-1", "docs/records/OD-LEDGER-999");
        let refusal = Refuse_A_Spent_Record(
            &undeclared,
            &document,
            &RecordDeclaration { published: &published, amending: &Territory::Empty() },
        )
        .expect_err("an unamended, already-published identifier must refuse");
        assert!(matches!(refusal, AddRefusal::RecordPublished { .. }), "got {refusal:?}");

        let amending = Territory::Of_Files(["docs/records/OD-LEDGER-999"]);
        let declared_edit = Item_Reserving("EDIT-1", "docs/records/OD-LEDGER-999");
        Refuse_A_Spent_Record(
            &declared_edit,
            &document,
            &RecordDeclaration { published: &published, amending: &amending },
        )
        .expect("a declared amendment of a published record must be accepted");
    }

    #[test]
    fn Test_Refuse_An_Unpublished_Amendment_Should_Refuse_A_Declared_Edit_Of_Nothing_Published()
    {
        let amending = Territory::Of_Files(["docs/records/OD-LEDGER-888"]);
        let published = Territory::Empty();

        let refusal =
            Refuse_An_Unpublished_Amendment(&amending, &published).expect_err("amending nothing published must refuse");

        assert!(matches!(refusal, AddRefusal::AmendmentNotPublished { .. }), "got {refusal:?}");
    }
}
