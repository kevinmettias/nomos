//! What refuses an item before it can join the board, decided over its territory.


use nomos_model::Intersection;

use crate::add_refusal::AddRefusal;
use crate::item::LedgerItem;
use crate::item_state::ItemState;
use crate::ledger_document::LedgerDocument;
use crate::territory::{Normalize_Path, Territory};



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
    published: &Territory,
    amending: &Territory,
) -> Result<(), AddRefusal>
{
    Refuse_An_Unpublished_Amendment(amending, published)?;

    for reserved in Record_Reservations(&item.territory)
    {
        let mine = Territory::Of_Files([reserved.clone()]);

        if !matches!(mine.Intersect(amending), Intersection::Overlaps(_))
        {
            Refuse_If_Published(&mine, &reserved, published)?;
        }

        Refuse_If_Reserved(&mine, &reserved, document)?;
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

/// The folded record directory, with its separator, as [`Normalize_Path`] leaves it.
const RECORD_DIRECTORY_PREFIX: &str = "docs/records/";
