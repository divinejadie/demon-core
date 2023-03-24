extern crate alloc;
use alloc::alloc::{alloc, dealloc, handle_alloc_error, realloc};

use crate::INLINE_SIZE;
use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::{self, ManuallyDrop, MaybeUninit},
    ptr::{self, NonNull},
};

#[repr(C)]
pub union Repr<T> {
    pub(crate) heap: ManuallyDrop<Heap<T>>,
    pub(crate) inline: ManuallyDrop<Inline<T>>,
}

#[repr(C)]
pub struct Inline<T> {
    pub(crate) disc: Discriminant,
    pub(crate) data: [u8; INLINE_SIZE],
    pub(crate) _phantom: PhantomData<T>,
}

#[repr(C)]
pub struct Heap<T> {
    pub(crate) capacity: usize,
    pub(crate) len: usize,
    pub(crate) ptr: NonNull<T>,
}

impl<T> Drop for Repr<T> {
    fn drop(&mut self) {
        match self.is_inline() {
            true => unsafe { ManuallyDrop::drop(&mut self.inline) },
            false => {
                unsafe { ManuallyDrop::drop(&mut self.heap) };

                let self_heap = self.get_heap_mut();
                let elem_size = mem::size_of::<T>();

                if self_heap.capacity != 0 && elem_size != 0 {
                    unsafe {
                        dealloc(
                            self_heap.ptr.as_ptr() as *mut u8,
                            Layout::array::<T>(self_heap.capacity).unwrap(),
                        );
                    }
                }
            }
        }
    }
}

impl<'a, T: Copy> Extend<&'a T> for Repr<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        for elem in iter {
            self.push(*elem);
        }
    }
}

impl<T> Extend<T> for Repr<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        for elem in iter {
            self.push(elem);
        }
    }
}

impl<T> Repr<T> {
    pub fn new_inline(data: &[T]) -> Self {
        let len = data.len();
        assert!(
            mem::size_of::<T>() * len <= INLINE_SIZE,
            "data too large to be stored inline"
        );
        let mut inline_data: [u8; INLINE_SIZE] = unsafe { MaybeUninit::zeroed().assume_init() };
        let inline_t: *mut [T] = unsafe { inline_data.as_mut_slice().align_to_mut::<T>().1 };
        let data_ptr: *const [T] = data;
        unsafe { ptr::copy_nonoverlapping(data_ptr as *const T, inline_t as *mut T, len) };

        Repr {
            inline: ManuallyDrop::new(Inline {
                disc: Discriminant::new(true, len as u8),
                data: inline_data,
                _phantom: PhantomData::<T>,
            }),
        }
    }

    #[inline]
    pub fn from_heap(data: &[T]) -> Self {
        let mut repr = Self::new_heap();

        repr.grow(data.len());

        let ptr: *mut T = repr.as_ptr_mut();
        let data_ptr = data as *const [T];
        unsafe { ptr::copy_nonoverlapping(data_ptr as *const T, ptr as *mut T, data.len()) };

        repr.set_len(data.len());
        repr
    }

    pub fn new_heap() -> Self {
        let len = 0;
        let capacity = if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            0
        };
        let heap_data = NonNull::dangling();

        Repr {
            heap: ManuallyDrop::new(Heap {
                len,
                capacity,
                ptr: heap_data,
            }),
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx >= self.len() {
            None
        } else {
            match self.is_inline() {
                true => Some(&self.inline_data()[idx]),
                false => Some(unsafe { &*self.get_heap().ptr.as_ptr().add(idx) }),
            }
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx >= self.len() {
            None
        } else {
            match self.is_inline() {
                true => Some(&mut self.inline_data_mut()[idx]),
                false => Some(unsafe { &mut *self.get_heap_mut().ptr.as_ptr().add(idx) }),
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match self.is_inline() {
            true => &self.inline_data()[..self.len()],
            false => unsafe {
                &*ptr::slice_from_raw_parts(self.get_heap().ptr.as_ptr(), self.get_heap().len)
            },
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let len = self.len();
        match self.is_inline() {
            true => &mut self.inline_data_mut()[..len],
            false => unsafe {
                &mut *ptr::slice_from_raw_parts_mut(self.get_heap().ptr.as_ptr(), len)
            },
        }
    }

    pub fn extend_from_slice(&mut self, data: &[T]) {
        let new_len = self.len() + data.len();
        let new_size = new_len * mem::size_of::<T>();

        if new_size >= self.capacity() {
            self.grow(new_size);
        }

        let ptr: *mut T = self.as_ptr_mut();
        let data_ptr = data as *const [T];
        unsafe { ptr::copy_nonoverlapping(data_ptr as *const T, ptr, data.len()) };

        self.set_len(new_len);
    }

    pub fn push(&mut self, elem: T) {
        let new_len = self.len() + 1;
        match self.is_inline() {
            true => {
                if new_len * mem::size_of::<T>() <= INLINE_SIZE {
                    let len = self.len();
                    self.inline_data_mut()[len] = elem;
                    self.set_len(new_len);
                } else {
                    self.grow(new_len);
                    self.heap_push(elem);
                }
            }
            false => {
                self.heap_push(elem);
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        let len = self.len();
        let ptr = unsafe { self.as_ptr().add(len - 1) };
        let data = unsafe { ptr::read(ptr) };
        self.set_len(len - 1);
        Some(data)
    }

    pub fn insert(&mut self, idx: usize, element: T) {
        let len = self.len();
        assert!(len + 1 > idx, "index is out of range");

        if len == self.capacity() {
            self.grow(0);
        }

        let count = self.len() - idx;
        let ptr: *mut T = self.as_ptr_mut();

        if idx < len {
            unsafe {
                ptr::copy_nonoverlapping(ptr.add(idx), ptr.add(idx + 1), count);
            }
        } else if idx == len {
            // no shift
        } else {
            unreachable!();
        }

        let dst_ptr: *mut T = unsafe { self.as_ptr_mut().add(idx) };

        unsafe {
            ptr::write(dst_ptr, element);
        }

        self.set_len(len + 1);
    }

    pub fn remove(&mut self, idx: usize) -> T {
        let len = self.len();

        assert!(len >= idx, "index out of range");

        let count = self.len() - idx - 1;
        let ptr: *const T = self.as_ptr();

        let elem = unsafe { ptr::read(ptr) };

        // shuffle

        let ptr: *mut T = self.as_ptr_mut();
        if idx < len {
            unsafe {
                ptr::copy(ptr.add(idx + 1), ptr.add(idx), count);
            }
        } else if idx == len {
            // no shift
        } else {
            unreachable!();
        }

        self.set_len(len - 1);

        elem
    }

    pub fn as_ptr(&self) -> *const T {
        match self.is_inline() {
            true => (self.inline_data() as *const [T]) as *const T,
            false => {
                let self_heap = self.get_heap();
                self_heap.ptr.as_ptr()
            }
        }
    }

    pub fn as_ptr_mut(&mut self) -> *mut T {
        match self.is_inline() {
            true => (self.inline_data_mut() as *mut [T]) as *mut T,
            false => {
                let self_heap = self.get_heap_mut();
                self_heap.ptr.as_ptr() as *mut T
            }
        }
    }

    pub fn heap_push(&mut self, elem: T) {
        if self.len() == self.capacity() {
            self.grow(self.len() + 1);
        }

        let self_heap = self.get_heap_mut();

        unsafe {
            ptr::write(self_heap.ptr.as_ptr().add(self_heap.len), elem);
        }

        self_heap.len += 1;
    }

    fn grow(&mut self, min_size: usize) {
        assert!(mem::size_of::<T>() != 0); // don't grow for zst

        let (new_cap, new_layout) = if self.capacity() == 0 {
            let new_cap = min_size.max(1);
            (new_cap, Layout::array::<T>(new_cap).unwrap())
        } else {
            // Grow at 2x
            let new_cap = min_size.max(2 * self.capacity());
            let new_layout = Layout::array::<T>(new_cap).unwrap();

            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.capacity() == 0 {
            // zst
            unsafe { alloc(new_layout) }
        } else {
            match self.is_inline() {
                true => {
                    // grow from stack to heap
                    debug_assert!(new_cap * mem::size_of::<T>() >= INLINE_SIZE);
                    self.inline_to_heap(new_cap)
                }
                false => {
                    // grow from heap to heap
                    let self_heap = self.get_heap_mut();

                    let old_layout = Layout::array::<T>(self_heap.capacity).unwrap();
                    let old_ptr = self_heap.ptr.as_ptr() as *mut u8;
                    unsafe { realloc(old_ptr, old_layout, new_layout.size()) }
                }
            }
        };

        let self_heap = self.get_heap_mut();

        self_heap.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => handle_alloc_error(new_layout),
        };
        self_heap.capacity = new_cap;
    }

    fn inline_to_heap(&mut self, new_capacity: usize) -> *mut u8 {
        assert!(self.is_inline());

        let new_self = Self::new_heap();
        let len = self.len();

        let old_self: Self = mem::replace(self, new_self);

        let mut self_heap = self.get_heap_mut();

        self_heap.len = len;
        self_heap.capacity = new_capacity; // overwritten again later

        let new_layout = Layout::array::<T>(new_capacity).unwrap();
        let ptr = unsafe { alloc(new_layout) } as *mut T;
        let data_ptr: *const [T] = old_self.inline_data();

        unsafe {
            ptr::copy_nonoverlapping(data_ptr as *const T, ptr as *mut T, len);
        }

        mem::forget(old_self);
        ptr as *mut u8
    }

    #[inline]
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        match self.is_inline() {
            true => {
                let len = self.len();
                unsafe { mem::transmute(&mut self.get_inline_mut().data[..len]) }
            }
            false => {
                let len = self.get_heap().len;
                unsafe {
                    mem::transmute(&mut *ptr::slice_from_raw_parts_mut(
                        self.get_heap_mut().ptr.as_ptr(),
                        len,
                    ))
                }
            }
        }
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        match self.is_inline() {
            true => unsafe { mem::transmute(&self.get_inline().data[..self.len()]) },
            false => unsafe {
                mem::transmute(&*ptr::slice_from_raw_parts(
                    self.get_heap().ptr.as_ptr(),
                    self.get_heap().len,
                ))
            },
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        match self.is_inline() {
            true => INLINE_SIZE,
            false => self.get_heap().capacity,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self.is_inline() {
            true => self.get_inline().disc.len(),
            false => self.get_heap().len,
        }
    }

    #[inline]
    pub fn set_len(&mut self, len: usize) {
        match self.is_inline() {
            true => {
                assert!(len * mem::size_of::<T>() <= INLINE_SIZE);
                self.get_inline_mut().disc.set_len(len as u8)
            }
            false => {
                #[cfg(target_pointer_width = "64")]
                assert!(len * mem::size_of::<T>() < 0x7FFFFFFFFFFFFFFF);
                #[cfg(target_pointer_width = "32")]
                assert!(len * mem::size_of::<T>() < 0x7FFFFFFF);
                self.get_heap_mut().len = len
            }
        }
    }

    #[inline]
    pub fn inline_data(&self) -> &[T] {
        let self_inline = self.get_inline();
        unsafe { self_inline.data.as_slice().align_to().1 }
    }

    #[inline]
    pub fn inline_data_mut(&mut self) -> &mut [T] {
        let self_inline = self.get_inline_mut();
        unsafe { self_inline.data.as_mut_slice().align_to_mut().1 }
    }

    #[inline]
    pub fn is_inline(&self) -> bool {
        let data = unsafe { &self.inline };
        data.disc.variant()
    }

    #[inline]
    fn get_inline(&self) -> &Inline<T> {
        debug_assert!(self.is_inline());

        unsafe { &self.inline }
    }

    #[inline]
    fn get_heap(&self) -> &Heap<T> {
        debug_assert!(!self.is_inline());

        unsafe { &self.heap }
    }

    #[inline]
    fn get_inline_mut(&mut self) -> &mut Inline<T> {
        debug_assert!(self.is_inline());

        unsafe { &mut self.inline }
    }

    #[inline]
    fn get_heap_mut(&mut self) -> &mut Heap<T> {
        debug_assert!(!self.is_inline());

        unsafe { &mut self.heap }
    }
}

#[derive(Clone, Copy)]
pub struct Discriminant(u8);

impl Discriminant {
    fn new(variant: bool, len: u8) -> Self {
        if variant {
            Self(len | 0b10000000)
        } else {
            Self(len << 1 >> 1)
        }
    }

    #[inline]
    fn variant(&self) -> bool {
        match self.0 & 0b10000000 {
            0b10000000 => true,
            _ => false,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        (self.0 << 1 >> 1) as usize
    }

    #[inline]
    fn set_len(&mut self, new_len: u8) {
        self.0 = (self.0 & 0b10000000) | new_len;
    }
}

#[cfg(test)]
mod test {
    use core::mem;

    use super::Discriminant;
    use crate::{repr::Repr, Str};

    #[test]
    fn repr_size() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(mem::size_of::<Repr<u32>>(), 24);

        #[cfg(target_pointer_width = "32")]
        assert_eq!(mem::size_of::<Repr<u32>>(), 12);
    }

    #[test]
    fn str_size_align() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(mem::size_of::<Str>(), 24);

        #[cfg(target_pointer_width = "32")]
        assert_eq!(mem::size_of::<Str>(), 12);
    }

    #[test]
    fn discriminant() {
        assert_eq!(Discriminant::new(true, 1).0, 0b10000001);
        assert_eq!(Discriminant::new(false, 1).0, 0b00000001);

        assert_eq!(Discriminant::new(true, 23).0, 0b10010111);
        assert_eq!(Discriminant::new(false, 23).0, 0b00010111);
    }

    #[test]
    fn discriminant_len() {
        assert_eq!(Discriminant::new(true, 2).len(), 2);
        assert_eq!(Discriminant::new(false, 2).len(), 2);
        assert_eq!(Discriminant::new(true, 36).len(), 36);
        assert_eq!(Discriminant::new(false, 36).len(), 36);
    }

    #[test]
    fn discriminant_set_len() {
        let mut disc = Discriminant::new(true, 13);
        disc.set_len(0);
        assert_eq!(disc.len(), 0);

        let mut disc = Discriminant::new(false, 13);
        disc.set_len(0);
        assert_eq!(disc.len(), 0);

        let mut disc = Discriminant::new(true, 0);
        disc.set_len(20);
        assert_eq!(disc.len(), 20);

        let mut disc = Discriminant::new(false, 0);
        disc.set_len(20);
        assert_eq!(disc.len(), 20);
    }

    #[test]
    fn discriminant_variant() {
        assert_eq!(Discriminant::new(true, 2).variant(), true);
        assert_eq!(Discriminant::new(false, 2).variant(), false);
        assert_eq!(Discriminant::new(true, 36).variant(), true);
        assert_eq!(Discriminant::new(false, 36).variant(), false);
    }
}
