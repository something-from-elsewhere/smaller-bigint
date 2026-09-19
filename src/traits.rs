//! A series of traits which exist to support the library, patching what I
//! consider to be holes in `std`. Implementing these traits in your own types
//! can enable them to be used by `BigUInt`s or `BigInt`s as a backing type

use std::{
    fmt::Debug,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub},
};

/// Describes a fixed-width signed integer
///
/// Using this trait with an unsigned backing representation may cause
/// unspecified behavior from this library
///
/// Should use two's complement semantics
pub trait FixedWidthInt:
    Debug
    + Sized
    + 'static
    + Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + Ord
    + From<i8>
    + TryInto<i8>
{
    /// The unsigned twin of this integer. If set to a differently-sized
    /// integer, may cause unspecified behavior
    type Unsigned: FixedWidthUInt;
    /// The size of the integer type in bits. May cause unspecified
    /// behavior if not equal to the total width including sign bit
    ///
    /// `BITS` must be `>= 8`, and `BITS % 8` must evaluate to `0`.
    /// Panics when used otherwise
    const BITS: u32;
    /// The maximum value of the signed integer. May cause unspecified
    /// behavior if not accurately represented
    const MAX: Self;
    /// The minimum value of the signed integer. May cause unspecified
    /// behavior if not accurately represented
    const MIN: Self;
    /// The maximum value of the signed integer's unsigned twin. May
    /// cause unspecified behavior if not accurately represented
    const MAX_UNSIGNED: Self::Unsigned;
    /// Works identically to standard library [`i8::overflowing_add`], with
    /// the bool returning true if overflow has occurred
    fn overflowing_add(self, rhs: Self) -> (Self, bool);
    /// Works identically to standard library [`i8::overflowing_sub`], with
    /// the bool returning true if overflow has occurred
    fn overflowing_sub(self, rhs: Self) -> (Self, bool);
    /// A multiplication method which should return the low and high parts in that order
    fn safe_multiply(self, rhs: Self) -> (Self::Unsigned, Self);
}

/// Describes a fixed-width unsigned integer
///
/// Using this trait with a signed backing representation may cause
/// unspecified behavior from this library
pub trait FixedWidthUInt:
    Debug
    + Sized
    + 'static
    + Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + Ord
    + From<u8>
    + TryInto<u8>
{
    /// The size of the integer type in bits. May cause unspecified
    /// behavior if not equal to the total width
    ///
    /// `BITS` must be `>= 8`, and `BITS % 8` must evaluate to `0`.
    /// Panics when used otherwise
    const BITS: u32;
    /// The maximum value of the unsigned integer. May cause unspecified
    /// behavior if not accurately represented
    const MAX: Self;
    /// Works identically to standard library [`u8::overflowing_add`], with
    /// the bool returning true if overflow has occurred
    fn overflowing_add(self, rhs: Self) -> (Self, bool);
    /// Works identically to standard library [`u8::overflowing_sub`], with
    /// the bool returning true if overflow has occurred
    fn overflowing_sub(self, rhs: Self) -> (Self, bool);
    /// A multiplication method which should return the low and high parts in that order
    fn safe_multiply(self, rhs: Self) -> (Self, Self);
}

macro_rules! impl_fw_sint {
    ($signed:ty, $unsigned:ty, $doublewide:ty) => {
        impl FixedWidthInt for $signed {
            type Unsigned = $unsigned;
            const MAX_UNSIGNED: $unsigned = <$unsigned>::MAX;
            const BITS: u32 = <$signed>::BITS;
            const MAX: $signed = <$signed>::MAX;
            const MIN: $signed = <$signed>::MIN;

            fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                self.overflowing_add(rhs)
            }

            fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                self.overflowing_sub(rhs)
            }

            #[expect(
                clippy::cast_sign_loss,
                reason = "Intentionally reinterpret two's-complement bits as unsigned"
            )]
            #[expect(
                clippy::cast_possible_truncation,
                reason = "Intentionally downcast and truncate"
            )]
            fn safe_multiply(self, rhs: Self) -> (Self::Unsigned, Self) {
                let lhs: $doublewide = self.into();
                let rhs: $doublewide = rhs.into();
                let result = lhs * rhs;
                (result as Self::Unsigned, (result >> Self::BITS) as Self)
            }
        }
    };
}

macro_rules! impl_fw_uint {
    ($unsigned:ty, $doublewide:ty) => {
        impl FixedWidthUInt for $unsigned {
            const BITS: u32 = <$unsigned>::BITS;
            const MAX: $unsigned = <$unsigned>::MAX;

            fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                self.overflowing_add(rhs)
            }

            fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                self.overflowing_sub(rhs)
            }

            #[expect(
                clippy::cast_possible_truncation,
                reason = "Intentionally downcast and truncate"
            )]
            fn safe_multiply(self, rhs: Self) -> (Self, Self) {
                let lhs: $doublewide = self.into();
                let rhs: $doublewide = rhs.into();
                let result = lhs * rhs;
                (result as Self, (result >> Self::BITS) as Self)
            }
        }
    };
}

impl_fw_sint!(i8, u8, i16);
impl_fw_sint!(i16, u16, i32);
impl_fw_sint!(i32, u32, i64);
impl_fw_sint!(i64, u64, i128);

impl_fw_uint!(u8, u16);
impl_fw_uint!(u16, u32);
impl_fw_uint!(u32, u64);
impl_fw_uint!(u64, u128);

impl FixedWidthInt for i128 {
    type Unsigned = u128;
    const MAX_UNSIGNED: Self::Unsigned = u128::MAX;
    const BITS: u32 = i128::BITS;
    const MAX: Self = i128::MAX;
    const MIN: Self = i128::MIN;

    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        self.overflowing_sub(rhs)
    }

    #[expect(
        clippy::cast_sign_loss,
        reason = "Intentionally reinterpret as unsigned"
    )]
    #[expect(clippy::cast_possible_truncation, reason = "Intentionally truncate")]
    #[expect(
        clippy::cast_possible_wrap,
        reason = "Intentionally reinterpret as signed"
    )]
    fn safe_multiply(self, rhs: Self) -> (Self::Unsigned, Self) {
        let (lhs_neg, rhs_neg) = (self < 0, rhs < 0);
        let lhs = self as u128;
        let rhs = rhs as u128;
        let lhs0: u128 = (lhs as u64).into();
        let lhs1: u128 = lhs >> 64;
        let rhs0: u128 = (rhs as u64).into();
        let rhs1: u128 = rhs >> 64;
        let prod0 = lhs0 * rhs0;
        let prod1 = lhs0 * rhs1;
        let prod2 = lhs1 * rhs0;
        let prod3 = lhs1 * rhs1;

        let (low, carry0) = prod0.overflowing_add(prod1 << 64);
        let (low, carry1) = low.overflowing_add(prod2 << 64);

        let mut high =
            prod3 + (prod1 >> 64) + (prod2 >> 64) + u128::from(carry0) + u128::from(carry1);

        if lhs_neg {
            high = high.wrapping_sub(rhs);
        }
        if rhs_neg {
            high = high.wrapping_sub(lhs);
        }

        (low, high as i128)
    }
}

impl FixedWidthUInt for u128 {
    const BITS: u32 = u128::BITS;
    const MAX: Self = u128::MAX;

    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.overflowing_add(rhs)
    }

    fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        self.overflowing_sub(rhs)
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "Intentionally downcast and truncate"
    )]
    fn safe_multiply(self, rhs: Self) -> (Self, Self) {
        let lhs0: u128 = (self as u64).into();
        let lhs1: u128 = self >> 64;
        let rhs0: u128 = (rhs as u64).into();
        let rhs1: u128 = rhs >> 64;
        let prod0 = lhs0 * rhs0;
        let prod1 = lhs0 * rhs1;
        let prod2 = lhs1 * rhs0;
        let prod3 = lhs1 * rhs1;

        let (low, carry0) = prod0.overflowing_add(prod1 << 64);
        let (low, carry1) = low.overflowing_add(prod2 << 64);

        let high = prod3 + (prod1 >> 64) + (prod2 >> 64) + u128::from(carry0) + u128::from(carry1);

        (low, high)
    }
}
