//! Every way adding an item is refused before it reaches the board.

use crate::LedgerError;
use crate::ItemId;
/// Why an item could not be put on the board.
///
/// Its own vocabulary and not a borrowed [`ClaimRefusal`] arm. Adding an item is not
/// claiming one — it takes no territory, judges no lease and consults no other holder — so
/// every arm of a claim's refusal would be a sentence about the wrong question. That is the
/// mis-subject `OD-LEDGER-014` measured, and the cost of a second small enum is smaller than
/// the cost of one arm meaning two things.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddRefusal
{
    /// The identifier is already on the board.
    ///
    /// An answer and not a store failure: the caller chose an identifier somebody else has
    /// already used, and the remedy is to choose another. Decided **inside** the lock, so two
    /// sessions adding one identifier at once cannot both be told it was free.
    AlreadyPresent
    {
        /// The identifier that is taken.
        item: ItemId,
    },
    /// The territory reserves a record identifier this repository has already published,
    /// and the item did not say it was amending it.
    ///
    /// Two acts reserve a published record and only one of them is this defect. A new
    /// decision that reached for an identifier somebody already spent must renumber. An
    /// amendment must not: it edits that record, and `OD-LEDGER-016` makes the identifier and
    /// the file one subject, so reserving the file *is* reserving the identifier. The two are
    /// told apart by the item declaring the second, never by inspecting the identifier — which
    /// is why this arm now means "undeclared" rather than "published".
    ///
    /// Distinct from [`AddRefusal::RecordReserved`] because the remedies are different and an
    /// author told the wrong one does the wrong thing. A published record is spent forever —
    /// the identifier is allocated and the file exists — so the fix is choosing another number
    /// or declaring the amendment. A reserved one belongs to an item that may yet be retired.
    ///
    /// Names the file rather than only the identifier, because an author who reads
    /// "`OD-LEDGER-025` is taken" has to go and find out by what, and the thing that answers
    /// that is a filename.
    ///
    /// A published identifier excludes nobody, which is why nothing caught this before:
    /// `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` reddens on two *open* items
    /// sharing an identifier, and one open item on a spent one is invisible to it.
    RecordPublished
    {
        /// The identifier, in the folded form both spellings reach.
        identifier: String,
        /// The record file that already carries it, as the caller found it.
        file: String,
    },
    /// The territory reserves a record identifier another open item already reserves.
    ///
    /// Decided inside the lock, and that is the whole of why it is here rather than in the
    /// command layer. Two sessions each taking the next free number read a board without the
    /// other's item and are both told it is free — which is exactly how `P11-DISPATCH-SPLIT`
    /// and this item's own first reissue both took `OD-LEDGER-022`, one add landing between
    /// the other's board read and its own.
    ///
    /// Only open items reserve. A `Done` or `Declined` item's territory is history, and
    /// refusing against it would make every closed item a permanent claim on its number.
    RecordReserved
    {
        /// The identifier, in the folded form both spellings reach.
        identifier: String,
        /// The open item that already reserves it.
        item: ItemId,
    },
    /// The item declared an amendment to a record this repository has not published.
    ///
    /// The mirror of [`AddRefusal::RecordPublished`], and here for the reason that arm no
    /// longer refuses on its own: once a declaration decides which act this is, a declaration
    /// that names nothing is a claim about the repository that the repository denies. Left
    /// unchecked it would be the cheapest way to defeat the guard — declare every reservation
    /// an amendment and no identifier is ever spent — and the author who did it by mistake
    /// would be told nothing at all.
    ///
    /// The remedy is the opposite of its mirror's. That one says the identifier is taken; this
    /// one says it is free, which means the item is allocating rather than amending and should
    /// say so.
    AmendmentNotPublished
    {
        /// The identifier the item said it was amending, folded as [`Self::RecordPublished`]
        /// folds its own.
        identifier: String,
    },
    /// The item would leave the board violating its own invariants.
    ///
    /// The commonest of these is an item that reserves nothing, which `AGENTS.md` states as a
    /// rule of the board: it would exclude nobody while looking like work.
    ///
    /// Distinct from [`AddRefusal::LedgerUnusable`] because the remedies are opposite and the
    /// exit codes differ. This one is the caller's own item to correct and the board is fine;
    /// that one means nobody can use the board until somebody looks at it. Collapsing them is
    /// how "your territory is empty" comes to read as "stop and fetch a person".
    WouldBeInvalid
    {
        /// Every violation the document would carry, not just the first.
        violations: Vec<String>,
    },
    /// The ledger itself could not be read or written.
    ///
    /// Its own arm for the reason [`ClaimRefusal::LedgerUnusable`] is: a caller told only
    /// "that identifier is taken" while the file is in fact unparseable goes and renames its
    /// item, and the rename does not help. `OD-LEDGER-009`.
    LedgerUnusable
    {
        /// What the store said, verbatim.
        cause: String,
    },
}

impl AddRefusal
{
    /// A one-line explanation a person or an agent can act on.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::AlreadyPresent { item } => format!("{item} is already on the ledger"),
            // Each names what to do next, because the two are told apart by the remedy and
            // an author who read only "taken" would pick the wrong one half the time.
            // Names both acts rather than only the commoner one. "Choose the next free one"
            // was the whole of this sentence once, and it is advice that renumbers a record
            // that should not move whenever the author meant to amend.
            Self::RecordPublished { identifier, file } => format!(
                "{identifier} is already published as {file}. If this item is a new decision, \
                 a record identifier is allocated once — choose the next free one. If it \
                 amends that record, say so with `--amends {file}`, which reserves the file it \
                 edits"
            ),
            Self::RecordReserved { identifier, item } => format!(
                "{identifier} is already reserved by {item}, which is open. Choose another \
                 identifier, or retire that item if it is not work"
            ),
            Self::AmendmentNotPublished { identifier } => format!(
                "{identifier} is declared as an amendment and no record here carries it. An \
                 amendment edits a record that exists; reserve a new one with `--territory`"
            ),
            // The wording [`LedgerError::Invalid`] would have produced, because this arm
            // exists to carry that refusal out through a different channel and not to
            // rephrase it. An operator who has seen one of these should recognise the other.
            Self::WouldBeInvalid { violations } =>
            {
                format!("ledger is invalid:\n  {}", violations.join("\n  "))
            }
            Self::LedgerUnusable { cause } => cause.clone(),
        };
    }
}

/// Which of a store failure's two meanings this is, for a caller adding an item.
///
/// [`LedgerError::Invalid`] arrives here by a different route from the rest. The others are
/// the store failing at its job; that one is [`FileLedger::Save`] doing its job, refusing a
/// document before it reaches the disk because the item just handed to it is not one the
/// board can hold. Reporting the second as the first is the conflation `OD-LEDGER-009`
/// records, and here it would cost the exit code an agent branches on.
impl From<&LedgerError> for AddRefusal
{
    fn from(error: &LedgerError) -> Self
    {
        return match error
        {
            LedgerError::Invalid { violations } => Self::WouldBeInvalid {
                violations: violations.clone(),
            },
            other => Self::LedgerUnusable {
                cause: other.to_string(),
            },
        };
    }
}
