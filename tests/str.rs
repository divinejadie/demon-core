use demon_core::Str;

#[test]
fn len_inline() {
    let primitive = "test string";

    assert_eq!(Str::new(primitive).len(), primitive.len());
    assert_eq!(Str::new("").len(), 0)
}

#[test]
fn capacity_inline() {
    let primitive = "test string";

    assert_eq!(Str::new(primitive).capacity(), 23);
    assert_eq!(Str::new("").capacity(), 23);
}

#[test]
fn read_inline() {
    assert_eq!(Str::new("test string"), "test string");
}

#[test]
fn bytes_inline() {
    assert_eq!(
        Str::new(std::str::from_utf8(&[240, 159, 146, 150]).unwrap()).as_bytes(),
        &[240, 159, 146, 150]
    );
}

#[test]
fn bytes_mut_inline() {
    let mut str = Str::new(std::str::from_utf8(&[240, 159, 146, 150]).unwrap());
    unsafe { str.as_bytes_mut()[3] = 151 };

    assert_eq!(str.as_bytes(), &[240, 159, 146, 151]);
    assert_eq!(str.len(), 4);
    assert_eq!(str.capacity(), 23);
}
