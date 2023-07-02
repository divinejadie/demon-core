extern crate alloc;

use crate::repr::Repr;
use crate::INLINE_SIZE;
use core::ops::Deref;

#[repr(transparent)]
pub struct Str(Repr<u8>);

impl Str {
    #[inline]
    pub fn new() -> Self {
        Self(Repr::<u8>::new_inline(&[]))
    }

    pub fn from(string: &str) -> Self {
        match string.len() {
            0..=INLINE_SIZE => Self(Repr::<u8>::new_inline(string.as_bytes())),
            _ => Self(Repr::<u8>::from_heap(string.as_bytes())),
        }
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.0.push(ch as u8),
            _ => self.0.extend(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        };
    }

    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.0.extend(string.as_bytes());
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.0.pop().map(|byte| byte as char)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.set_len(0);
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.bytes()
    }

    #[inline]
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.0.bytes_mut()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.deref()
    }

    #[inline]
    pub fn is_inline(&self) -> bool {
        self.0.is_inline()
    }
}

impl PartialEq for Str {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
    #[inline]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self[..], &other[..])
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
            #[inline]
            fn ne(&self, other: &$lhs) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }
    };
}

impl_eq! { Str, str }
impl_eq! {Str, alloc::string::String }
impl_eq! { Str, &'a str }
#[cfg(not(no_global_oom_handling))]
impl_eq! { alloc::borrow::Cow< 'a,str>, Str }

impl AsRef<str> for Str {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl Deref for Str {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl core::fmt::Debug for Str {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&**self, f)
    }
}
