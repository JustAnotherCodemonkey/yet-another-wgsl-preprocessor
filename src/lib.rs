#![warn(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod macros;
pub mod parsing;
pub mod utils;
