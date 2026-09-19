use std::{
    any::Any,
    cmp::max,
    ops::{Add, Mul, Sub},
    str::FromStr,
};

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
    pub(crate) backing: Vec<S>,
}

#[derive(Debug)]
pub enum ParseBigUIntError {
    InvalidDigit,
    NegativeNumber,
}

impl<S: FixedWidthUInt> BigUInt<S> {
    pub fn new(init_val: S) -> Self {
        Self {
            backing: vec![init_val],
        }
    }

    ///
    /// # Panics
    /// This panics if an invalid radix (not between 2 and 36 inclusive) is encountered,
    /// or if the supplied backing type uses a nonpositive or indivisible-by-8 value for
    /// `BITS`
    ///
    /// # Errors
    /// If a negative number or out-of-radix digit is encountered
    pub fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseBigUIntError> {
        assert!(
            (2..37).contains(&radix),
            "Radix must be an integer from 2 to 36 inclusive!"
        );
        assert!(
            S::BITS >= 8 && S::BITS % 8 == 0,
            "FixedWidthUInt::BITS must be a positive multiple of 8!"
        );
        if s.starts_with('-') {
            return Err(ParseBigUIntError::NegativeNumber);
        }
        let mut result: BigUInt<S> = 0_u8.into();
        let radix_u8: u8 = radix.try_into().ok().unwrap();
        for ch in s.chars() {
            let digit: u8 = ch
                .to_digit(radix)
                .ok_or(ParseBigUIntError::InvalidDigit)?
                .try_into()
                .ok()
                .unwrap();
            result = result * radix_u8 + digit;
        }

        Ok(result)
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

    fn carrying_add(lhs: S, rhs: S, carry: bool) -> (S, bool) {
        let (acc, more_carry) = lhs.overflowing_add(rhs);
        let (acc, carry) = acc.overflowing_add(u8::from(carry).into());
        (acc, carry || more_carry)
    }

    fn borrowing_sub(lhs: S, rhs: S, borrow: bool) -> (S, bool) {
        let (acc, more_borrow) = lhs.overflowing_sub(rhs);
        let (acc, borrow) = acc.overflowing_sub(u8::from(borrow).into());
        (acc, borrow || more_borrow)
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

impl<S: FixedWidthUInt> FromStr for BigUInt<S> {
    type Err = ParseBigUIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

//=========================== Addition ===================================

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

impl<S, R> Add<&BigUInt<R>> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    type Output = Self;
    fn add(self, rhs: &BigUInt<R>) -> Self::Output {
        let rhs: Self = rhs.into();
        self + rhs
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
                (Some(lhs), Some(rhs)) => BigUInt::carrying_add(*lhs, *rhs, carry),
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

//========================== Subtraction =================================

impl<S, R> Sub<R> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    type Output = Self;

    fn sub(self, rhs: R) -> Self::Output {
        let rhs: Self = rhs.into();
        self - rhs
    }
}

impl<S, R> Sub<&BigUInt<R>> for BigUInt<S>
where
    S: FixedWidthUInt,
    R: FixedWidthUInt,
{
    type Output = Self;
    fn sub(self, rhs: &BigUInt<R>) -> Self::Output {
        let rhs: Self = rhs.into();
        self - rhs
    }
}

impl<S: FixedWidthUInt> Sub for BigUInt<S> {
    type Output = Self;
    /// Will mirror runtime behavior for underflow, when wrapping wraps to the current
    /// logical width of the underlying representation
    ///
    /// # Examples
    /// ```
    /// use smaller_bigint::BigUInt;
    ///
    /// let a = BigUInt::<u8>::from(420_u32);
    /// let b = BigUInt::<u8>::new(67_u8);
    ///
    /// assert_eq!(a - b, 420_u32 - 67_u32);
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        assert!(
            S::BITS >= 8 && S::BITS % 8 == 0,
            "FixedWidthUInt::BITS must be a positive multiple of 8!"
        );

        let mut backing = Vec::new();
        let mut borrow = false;
        for i in 0..max(self.backing.len(), rhs.backing.len()) {
            let acc;
            (acc, borrow) = match (self.backing.get(i), rhs.backing.get(i)) {
                (Some(lhs), Some(rhs)) => BigUInt::borrowing_sub(*lhs, *rhs, borrow),
                (Some(lhs), None) => lhs.overflowing_sub(u8::from(borrow).into()),
                (None, Some(rhs)) => BigUInt::borrowing_sub(0_u8.into(), *rhs, borrow),
                (None, None) => break,
            };
            backing.push(acc);
        }

        if borrow {
            let _ = 0_u8 - u8::from(borrow);
        }

        Self { backing }
    }
}

// TODO: MULTIPLICATION
//========================= Multiplication ===============================

impl<S: FixedWidthUInt> BigUInt<S> {
    /// I make no guarantees as I don't have it in me to inspect the generated assembly, but this
    /// multiplication SHOULD be constant-time with respect to operand values for fixed operand
    /// widths. Its result width depends on the operand widths alone, and so leaks no additional
    /// magnitude info <3
    #[must_use]
    pub fn secure_mul(self, rhs: Self) -> Self {
        let mut products: Vec<Vec<S>> = Vec::new();

        let mut carry = false;
        for chunk in rhs.backing {
            let mut product: Vec<S> = vec![0_u8.into()];

            for l_chunk in &self.backing {
                let (low, high) = chunk.safe_multiply(*l_chunk);
                let last_idx = product.len() - 1;
                let mut result;
                (result, carry) = BigUInt::carrying_add(product[last_idx], low, carry);
                product[last_idx] = result;
                (result, carry) = high.overflowing_add(u8::from(carry).into());
                product.push(result);
            }
            products.push(product);
        }

        let mut result = Vec::new();
        for p in &products[0] {
            result.push(*p);
        }
        result.push(0_u8.into());
        for i in 1..products.len() {
            let mut carry = false;
            for j in 0..products[i].len() {
                (result[i + j], carry) =
                    BigUInt::carrying_add(result[i + j], products[i][j], carry);
            }
            result.push(u8::from(carry).into());
        }

        BigUInt { backing: result }
    }
}

impl<S: FixedWidthUInt, R: FixedWidthUInt> Mul<R> for BigUInt<S> {
    type Output = Self;
    fn mul(self, rhs: R) -> Self::Output {
        let rhs: BigUInt<S> = rhs.into();
        self * rhs
    }
}

impl<S: FixedWidthUInt, R: FixedWidthUInt> Mul<&BigUInt<R>> for BigUInt<S> {
    type Output = Self;
    fn mul(self, rhs: &BigUInt<R>) -> Self::Output {
        let rhs: Self = rhs.into();
        self * rhs
    }
}

impl<S: FixedWidthUInt> Mul for BigUInt<S> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self.secure_mul(rhs);

        if result.backing[result.backing.len() - 1] == 0_u8.into() {
            result.backing.pop();
            if result.backing[result.backing.len() - 1] == 0_u8.into() {
                result.backing.pop();
            }
        }

        result
    }
}
