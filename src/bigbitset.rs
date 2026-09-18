use std::{error::Error, fmt::Display};

use crate::BigUInt;

#[derive(Debug)]
pub struct BigBitSet {
    backing: BigUInt<u8>,
    len: usize,
}

#[derive(Debug)]
pub struct BoundsError {
    index: usize,
    len: usize,
}

impl BigBitSet {
    #[must_use]
    pub fn new(flags: &[bool]) -> Self {
        let mut result: Vec<u8> = Vec::new();
        let mut word: u8 = 0;
        for (i, flag) in flags.iter().enumerate() {
            word |= u8::from(*flag) << (i % 8);
            if i % 8 == 7 {
                result.push(word);
                word = 0;
            }
        }
        if word != 0 {
            result.push(word);
        }

        Self {
            backing: BigUInt { backing: result },
            len: flags.len(),
        }
    }

    #[must_use]
    pub fn get(&self, i: usize) -> Option<bool> {
        if i >= self.len {
            return None;
        }
        let mask: u8 = 0x01 << (i % 8);
        Some(self.backing.backing[i / 8] & mask != 0_u8)
    }

    /// # Panics
    /// When the index provided is out of range
    pub fn set(&mut self, i: usize, val: bool) {
        assert!(
            i < self.len,
            "Index {i} out of range [0, {}) in BigBitSet::set method!",
            self.len
        );
        let mask: u8 = 0x01 << (i % 8);
        if val {
            self.backing.backing[i / 8] |= mask;
        } else {
            self.backing.backing[i / 8] &= !mask;
        }
    }

    /// # Errors
    /// Returns a `BoundsError` with the attempted index and length of the array
    pub fn try_set(&mut self, i: usize, val: bool) -> Result<(), BoundsError> {
        if i >= self.len {
            return Err(BoundsError {
                index: i,
                len: self.len,
            });
        }

        self.set(i, val);
        Ok(())
    }

    pub fn append(&mut self, val: bool) {
        let localidx = self.len % 8;
        let mask: u8 = 0x01 << (self.len % 8);
        if localidx == 0 {
            self.backing.backing.push(0_u8);
        }
        if val {
            self.backing.backing[self.len / 8] |= mask;
        } else {
            self.backing.backing[self.len / 8] &= !mask;
        }
        self.len += 1;
    }

    pub fn resize(&mut self, i: usize) {
        self.backing.backing.resize(i.div_ceil(8), 0);

        self.len = i;
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl BoundsError {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Display for BoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Passed index must be below {}, but {} was found!",
            self.len, self.index
        )
    }
}

impl Error for BoundsError {}
