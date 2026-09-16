//! Which fixed-width integer holds a stated `[min, max]` range.
//!
//! `Narrowest_That_Holds` and the arithmetic behind it, ported from `rust_scalar_range.go`'s
//! `narrowest_ThatHolds`/`can_Hold_Range`/`is_cheaper`. Grouped apart from [`super`] for the
//! same reason [`super::bound`] is: this answers one question — what type would hold the
//! range — and the rules that ask it are two callers, not a second copy of the arithmetic.

/// The widest integer this rule reasons about — bounds are parsed as `i64`, so a 64-bit
/// candidate holds any value they can carry.
pub(super) const MAX_SCALAR_BITS: u32 = 64;

/// `rust_remedies` in `rust_scalar_range.go`: the fixed-width candidates a remedy may be
/// drawn from. `(spelling, bits, signed)`.
const RUST_REMEDIES: &[(&str, u32, bool)] =
    &[("u8", 8, false), ("i8", 8, true), ("u16", 16, false), ("i16", 16, true), ("u32", 32, false), ("i32", 32, true), ("u64", 64, false), ("i64", 64, true)];

/// Picks the cheapest candidate in [`RUST_REMEDIES`] that holds `[min, max]` — unsigned
/// preferred at equal width, ported from `Narrowest_That_Holds`/`is_cheaper`.
pub(super) fn Narrowest_That_Holds(min: i64, max: i64) -> Option<(&'static str, u32, bool)>
{
    let mut best: Option<(&'static str, u32, bool)> = None;

    for &(spelling, bits, signed) in RUST_REMEDIES
    {
        if !Can_Hold_Range(bits, Signedness::From_Bool(signed), min, max)
        {
            continue;
        }

        best = match best
        {
            None => Some((spelling, bits, signed)),
            Some((_, best_bits, best_signed))
                if Is_Cheaper(bits, Signedness::From_Bool(signed), best_bits, Signedness::From_Bool(best_signed)) =>
            {
                Some((spelling, bits, signed))
            }
            keep => keep,
        };
    }

    return best;
}

/// Whether an integer representation can hold values below zero — named so `Can_Hold_Range`
/// takes no bare `bool`, which a call site could pass in the wrong position with nothing to
/// catch it.
enum Signedness
{
    Signed,
    Unsigned,
}

impl Signedness
{
    fn From_Bool(signed: bool) -> Self
    {
        return if signed { Signedness::Signed } else { Signedness::Unsigned };
    }

    fn Is_Signed(self) -> bool
    {
        return matches!(self, Signedness::Signed);
    }
}

/// Ported from `can_Hold_Range`, using `i128` intermediates so the ceiling computation
/// never overflows for any width up to [`MAX_SCALAR_BITS`].
fn Can_Hold_Range(bits: u32, signedness: Signedness, min: i64, max: i64) -> bool
{
    if bits == 0 || bits > MAX_SCALAR_BITS
    {
        return false;
    }

    return match signedness
    {
        Signedness::Unsigned => Is_Fitting_Unsigned(bits, min, max),
        Signedness::Signed => Is_Fitting_Signed(bits, min, max),
    };
}

fn Is_Fitting_Unsigned(bits: u32, min: i64, max: i64) -> bool
{
    if min < 0
    {
        return false;
    }
    if bits == MAX_SCALAR_BITS
    {
        return true;
    }
    let ceiling: i128 = (1i128 << bits).saturating_sub(1);
    return i128::from(max) <= ceiling;
}

fn Is_Fitting_Signed(bits: u32, min: i64, max: i64) -> bool
{
    if bits == MAX_SCALAR_BITS
    {
        return true;
    }
    let ceiling: i128 = (1i128 << bits.saturating_sub(1)).saturating_sub(1);
    return i128::from(min) >= ceiling.saturating_neg().saturating_sub(1) && i128::from(max) <= ceiling;
}

/// A candidate beats the current best when it is narrower, or equally wide but unsigned
/// where the current best is signed.
fn Is_Cheaper(bits: u32, signedness: Signedness, best_bits: u32, best_signedness: Signedness) -> bool
{
    return bits < best_bits || (bits == best_bits && !signedness.Is_Signed() && best_signedness.Is_Signed());
}
