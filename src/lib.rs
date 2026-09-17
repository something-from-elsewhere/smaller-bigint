#![warn(clippy::pedantic)]
#![allow(dead_code)]
//! # Smaller Bigint
//! `smaller_bigint` is a set of storage-optimized arbitrary-width integers,
//! each backed by a user-chosen or otherwise inferred value type

mod bigbitset;
mod biguint;
pub mod traits;

pub use bigbitset::BigBitSet;
pub use biguint::BigUInt;

#[cfg(test)]
mod tests {}
