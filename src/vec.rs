extern crate alloc;

use core::{
    marker::PhantomData,
    mem,
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

    #[inline]
    pub fn insert(&mut self, idx: usize, element: T) {
        self.0.insert(idx, element);
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) -> T {
        self.0.remove(idx)
    }

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
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.as_slice_mut().iter_mut()
    }

    #[inline]
    pub fn drain(&mut self) -> Drain<T> {
        unsafe {
            let start = self.0.as_ptr_mut();
            let end = start.add(self.len());

            Drain {
                phantom: PhantomData,
                start,
                end,
            }
        }
    }
}

impl<T: Clone> Vector<T> {
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

impl<T: Clone> Clone for Vector<T> {
    fn clone(&self) -> Self {
        let mut vec = Vector::<T>::new();
        vec.extend_from_slice(&self);
        vec
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

unsafe impl<T: Send> Send for Vector<T> {}
unsafe impl<T: Sync> Sync for Vector<T> {}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

unsafe impl<'a, T: Send> Send for Drain<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Drain<'a, T> {}

impl<T: Clone> From<&[T]> for Vector<T> {
    fn from(value: &[T]) -> Self {
        let mut vec = Vector::<T>::new();
        vec.extend_from_slice(value);
        vec
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T> {
    fn from(value: [T; N]) -> Self {
        let mut vec = Vector::<T>::new();
        vec.extend(value);
        vec
    }
}
impl<T: Clone> From<&mut [T]> for Vector<T> {
    fn from(value: &mut [T]) -> Self {
        let mut vec = Vector::new();
        vec.extend_from_slice(value);
        vec
    }
}

impl<T> From<alloc::vec::Vec<T>> for Vector<T> {
    fn from(mut value: alloc::vec::Vec<T>) -> Self {
        let ptr = value.as_mut_ptr();
        let mut vec = Vector::new_heap();

        vec.0.get_heap_mut().ptr = ptr::NonNull::new(ptr).unwrap();
        vec.0.set_len(value.len());
        vec.0.get_heap_mut().capacity = value.capacity();

        mem::forget(value);

        vec
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end -= 1;
                Some(ptr::read(self.vec.0.as_ptr_mut().add(self.end)))
            }
        }
    }
}

pub struct IntoIter<T> {
    vec: Vector<T>,
    start: usize,
    end: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            let next = self.start;
            self.start += 1;
            Some(unsafe { ptr::read(self.vec.as_mut_ptr().add(next)) })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len = if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.vec.len()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            end: self.len(),
            vec: self,
            start: 0,
        }
    }
}

pub struct Drain<'a, T: 'a> {
    phantom: PhantomData<&'a mut Vector<T>>,
    start: *const T,
    end: *const T,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.end = self.end.wrapping_sub(1);
                    Some(mem::zeroed())
                } else {
                    let old_ptr = self.start;
                    self.start = self.start.add(1);
                    Some(ptr::read(old_ptr))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len =
            (self.end as usize - self.start as usize) / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.wrapping_sub(1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for Drain<'a, T> {}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in self {}
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

impl<T, U, const N: usize> PartialEq<&[U; N]> for Vector<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &&[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for Vector<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}
impl<T, U> PartialEq<Vector<U>> for Vector<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Vector<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T: Eq> Eq for Vector<T> {}

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
