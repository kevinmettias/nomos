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

        let Some(file) = Published_Counterpart(&mine, published)
        else
        {
            return Err(AddRefusal::AmendmentNotPublished {
                identifier: Normalize_Path(declared),
            });
        };

        Refuse_A_Misspelled_Amendment(AmendmentSpelling { declared, file: &file })?;
    }

    return Ok(());
}

/// The published record file a declaration folds onto, or `None`.
///
/// Returns which file rather than a bool, because the two callers of the answer want
/// different halves of it: one needs to know an amendment names something, and the other
/// needs the filename to put in front of an author who spelled it wrongly.
fn Published_Counterpart(mine: &Territory, published: &Territory) -> Option<String>
{
    return published
        .paths
        .iter()
        .find(|file| {
            let theirs = Territory::Of_Files([(*file).clone()]);
            return matches!(mine.Intersect(&theirs), Intersection::Overlaps(_));
        })
        .cloned();
}

/// Refuses a declaration that claims to be a record's filename and is a different one.
///
/// The reservation this guards is already correct without it: `OD-LEDGER-016`'s fold makes
/// every slug for one identifier the same subject, so a mistyped slug excludes exactly the
/// writers the right one would have. That is why nothing else catches it, and why what the
/// item keeps is a path opening nothing while its own success line names that path as the
/// file it reserved.
///
/// Judged on the `.md` suffix rather than on whether the file exists on disk, because this
/// crate does not read the filesystem to decide a territory and should not learn to for
/// this: `published` is what the caller says the repository has published, and comparing
/// against it keeps the answer a function of the arguments. The suffix is what separates the
/// two spellings the fold admits — a bare `docs/records/OD-LEDGER-016` claims no filename
/// and so cannot have got one wrong.
///
/// Compared through [`nomos_model::Normalize_Path`] and not through this crate's own
/// [`Normalize_Path`], which is the only subtle line here. The latter folds a record
/// filename onto its identifier, and that fold is precisely the difference being looked for:
/// through it every slug for one identifier is one string, the suffix is gone with the slug,
/// and this function can never see anything to refuse. The plain fold still settles
/// separators and case, so `Docs\Records\OD-LEDGER-006-x.md` is not called a misspelling of
/// `docs/records/od-ledger-006-x.md`.
fn Refuse_A_Misspelled_Amendment(spelling: AmendmentSpelling<'_>) -> Result<(), AddRefusal>
{
    let AmendmentSpelling { declared, file } = spelling;
    let spelled = nomos_model::Normalize_Path(declared);

    if !spelled.ends_with(RECORD_FILE_SUFFIX) || spelled == nomos_model::Normalize_Path(file)
    {
        return Ok(());
    }

    return Err(AddRefusal::AmendmentMisspelled {
        identifier: Normalize_Path(file),
        declared: declared.to_owned(),
        file: file.to_owned(),
    });
}

/// What an item declared as a record's filename, and the file it actually names -- paired
/// so a caller cannot transpose which is which, since both are `&str` and the compiler
/// cannot catch a swap between them on its own.
struct AmendmentSpelling<'a>
{
    /// The path the item declared, as it was authored.
    declared: &'a str,
    /// The file this repository actually published the identifier as.
    file: &'a str,
}

/// The folded record directory, with its separator, as [`Normalize_Path`] leaves it.
const RECORD_DIRECTORY_PREFIX: &str = "docs/records/";

/// What a declaration ends with when it claims to be a record's filename rather than its
/// bare identifier.
const RECORD_FILE_SUFFIX: &str = ".md";

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ItemId, ItemKind, ItemOrigin, ItemState};

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

    fn Document_Of(items: Vec<LedgerItem>) -> LedgerDocument
    {
        return LedgerDocument { schema_version: crate::SCHEMA_VERSION, items };
    }

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
