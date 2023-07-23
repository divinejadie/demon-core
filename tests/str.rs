extern crate alloc;

use demon_core::Str;
use proptest::prelude::*;

#[test]
fn new() {
    let s = Str::new();

    assert_eq!(&s, "");
    assert_eq!(s.len(), 0);
}

#[test]
fn push_str() {
    let mut s = Str::from("test");
    s.push_str(" string");

    assert_eq!(&s, "test string");
    assert_eq!(s.len(), 11);
}

#[test]
fn push_char() {
    let mut s = Str::from("tes");
    s.push('t');

    assert_eq!(&s, "test");
    assert_eq!(s.len(), 4);
}

#[test]
fn pop_char() {
    let mut s = Str::from("test");
    assert_eq!(s.pop(), Some('t'));
    assert_eq!(s.pop(), Some('s'));
    assert_eq!(s.pop(), Some('e'));
    assert_eq!(s.pop(), Some('t'));
}

#[test]
fn clear() {
    let mut s = Str::from("raspberry");
    let cap = s.capacity();
    s.clear();
    assert_eq!(&s, "");
    assert_eq!(s.len(), 0);
    assert_eq!(s.capacity(), cap);
}

#[test]
fn len_inline() {
    let primitive = "test string";

    assert_eq!(Str::from(primitive).len(), primitive.len());
    assert_eq!(Str::from("").len(), 0)
}

#[test]
fn capacity_inline() {
    let primitive = "test string";

    #[cfg(target_pointer_width = "64")]
    assert_eq!(Str::from(primitive).capacity(), 23);

    #[cfg(target_pointer_width = "64")]
    assert_eq!(Str::from("").capacity(), 23);

    #[cfg(target_pointer_width = "32")]
    assert_eq!(Str::from(primitive).capacity(), 11);

    #[cfg(target_pointer_width = "32")]
    assert_eq!(Str::from("").capacity(), 11);
}

#[test]
fn read_inline() {
    assert_eq!(Str::from("test string"), "test string");
}

#[test]
fn bytes_inline() {
    assert_eq!(
        Str::from(std::str::from_utf8(&[240, 159, 146, 150]).unwrap()).as_bytes(),
        &[240, 159, 146, 150]
    );
}

#[test]
fn bytes_mut_inline() {
    let mut str = Str::from(std::str::from_utf8(&[240, 159, 146, 150]).unwrap());
    unsafe { str.as_bytes_mut()[3] = 151 };

    assert_eq!(str.as_bytes(), &[240, 159, 146, 151]);
    assert_eq!(str.len(), 4);

    #[cfg(target_pointer_width = "64")]
    assert_eq!(str.capacity(), 23);
    #[cfg(target_pointer_width = "32")]
    assert_eq!(str.capacity(), 11);
}

proptest! {
    #[test]
    #[cfg_attr(miri, ignore)]
    fn proptest(text in "\\PC*") {
        let str = Str::from(&text);
        assert_eq!(&str, &text);
    }
}

#[test]
fn clone() {
    let mut s = Str::from("aaa");
    let c = s.clone();
    _ = s.pop();

    assert_eq!(s, "aa");
    assert_eq!(c, "aaa");
}

#[test]
fn display() {
    let s = Str::from("Test String");
    assert_eq!(&format!("{s}"), "Test String");
    assert_eq!(format!("{s}"), format!("{}", "Test String"));
}
