//! The one place a fact identity becomes a number, and the only thing that turns it back.
//!
//! A child of [`super`] rather than a crate-root module, for the reason the invalidation
//! walk and the written form beside it are: none of it is part of [`super::MemoryFactStore`]'s
//! public surface. What a caller addresses a fact by does not change — a [`FactKey`], and the
//! [`Digest128`] it hashes to — and nothing here is reachable from outside this crate.
//!
//! # Why the store's own structures do not key on the digest
//!
//! A [`Digest128`] is sixteen bytes, so every map keyed on one compares sixteen bytes per
//! probe and every edge set stores sixteen more per edge. Worse, a caller arrives holding a
//! `FactKey` rather than a digest, and [`FactKey::Digest`] is not a field read: it allocates
//! a vector per key component and hashes the lot. Paid once per materialization that is
//! nothing; paid once per *read*, on the crate's only read path, it was the largest single
//! cost of reading a fact.
//!
//! So a key is hashed once, the first time this store is shown it, and everything the store
//! keeps afterwards keys on the [`FactSlot`] that interning returned. The digest is kept
//! beside the key it was computed from and handed back on demand — the written form needs
//! it, because a slot means nothing to another process — and it is computed in exactly one
//! place, so there is no second answer to what a key's digest is.

use std::collections::HashMap;
use std::sync::Arc;

use nomos_contracts::Digest128;

use crate::FactKey;

/// A fact identity as this store addresses it: a position in the one table that holds them.
///
/// Meaningful only within the [`FactIdentities`] that minted it, and deliberately carrying
/// no way to be constructed from a number by anything else — a slot from one store read
/// against another would name a different fact, silently. What crosses a process boundary is
/// the digest, never this.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FactSlot(usize);

impl FactSlot
{
    /// The slot as an index into a structure the store keeps one entry per slot in.
    pub(crate) const fn Position(self) -> usize
    {
        return self.0;
    }
}

/// Every fact identity this store has been shown, each with its digest and its slot.
///
/// One table and one index into it, sharing the key itself rather than holding it twice:
/// there is no arrangement of these two in which they can disagree about what a slot names
/// or about which slot a key has.
pub(crate) struct FactIdentities
{
    /// The identities, in the order they were first seen. A [`FactSlot`] is a position here.
    interned: Vec<Interned>,
    /// Which slot a key already has, if it has one.
    by_key: HashMap<Arc<FactKey>, FactSlot>,
}

/// One interned identity: the key, and the digest computed from it when it was interned.
struct Interned
{
    digest: Digest128,
    key: Arc<FactKey>,
}

impl FactIdentities
{
    pub(crate) fn New() -> Self
    {
        return Self {
            interned: Vec::new(),
            by_key: HashMap::new(),
        };
    }

    /// The slot `key` already has, minting one — and hashing the key to a digest — if this
    /// is the first time the store has been shown it.
    pub(crate) fn Intern(&mut self, key: &FactKey) -> FactSlot
    {
        if let Some(known) = self.by_key.get(key)
        {
            return *known;
        }

        let shared = Arc::new(key.clone());
        let slot = FactSlot(self.interned.len());
        self.interned.push(Interned {
            digest: key.Digest(),
            key: Arc::clone(&shared),
        });
        self.by_key.insert(shared, slot);

        return slot;
    }

    /// The slot `key` has, or nothing — a read, which must not mint a slot for a key the
    /// store was never given.
    pub(crate) fn Slot_Of(&self, key: &FactKey) -> Option<FactSlot>
    {
        return self.by_key.get(key).copied();
    }

    pub(crate) fn Key_Of(&self, slot: FactSlot) -> Option<&FactKey>
    {
        return self.interned.get(slot.Position()).map(|interned| return interned.key.as_ref());
    }

    /// The digest `slot`'s key hashes to — what a caller, a file or another process
    /// identifies the fact by.
    pub(crate) fn Digest_Of(&self, slot: FactSlot) -> Option<Digest128>
    {
        return self.interned.get(slot.Position()).map(|interned| return interned.digest);
    }

    /// How many identities have been interned, which is also how many slots exist.
    pub(crate) fn Count(&self) -> usize
    {
        return self.interned.len();
    }

    /// Every identity, slot and key together, in the order they were first seen.
    pub(crate) fn Iter(&self) -> impl Iterator<Item = (FactSlot, &FactKey)>
    {
        return self
            .interned
            .iter()
            .enumerate()
            .map(|(position, interned)| return (FactSlot(position), interned.key.as_ref()));
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{GuaranteeDigest, InputDigest};
    use nomos_contracts::{
        Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, FactVariant,
        Guarantee, IncrementalGranularity, ProviderId, SubjectId,
    };

    /// The variant component of every key here.
    const VARIANT_SEED: u8 = 3;

    /// The configuration component of every key here, distinct from [`VARIANT_SEED`] so a
    /// key assembled with the wrong one is not silently the same key.
    const CONFIGURATION_SEED: u8 = 4;

    /// The subject of the second key, distinct from the first key's `1`.
    const SECOND_SUBJECT_SEED: u8 = 2;

    #[test]
    fn Test_Intern_Should_Give_One_Key_One_Slot_However_Often_It_Is_Shown()
    {
        let mut identities = FactIdentities::New();

        let first = identities.Intern(&Key_For(1));
        let again = identities.Intern(&Key_For(1));

        assert_eq!(first, again);
        assert_eq!(identities.Count(), 1);
    }

    #[test]
    fn Test_Intern_Should_Give_Two_Keys_Two_Slots()
    {
        let mut identities = FactIdentities::New();

        let first = identities.Intern(&Key_For(1));
        let second = identities.Intern(&Key_For(SECOND_SUBJECT_SEED));

        assert_ne!(first, second);
        assert_eq!(identities.Count(), 2);
    }

    /// The property the whole type exists to keep: the digest a slot answers with is the
    /// digest the key itself hashes to, so interning changed what a lookup costs and not
    /// what a fact is identified by.
    #[test]
    fn Test_Digest_Of_Should_Answer_What_The_Key_Itself_Hashes_To()
    {
        let mut identities = FactIdentities::New();
        let key = Key_For(1);

        let slot = identities.Intern(&key);

        assert_eq!(identities.Digest_Of(slot), Some(key.Digest()));
        assert_eq!(identities.Key_Of(slot), Some(&key));
    }

    #[test]
    fn Test_Slot_Of_Should_Not_Mint_A_Slot_For_A_Key_Nobody_Interned()
    {
        let identities = FactIdentities::New();

        assert_eq!(identities.Slot_Of(&Key_For(1)), None);
        assert_eq!(identities.Count(), 0);
    }

    #[test]
    fn Test_Slot_Of_Should_Find_What_Intern_Minted()
    {
        let mut identities = FactIdentities::New();
        let slot = identities.Intern(&Key_For(1));

        assert_eq!(identities.Slot_Of(&Key_For(1)), Some(slot));
    }

    #[test]
    fn Test_Iter_Should_Name_Every_Interned_Key_Against_Its_Own_Slot()
    {
        let mut identities = FactIdentities::New();
        let first = identities.Intern(&Key_For(1));
        let second = identities.Intern(&Key_For(SECOND_SUBJECT_SEED));

        let seen: Vec<(FactSlot, FactKey)> =
            identities.Iter().map(|(slot, key)| return (slot, key.clone())).collect();

        assert_eq!(seen, vec![(first, Key_For(1)), (second, Key_For(SECOND_SUBJECT_SEED))]);
    }

    #[test]
    fn Test_Position_Should_Address_A_Structure_Holding_One_Entry_Per_Slot()
    {
        let mut identities = FactIdentities::New();
        identities.Intern(&Key_For(1));
        let second = identities.Intern(&Key_For(SECOND_SUBJECT_SEED));

        assert_eq!(second.Position(), 1);
    }

    #[test]
    fn Test_Key_Of_Should_Answer_Nothing_For_A_Slot_Beyond_What_Was_Interned()
    {
        let mut identities = FactIdentities::New();
        let only = identities.Intern(&Key_For(1));
        let mut other = FactIdentities::New();
        other.Intern(&Key_For(1));
        other.Intern(&Key_For(SECOND_SUBJECT_SEED));
        let beyond = other.Slot_Of(&Key_For(SECOND_SUBJECT_SEED)).expect("just interned");

        assert!(identities.Key_Of(only).is_some());
        assert_eq!(identities.Key_Of(beyond), None);
    }

    fn Seeded_Digest(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Key_For(subject_seed: u8) -> FactKey
    {
        let guarantee = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        );

        return FactKey {
            contract: CapabilityId::New("nomos.cap.test.identities"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Seeded_Digest(subject_seed)),
            semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
            provider: ProviderId::New("nomos.provider.test"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&guarantee),
            variant: BuildVariantId::From_Digest(Seeded_Digest(VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_SEED)),
        };
    }
}
