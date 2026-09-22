//! A proposed correction: why, and what it would change.

use super::CorrectionId;
use crate::{CandidateLabel, ChangeSet, CorrectionClass, ReadWriteResolution, ReadWriteSet};

/// A proposed correction: a description of why, and the [`ChangeSet`] that would carry it
/// out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionCandidate
{
    id: CorrectionId,
    description: String,
    change: ChangeSet,
    class: CorrectionClass,
    labels: Vec<CandidateLabel>,
    read_write: Vec<ReadWriteSet>,
    reads: Vec<ReadWriteSet>,
}

impl CorrectionCandidate
{
    /// Constructs a candidate. `class` is `COR-001`'s fix-action class and `labels` is
    /// `COR-010`'s independent set of descriptive labels -- both declared by the caller,
    /// never computed here, the same way `ValidatedPlan::Commit`'s `Evidence` parameter
    /// is declared rather than judged (`OD-CORRECTIONS-002`).
    ///
    /// `COR-EXEC-001` requires every correction operation to carry a read/write set at the
    /// strongest resolution available to it. `change` is the only source this constructor
    /// has, so it seeds [`Self::Read_Write`] with one [`ReadWriteSet::Declared`] at
    /// [`ReadWriteResolution::Artifact`] -- the same paths [`ChangeSet::Touched`] already
    /// answers, now carried as the typed fact the requirement names rather than left as an
    /// answer a caller has to know to ask `change` for. A caller with something stronger
    /// to say -- a resolved symbol, a semantic fact a provider produced -- adds it with
    /// [`Self::With_Read_Write`]; nothing here ever removes the artifact-tier floor, since
    /// a stronger claim about part of a change is not a claim about the rest of it.
    #[must_use]
    pub fn New(
        description: impl Into<String>,
        change: ChangeSet,
        class: CorrectionClass,
        labels: Vec<CandidateLabel>,
    ) -> Self
    {
        let description = description.into();
        let id = Identity_Of(&description, &change);
        let read_write = vec![Artifact_Tier_Of(&change)];

        return Self {
            id,
            description,
            change,
            class,
            labels,
            read_write,
            reads: Vec::new(),
        };
    }

    /// Adds a set of subjects this candidate reads and does not write -- a manifest it
    /// consulted for a version, a sibling file whose content its edit assumes.
    ///
    /// Every set on [`Self::Read_Write`] is both read and written, because the door
    /// asserts a path's prior content before replacing it (`StagedPlan`'s staleness check),
    /// so a read-only claim is the one thing [`Self::With_Read_Write`] cannot say. It is
    /// declared, never computed: a [`ChangeSet`] knows what its author changed and nothing
    /// about what its author looked at to decide. `COR-EXEC-003` is what reads it -- a
    /// plan that writes a subject another plan only reads cannot share a wave with it,
    /// and without this the read half of that comparison would be empty for every plan.
    #[must_use]
    pub fn With_Read(mut self, set: ReadWriteSet) -> Self
    {
        self.reads.push(set);

        return self;
    }

    /// Every read-only set this candidate declares -- empty unless [`Self::With_Read`]
    /// added one, since nothing here can derive a read from a change.
    #[must_use]
    pub fn Reads(&self) -> &[ReadWriteSet]
    {
        return &self.reads;
    }

    /// Adds a read/write set beyond the artifact-tier floor [`Self::New`] always seeds --
    /// a stronger declared claim, or a derived one carrying the provenance
    /// [`crate::DerivedProvenance`] requires.
    #[must_use]
    pub fn With_Read_Write(mut self, set: ReadWriteSet) -> Self
    {
        self.read_write.push(set);

        return self;
    }

    /// Every read/write set this candidate carries, at the strongest resolution declared
    /// or derived for it -- `COR-EXEC-001`'s own requirement, never empty: [`Self::New`]
    /// always seeds the artifact tier [`ChangeSet::Touched`] answers.
    #[must_use]
    pub fn Read_Write(&self) -> &[ReadWriteSet]
    {
        return &self.read_write;
    }

    #[must_use]
    pub const fn Id(&self) -> CorrectionId
    {
        return self.id;
    }

    #[must_use]
    pub fn Description(&self) -> &str
    {
        return &self.description;
    }

    #[must_use]
    pub const fn Change(&self) -> &ChangeSet
    {
        return &self.change;
    }

    /// `COR-001`'s fix-action class this candidate was declared under.
    #[must_use]
    pub const fn Class(&self) -> CorrectionClass
    {
        return self.class;
    }

    /// `COR-010`'s independent descriptive labels this candidate was declared with.
    #[must_use]
    pub fn Labels(&self) -> &[CandidateLabel]
    {
        return &self.labels;
    }
}

/// The one read/write set every candidate can honestly claim from its `change` alone:
/// every path [`ChangeSet::Touched`] names, declared at [`ReadWriteResolution::Artifact`]
/// -- the weakest tier `COR-EXEC-001` names, and the only one a bare set of edits
/// justifies without a caller adding something stronger.
fn Artifact_Tier_Of(change: &ChangeSet) -> ReadWriteSet
{
    let entries: Vec<String> = change.Touched().into_iter().map(str::to_owned).collect();

    return ReadWriteSet::Declared(ReadWriteResolution::Artifact, entries);
}

/// A digest of the description and every edit, each part length-framed so that one edit's
/// boundary cannot be mistaken for another's.
fn Identity_Of(description: &str, change: &ChangeSet) -> CorrectionId
{
    use nomos_model::Digest_Of_Parts;

    let mut parts: Vec<&[u8]> = vec![description.as_bytes()];

    for edit in change.Edits()
    {
        parts.push(edit.Path().as_bytes());
        parts.push(edit.Before().unwrap_or_default().as_bytes());
        parts.push(edit.After().unwrap_or_default().as_bytes());
    }

    return CorrectionId::From_Digest(Digest_Of_Parts(&parts));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Edit;

    #[test]
    fn Test_New_Should_Give_Identical_Candidates_The_Same_Identity()
    {
        let change = Change_Setting_A_To("x");

        let one = CorrectionCandidate::New("fix a", change.clone(), CorrectionClass::Mechanical, vec![]);
        let other = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, vec![]);

        assert_eq!(one.Id(), other.Id());
    }

    #[test]
    fn Test_Id_Should_Return_The_Same_Value_Every_Time_Its_Called()
    {
        let change = Change_Setting_A_To("x");
        let candidate = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, vec![]);

        assert_eq!(candidate.Id(), candidate.Id());
    }

    #[test]
    fn Test_A_Different_Description_Should_Change_The_Identity()
    {
        let change = Change_Setting_A_To("x");

        let one = CorrectionCandidate::New("fix a", change.clone(), CorrectionClass::Mechanical, vec![]);
        let other = CorrectionCandidate::New("fix a differently", change, CorrectionClass::Mechanical, vec![]);

        assert_ne!(one.Id(), other.Id());
    }

    #[test]
    fn Test_A_Different_Change_Should_Change_The_Identity()
    {
        let one = CorrectionCandidate::New("fix a", Change_Setting_A_To("x"), CorrectionClass::Mechanical, vec![]);

        let other = CorrectionCandidate::New("fix a", Change_Setting_A_To("y"), CorrectionClass::Mechanical, vec![]);

        assert_ne!(one.Id(), other.Id());
    }

    #[test]
    fn Test_Class_Should_Not_Affect_The_Identity()
    {
        let change = Change_Setting_A_To("x");

        let mechanical = CorrectionCandidate::New("fix a", change.clone(), CorrectionClass::Mechanical, vec![]);
        let agent = CorrectionCandidate::New(
            "fix a",
            change,
            CorrectionClass::Agent,
            vec![CandidateLabel::AgentProposed, CandidateLabel::Speculative],
        );

        assert_eq!(
            mechanical.Id(),
            agent.Id(),
            "class and labels are declared metadata, not part of what identifies a correction"
        );
    }

    #[test]
    fn Test_Class_And_Labels_Are_Carried_Rather_Than_Computed()
    {
        let change = Change_Setting_A_To("x");
        let labels = vec![CandidateLabel::MechanicallySafe, CandidateLabel::BehaviorPreserving];

        let candidate = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, labels.clone());

        assert_eq!(candidate.Class(), CorrectionClass::Mechanical);
        assert_eq!(candidate.Labels(), labels.as_slice());
    }

    /// `COR-EXEC-001`'s floor: every candidate carries a real, declared read/write set
    /// from the moment it is constructed, at the one tier a bare `ChangeSet` justifies --
    /// never an empty list a caller has to know to fill in.
    #[test]
    fn Test_New_Should_Seed_An_Artifact_Tier_Read_Write_Set_From_The_Change()
    {
        let edit_a = Edit::New("a.rs", None, Some("x".to_owned()));
        let edit_b = Edit::New("b.rs", Some("y".to_owned()), None);
        let change = ChangeSet::Empty().With(edit_a).With(edit_b);

        let candidate = CorrectionCandidate::New("fix a and b", change, CorrectionClass::Mechanical, vec![]);

        assert_eq!(candidate.Read_Write().len(), 1, "{:?}", candidate.Read_Write());
        let seeded = candidate.Read_Write().first().expect("asserted len 1 above");
        assert_eq!(seeded.Resolution(), ReadWriteResolution::Artifact);
        assert_eq!(seeded.Entries(), ["a.rs", "b.rs"]);
        assert!(seeded.Derived_Provenance().is_none(), "the seeded floor is declared, not derived");
    }

    /// A caller with a stronger declared claim adds it beside the artifact-tier floor,
    /// which stays -- a `Symbol`-tier claim about one edit is not a claim about every edit
    /// in the change.
    #[test]
    fn Test_With_Read_Write_Should_Add_A_Declared_Set_Beside_The_Seeded_Floor()
    {
        let change = Change_Setting_A_To("x");
        let stronger = ReadWriteSet::Declared(ReadWriteResolution::Symbol, vec!["nomos_corrections::Edit".to_owned()]);

        let candidate = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, vec![]).With_Read_Write(stronger.clone());

        let [floor, added] = candidate.Read_Write()
        else
        {
            panic!("asserted the seeded floor plus one added set: {:?}", candidate.Read_Write());
        };
        assert_eq!(floor.Resolution(), ReadWriteResolution::Artifact, "the seeded floor is not replaced");
        assert_eq!(*added, stronger);
    }

    /// A derived set carries the provenance `COR-EXEC-001` requires of one, unchanged by
    /// passing through `CorrectionCandidate`.
    #[test]
    fn Test_With_Read_Write_Should_Carry_A_Derived_Sets_Own_Provenance()
    {
        use crate::DerivedProvenance;
        use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

        let change = Change_Setting_A_To("x");
        let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
        let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), guarantee, "high", "invalidated when the file's own syntax fact is recomputed");
        let derived = ReadWriteSet::Derived(ReadWriteResolution::Symbol, vec!["nomos_corrections::Edit".to_owned()], provenance);

        let candidate = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, vec![]).With_Read_Write(derived);

        let carried = candidate.Read_Write().last().expect("With_Read_Write appended one");
        let carried_provenance = carried.Derived_Provenance().expect("a derived set carries its provenance through unchanged");
        assert_eq!(carried_provenance.Provider(), &ProviderId::New("nomos-lang-rust"));
        assert_eq!(carried_provenance.Confidence(), "high");
    }

    /// A read is a claim the change cannot make on its own, so a fresh candidate declares
    /// none -- the read half of `COR-EXEC-003`'s comparison is empty until a caller says
    /// otherwise, never guessed from the edits.
    #[test]
    fn Test_New_Should_Seed_No_Read_Only_Sets()
    {
        let candidate = CorrectionCandidate::New("fix a", Change_Setting_A_To("x"), CorrectionClass::Mechanical, vec![]);

        assert!(candidate.Reads().is_empty());
    }

    #[test]
    fn Test_With_Read_Should_Add_A_Read_Only_Set_Beside_The_Read_Write_Floor()
    {
        let read = ReadWriteSet::Declared(ReadWriteResolution::Artifact, vec!["Cargo.toml".to_owned()]);

        let candidate = CorrectionCandidate::New("fix a", Change_Setting_A_To("x"), CorrectionClass::Mechanical, vec![]).With_Read(read.clone());

        assert_eq!(candidate.Reads(), [read]);
        assert_eq!(candidate.Read_Write().len(), 1, "a read-only set is not a read/write set: {:?}", candidate.Read_Write());
    }

    #[test]
    fn Test_A_Declared_Read_Should_Not_Change_The_Identity()
    {
        let change = Change_Setting_A_To("x");
        let read = ReadWriteSet::Declared(ReadWriteResolution::Artifact, vec!["Cargo.toml".to_owned()]);

        let bare = CorrectionCandidate::New("fix a", change.clone(), CorrectionClass::Mechanical, vec![]);
        let reading = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, vec![]).With_Read(read);

        assert_eq!(bare.Id(), reading.Id(), "a read is declared metadata, not part of what identifies a correction");
    }

    /// The one-edit changeset every test above builds: `a.rs` set to `content`, with no
    /// prior content declared. Encapsulated once so the same `Edit::New` /
    /// `ChangeSet::Empty().With` pairing is not repeated at every call site.
    fn Change_Setting_A_To(content: &str) -> ChangeSet
    {
        let edit = Edit::New("a.rs", None, Some(content.to_owned()));
        return ChangeSet::Empty().With(edit);
    }
}
