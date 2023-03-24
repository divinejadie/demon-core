#![no_std]

mod repr;
mod str;
mod vec;

#[cfg(target_pointer_width = "64")]
const INLINE_SIZE: usize = 23;
#[cfg(target_pointer_width = "32")]
const INLINE_SIZE: usize = 11;

pub use crate::str::Str;
pub use crate::vec::Vector;
