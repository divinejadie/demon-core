use demon_core::Vector;

#[test]
fn get() {
    let mut vec = Vector::<u32>::new();

    assert_eq!(vec.get(0), None);

    vec.push(1);

    assert_eq!(vec.get(0), Some(&1));
    assert_eq!(vec.is_inline(), true);

    let mut vec = Vector::<u32>::new_heap();

    assert_eq!(vec.get(0), None);
    assert_eq!(vec.is_inline(), false);

    vec.push(1);

    assert_eq!(vec.get(0), Some(&1));
}

#[test]
fn get_mut() {
    let mut vec = Vector::<u32>::new();

    assert_eq!(vec.get_mut(0), None);

    vec.push(1);

    assert_eq!(vec.get(0), Some(&1));
    *vec.get_mut(0).unwrap() = 0;
    assert_eq!(vec.get(0), Some(&0));
    assert_eq!(vec.is_inline(), true);

    let mut vec = Vector::<u32>::new_heap();

    assert_eq!(vec.get_mut(0), None);
    assert_eq!(vec.is_inline(), false);

    vec.push(1);

    assert_eq!(vec.get(0), Some(&1));
    *vec.get_mut(0).unwrap() = 0;
    assert_eq!(vec.get(0), Some(&0));
}

#[test]
fn push() {
    let mut vec = Vector::<u32>::new();
    vec.push(0);
    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec[0], 0);

    let mut vec = Vector::<u32>::new_heap();
    vec.push(1);

    assert_eq!(vec.is_inline(), false);
    assert_eq!(vec[0], 1);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn extend() {
    let mut vec = Vector::<u32>::new();
    vec.extend(&[0, 1, 2, 3]);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec, &[0, 1, 2, 3]);

    let mut vec = Vector::<u32>::new_heap();
    vec.extend(&[0, 1, 2, 3, 4]);
    assert_eq!(vec.len(), 5);
    assert_eq!(vec.is_inline(), false);
    assert_eq!(vec, &[0, 1, 2, 3, 4]);

    let mut vec = Vector::<u32>::new();
    vec.extend_from_slice(&[0, 1, 2, 3, 4]);
    assert_eq!(vec, &[0, 1, 2, 3, 4]);
}

#[test]
#[cfg(target_pointer_width = "32")]
fn extend() {
    let mut vec = Vector::<u32>::new();
    vec.extend(&[0, 1]);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec, &[0, 1]);

    let mut vec = Vector::<u32>::new_heap();
    vec.extend(&[0, 1, 2, 3, 4]);
    assert_eq!(vec.len(), 5);
    assert_eq!(vec.is_inline(), false);
    assert_eq!(vec, &[0, 1, 2, 3, 4]);

    let mut vec = Vector::<u32>::new();
    vec.extend_from_slice(&[0, 1, 2, 3, 4]);
    assert_eq!(vec, &[0, 1, 2, 3, 4]);
}

#[test]
fn pop() {
    let mut vec = Vector::<u32>::new();
    vec.push(4);

    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.pop(), Some(4));
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.len(), 0);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn grow() {
    let mut vec = Vector::<u8>::new();
    vec.extend(&[
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
    ]);

    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec.len(), 22);

    vec.push(22);
    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec.len(), 23);

    vec.push(23);
    assert_eq!(vec.is_inline(), false);
    assert_eq!(vec.len(), 24);

    assert_eq!(
        vec,
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
    )
}

#[test]
#[cfg(target_pointer_width = "32")]
fn grow() {
    let mut vec = Vector::<u8>::new();
    vec.extend(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec.len(), 10);

    vec.push(10);
    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec.len(), 11);

    vec.push(11);
    assert_eq!(vec.is_inline(), false);
    assert_eq!(vec.len(), 12);

    assert_eq!(vec, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
}

#[test]
fn from() {
    let vec = Vector::<i32>::from_heap(&[0, 2, 3, 4]);
    assert_eq!(vec.is_inline(), false);
    assert_eq!(vec, &[0, 2, 3, 4]);
}

#[test]
fn collect() {
    let iter = [0i32, 1i32, 2i32, 4i32].into_iter();
    let vec: Vector<i32> = iter.collect();
    assert_eq!(vec, &[0i32, 1i32, 2i32, 4i32]);
}
#[test]
fn insert() {
    let mut vec = Vector::new();
    vec.extend(&[0, 1, 3]);
    vec.insert(2, 2);
    assert_eq!(vec, &[0, 1, 2, 3]);

    let mut vec = Vector::new();
    vec.extend(&[0, 1, 3]);
    vec.insert(3, 2);

    assert_eq!(vec, &[0, 1, 3, 2]);

    let mut vec = Vector::new();
    vec.insert(0, 2);
    assert_eq!(vec[0], 2);
    let mut vec = Vector::new_heap();
    vec.extend(&[0, 1, 3]);
    vec.insert(3, 2);
    assert_eq!(vec, &[0, 1, 3, 2]);
}
