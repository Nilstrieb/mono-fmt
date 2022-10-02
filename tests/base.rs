use std::cell::Cell;

#[macro_use]
extern crate mono_fmt;

#[test]
fn test_format_flags() {
    // No residual flags left by pointer formatting
    let p = "".as_ptr();
    assert_eq!(format!("{:p} {:x}", p, 16), format!("{p:p} 10"));

    assert_eq!(format!("{: >3}", 'a'), "  a");
}

#[test]
fn test_pointer_formats_data_pointer() {
    let b: &[u8] = b"";
    let s: &str = "";
    assert_eq!(format!("{s:p}"), format!("{:p}", s.as_ptr()));
    assert_eq!(format!("{b:p}"), format!("{:p}", b.as_ptr()));
}

#[test]
fn only_eval_once() {
    let evil = Cell::new(0);
    let _ = format!("{0} {0}", {
        evil.set(evil.get() + 1);
        0
    });
    assert_eq!(evil.get(), 1);
}
