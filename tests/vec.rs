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
fn extend() {
    let mut vec = Vector::<u32>::new();
    vec.extend(&[0, 1, 2, 3, 4]);
    assert_eq!(vec.len(), 5);
    assert_eq!(vec.is_inline(), true);
    assert_eq!(vec, &[0, 1, 2, 3, 4]);

    let mut vec = Vector::<u32>::new_heap();
    vec.extend(&[0, 1, 2, 3, 4]);
    assert_eq!(vec.len(), 5);
    assert_eq!(vec.is_inline(), false);
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
fn collect() {
    let iter = [0i32, 1i32, 2i32, 4i32].into_iter();
    let vec: Vector<i32> = iter.collect();
    assert_eq!(vec, &[0i32, 1i32, 2i32, 4i32]);
}
