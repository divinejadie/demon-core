extern crate alloc;

use core::{
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr,
    slice::SliceIndex,
};

use alloc::fmt;

use crate::repr::Repr;

#[repr(transparent)]
pub struct Vector<T>(Repr<T>);

impl<T> Vector<T> {
    #[inline]
    pub fn new() -> Self {
        Self(Repr::<T>::new_inline(&[]))
    }

    #[inline]
    pub fn new_heap() -> Self {
        Self(Repr::<T>::new_heap())
    }

    #[inline]
    pub fn from_heap(data: &[T]) -> Self {
        Self(Repr::<T>::from_heap(data))
    }

    #[inline]
    pub unsafe fn as_bytes(&self) -> &[u8] {
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
    pub fn get(&mut self, idx: usize) -> Option<&T> {
        self.0.get(idx)
    }

    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.0.get_mut(idx)
    }

    #[inline]
    pub fn push(&mut self, element: T) {
        self.0.push(element);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    // #[inline]
    // pub fn insert(&mut self, element: T, idx: usize) {}

    // #[inline]
    // pub fn remove(&mut self, idx: usize) -> Option<T> {}

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self.0.as_slice_mut()
    }

    #[inline]
    pub fn is_inline(&self) -> bool {
        self.0.is_inline()
    }

    #[inline]
    pub fn extend_from_slice(&mut self, data: &[T]) {
        self.0.extend_from_slice(data)
    }
}

impl<T: fmt::Debug> fmt::Debug for Vector<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

pub struct IntoIter<T> {
    pub(super) phantom: PhantomData<T>,
    pub(super) ptr: *const T,
    pub(super) end: *const T, // If T is a ZST, this is actually ptr+len. This encoding is picked so that
                              // ptr == end is a quick test for the Iterator being empty, that works
                              // for both ZST and non-ZST.
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.ptr == self.end {
            None
        } else if mem::size_of::<T>() == 0 {
            self.end = self.end.wrapping_sub(1);

            // Make up a value of this ZST.
            Some(unsafe { mem::zeroed() })
        } else {
            let old = self.ptr;
            self.ptr = unsafe { self.ptr.add(1) };

            Some(unsafe { ptr::read(old) })
        }
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let mut me = ManuallyDrop::new(self);
            let begin = me.as_mut_ptr();
            let end = if mem::size_of::<T>() == 0 {
                begin.wrapping_add(me.len())
            } else {
                begin.add(me.len()) as *const T
            };
            IntoIter {
                phantom: PhantomData,
                ptr: begin,
                end,
            }
        }
    }
}

impl<T> FromIterator<T> for Vector<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vector::new();
        vec.extend(iter);
        vec
    }
}

impl<T, U> PartialEq<&[U]> for Vector<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &&[U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for Vector<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for Vector<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T> DerefMut for Vector<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        self.as_slice_mut()
    }
}

impl<T> AsRef<[T]> for Vector<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> Extend<T> for Vector<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        Extend::extend(&mut self.0, iter)
    }
}

impl<'a, T: Copy> Extend<&'a T> for Vector<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}
