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
pub trait FixedWidthSInt:
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
}

macro_rules! impl_fw_sint {
    ($signed:ty, $unsigned:ty) => {
        impl FixedWidthSInt for $signed {
            type Unsigned = $unsigned;
            const MAX_UNSIGNED: $unsigned = <$unsigned>::MAX;
            const BITS: u32 = <$signed>::BITS;
            const MAX: $signed = <$signed>::MAX;
            const MIN: $signed = <$signed>::MIN;

            fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                self.overflowing_add(rhs)
            }
        }
    };
}

macro_rules! impl_fw_uint {
    ($unsigned:ty) => {
        impl FixedWidthUInt for $unsigned {
            const BITS: u32 = <$unsigned>::BITS;
            const MAX: $unsigned = <$unsigned>::MAX;

            fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                self.overflowing_add(rhs)
            }
        }
    };
}

impl_fw_sint!(i8, u8);
impl_fw_sint!(i16, u16);
impl_fw_sint!(i32, u32);
impl_fw_sint!(i64, u64);
impl_fw_sint!(i128, u128);

impl_fw_uint!(u8);
impl_fw_uint!(u16);
impl_fw_uint!(u32);
impl_fw_uint!(u64);
impl_fw_uint!(u128);
