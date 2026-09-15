//! The reordering itself: how one is produced, that it is one, and how two are compared.
//!
//! The test that a permutation loses nothing is the only one in this suite that needs no
//! corpus — it builds five hundred synthetic members, because the claim is about the walk
//! rather than about any particular tree. It lives beside the walk it checks.

use crate::arrival::{Fresh, Ingest, Workspace};

/// How many arrival orders this suite produces, and therefore how many seeds it tries. The
/// hundred permutations are a hundred distinct orders, so the count is one quantity.
pub(crate) const PERMUTATION_COUNT: u32 = 100;

/// How many synthetic members the permutation check runs over. A member count far below the
/// real corpus's, because the claim is about the walk and not about any particular tree.
const SYNTHETIC_MEMBER_COUNT: u32 = 500;

/// The fewest distinct orders a hundred seeds must produce before the test above asserts
/// anything. Half of them, which is far below what the walk produces and far above the one a
/// broken walk would.
const DISTINCT_ORDER_FLOOR: usize = 50;

/// How far the starting offset moves for each successive seed. A prime, so successive seeds
/// begin far apart in the member list rather than on adjacent entries.
const OFFSET_GROWTH: usize = 97;

/// How much the stride grows for each successive permutation index.
const STRIDE_GROWTH: usize = 7;

/// One past the largest stride the per-permutation growth produces, so that change-set
/// boundaries fall between different files whatever the corpus length.
const STRIDE_CEILING: usize = 511;

/// What one arrival order produced, and the stride it arrived under.
pub(crate) struct Taken
{
    pub(crate) bytes: Vec<u8>,
    pub(crate) members: usize,
    pub(crate) stride: usize,
}

/// One arrival order, ingested and encoded.
///
/// The stride varies with the permutation too, so change-set boundaries fall between
/// different files each time. Order-independence within one set is a much weaker property
/// than order-independence across them, and only the second one is what a checkout and an
/// editor arriving in either order actually needs.
pub(crate) fn Snapshot_Of_One_Order(members: &[(String, String)], permutation: u32) -> Taken
{
    let ordered = Permuted(members, permutation);
    let stride = 1_usize.saturating_add(
        usize::try_from(permutation)
            .unwrap_or(0)
            .saturating_mul(STRIDE_GROWTH)
            .checked_rem(STRIDE_CEILING)
            .unwrap_or(0),
    );
    let mut workspace: Workspace = Fresh();

    Ingest(&mut workspace, &ordered, stride);

    return Taken {
        bytes: workspace.Snapshot().Encode(),
        members: workspace.Snapshot().Length(),
        stride,
    };
}

/// The first order to arrive becomes the baseline every later one is compared against.
pub(crate) fn Assert_Same_As_The_First(
    baseline: &mut Option<Taken>,
    taken: Taken,
    permutation: u32,
)
{
    let Some(first) = baseline.as_ref()
    else
    {
        *baseline = Some(taken);

        return;
    };

    assert_eq!(
        taken.members, first.members,
        "permutation {permutation} produced a different number of members"
    );
    assert!(
        taken.bytes == first.bytes,
        "permutation {permutation} (stride {}) produced different bytes",
        taken.stride
    );
}

/// A stride larger than any realistic corpus, and prime, so that stepping by it visits every
/// index before repeating for the corpus lengths this runs over. The linear scan in
/// `Walk_From` makes the walk a permutation for any stride at all; the prime is what keeps it
/// from degenerating into the identity order.
const STRIDE: usize = 7_919;

/// A deterministic reordering, distinct for each `seed`.
///
/// A multiplicative step over the index. Because the step and the length are coprime for
/// the seeds used, the walk visits every element exactly once — a shuffle that dropped or
/// repeated elements would make the test compare snapshots of different corpora and pass
/// only by accident.
pub(crate) fn Permuted(members: &[(String, String)], seed: u32) -> Vec<(String, String)>
{
    let Some(offset) = usize::try_from(seed)
        .unwrap_or(0)
        .saturating_mul(OFFSET_GROWTH)
        .checked_rem(members.len())
    else
    {
        return Vec::new();
    };

    return Walk_From(members, offset);
}

/// Step by `STRIDE` from an offset, and where that lands on something already taken, scan
/// forward to the next free slot.
fn Walk_From(members: &[(String, String)], offset: usize) -> Vec<(String, String)>
{
    let len = members.len();
    let mut permuted = Vec::with_capacity(len);
    let mut taken = vec![false; len];
    let mut at = offset;
    for _ in 0..len
    {
        while taken.get(at).copied().unwrap_or(false)
        {
            at = at.saturating_add(1).checked_rem(len).unwrap_or(0);
        }
        if let (Some(member), Some(slot)) = (members.get(at), taken.get_mut(at))
        {
            permuted.push(member.clone());
            *slot = true;
        }
        at = at.saturating_add(STRIDE).checked_rem(len).unwrap_or(0);
    }

    return permuted;
}

/// The permutation itself must be a permutation, or the test above compares snapshots of
/// different corpora.
#[test]
fn Test_The_Permutation_Should_Reorder_Without_Losing_Anything()
{
    let members: Vec<(String, String)> = (0..SYNTHETIC_MEMBER_COUNT)
        .map(|index| return (format!("src/f{index}.rs"), format!("fn f{index}() {{}}")))
        .collect();

    let mut orders = std::collections::BTreeSet::new();
    for seed in 0..PERMUTATION_COUNT
    {
        let permuted = Permuted(&members, seed);
        let order = Path_Order(&permuted);

        Assert_Holds_Every_Member(&permuted, &members, seed);
        orders.insert(order);
    }

    assert!(
        orders.len() > DISTINCT_ORDER_FLOOR,
        "a hundred seeds produced only {} distinct orders; the test above would be \
         asserting the same order against itself",
        orders.len()
    );
}

/// The same members, the same count, in a different order — which is what a permutation is.
fn Assert_Holds_Every_Member(permuted: &[(String, String)], members: &[(String, String)], seed: u32)
{
    assert_eq!(permuted.len(), members.len(), "seed {seed} changed the length");

    let mut sorted = permuted.to_vec();
    sorted.sort();
    let mut expected = members.to_vec();
    expected.sort();

    assert_eq!(sorted, expected, "seed {seed} lost or repeated a member");
}

/// The order alone, which is what makes one permutation distinct from another.
fn Path_Order(permuted: &[(String, String)]) -> Vec<String>
{
    return permuted
        .iter()
        .map(|(path, _)| return path.clone())
        .collect();
}
