use std::{any::Any, cmp::max, ops::Add};

use crate::traits::FixedWidthUInt;

/// An arbitrary-width unsigned integer with a selectable backing type
///
/// (Will) Support(s) all standard numeric operations, as well as widening
/// or narrowing the backing representation
#[derive(Debug)]
pub struct BigUInt<S>
where
    S: FixedWidthUInt,
{
    // Limbs are stored least-significant first
    backing: Vec<S>,
}

impl<S: FixedWidthUInt> BigUInt<S> {
    pub fn new(init_val: S) -> Self {
        Self {
            backing: vec![init_val],
        }
    }

    fn pack_into_vec<R: FixedWidthUInt>(mut value: R) -> Vec<S> {
        if let Some(value) = (&value as &dyn Any).downcast_ref::<S>() {
            return vec![*value];
        }
        let mut result = Vec::new();
        let mask: R = u8::MAX.into();

        let mut word_completion: u32 = 0;
        let mut word: S = 0_u8.into();
        let byte_count = R::BITS / 8;
        for byte_idx in 0..byte_count {
            let tmp: u8 = (value & mask).try_into().ok().unwrap();
            let tmp: S = tmp.into();
            word = word | (tmp << word_completion);
            if byte_idx + 1 < byte_count {
                value = value >> 8;
            }
            word_completion += 8;
            if word_completion == S::BITS {
                result.push(word);
                word_completion = 0;
                word = 0_u8.into();
            }
            if value == 0_u8.into() {
                break;
            }
        }
        if word != 0_u8.into() {
            result.push(word);
        }

        if result.is_empty() {
            vec![0_u8.into()]
        } else {
            result
        }
    }

    fn overflowing_add_with_carry(lhs: S, rhs: S, carry: bool) -> (S, bool) {
        let (acc, more_carry) = lhs.overflowing_add(rhs);
        let (acc, carry) = acc.overflowing_add(u8::from(carry).into());
        (acc, carry || more_carry)
    }
}

impl<S, R> From<R> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    /// Converts from an arbitrarily wide unsigned integer to a `BigUInt`
    /// of the supplied type
    ///
    /// # Panics
    /// This function panics when the conversion target `S` has a malformed
    /// `BITS` field
    ///
    /// # Examples
    /// ```
    /// use smaller_bigint::BigUInt;
    ///
    /// let big: BigUInt<u8> = 679_u32.into();
    /// let small: BigUInt<u16> = 52_u8.into();
    ///
    /// assert_eq!(big, 679_u32);
    /// assert_eq!(small, 52_u32);
    /// ```
    fn from(value: R) -> Self {
        assert!(
            S::BITS >= 8 && S::BITS % 8 == 0,
            "FixedWidthUInt::BITS must be a positive multiple of 8!"
        );
        Self {
            backing: BigUInt::pack_into_vec(value),
        }
    }
}

impl<S, R> PartialEq<R> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    fn eq(&self, other: &R) -> bool {
        let other: BigUInt<S> = (*other).into();
        self == &other
    }
}
impl<S: FixedWidthUInt> Eq for BigUInt<S> {}

impl<S, R> PartialEq<BigUInt<R>> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    fn eq(&self, other: &BigUInt<R>) -> bool {
        let other: BigUInt<S> = other.into();
        self.backing == other.backing
    }
}

impl<S, R> From<&BigUInt<R>> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    fn from(value: &BigUInt<R>) -> Self {
        assert!(
            S::BITS >= 8 && S::BITS % 8 == 0,
            "FixedWidthUInt::BITS must be a positive multiple of 8!"
        );
        if let Some(val) = (&value.backing as &dyn Any).downcast_ref::<Vec<S>>() {
            return Self {
                backing: val.clone(),
            };
        }
        let mask: R = u8::MAX.into();
        let mut backing: Vec<S> = Vec::new();
        let mut word_completion: u32 = 0;
        let mut word: S = 0_u8.into();

        for &foreign_word in &value.backing {
            let mut foreign_word = foreign_word;
            let byte_count = R::BITS / 8;
            for byte_idx in 0..byte_count {
                let cronch: u8 = (foreign_word & mask).try_into().ok().unwrap();
                let cronch: S = cronch.into();
                word = word | (cronch << word_completion);
                if byte_idx + 1 < byte_count {
                    foreign_word = foreign_word >> 8;
                }
                word_completion += 8;
                if word_completion == S::BITS {
                    backing.push(word);
                    word = 0_u8.into();
                    word_completion = 0;
                }
            }
        }
        if word_completion != 0 {
            backing.push(word);
        }

        Self { backing }
    }
}

impl<S: FixedWidthUInt> Clone for BigUInt<S> {
    fn clone(&self) -> Self {
        assert!(
            S::BITS >= 8 && S::BITS % 8 == 0,
            "FixedWidthUInt::BITS must be a positive multiple of 8!"
        );
        Self {
            backing: self.backing.clone(),
        }
    }
}

// impl<S: FixedWidthUInt> From<&str> for BigUInt<S> {
//     fn from(_value: &str) -> Self {
//         assert!(
//             S::BITS >= 8 && S::BITS % 8 == 0,
//             "FixedWidthUInt::BITS must be a positive multiple of 8!"
//         );
//         Self {
//             // TODO: String conversion
//             backing: vec![S::from(0_u8)],
//         }
//     }
// }

impl<S, R> Add<R> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    type Output = Self;
    fn add(self, rhs: R) -> Self::Output {
        let rhs: BigUInt<S> = rhs.into();
        self.add(rhs)
    }
}

impl<S: FixedWidthUInt> Add for BigUInt<S> {
    type Output = Self;

    /// Will grow the returned value as needed to fit the resultant value
    ///
    /// # Examples
    /// ```
    /// use smaller_bigint::BigUInt;
    ///
    /// let full = BigUInt::<u8>::new(255);
    /// let over = BigUInt::<u8>::new(1);
    ///
    /// assert_eq!(full + over, 256_u32);
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        assert!(
            S::BITS >= 8 && S::BITS % 8 == 0,
            "FixedWidthUInt::BITS must be a positive multiple of 8!"
        );

        let mut backing = Vec::new();
        let mut carry = false;
        for i in 0..max(self.backing.len(), rhs.backing.len()) {
            let acc;
            (acc, carry) = match (self.backing.get(i), rhs.backing.get(i)) {
                (Some(lhs), Some(rhs)) => BigUInt::overflowing_add_with_carry(*lhs, *rhs, carry),
                (Some(lhs), None) => lhs.overflowing_add(u8::from(carry).into()),
                (None, Some(rhs)) => rhs.overflowing_add(u8::from(carry).into()),
                (None, None) => break,
            };
            backing.push(acc);
        }
        if carry {
            backing.push(1_u8.into());
        }
        Self { backing }
    }
}
