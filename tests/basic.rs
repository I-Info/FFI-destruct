#![allow(dead_code, unused)]

use destruct_macro_derive::Destruct;
use std::ffi::*;

#[derive(Destruct)]
struct TestA {
    a: *const std::ffi::c_char,
    b: *const std::os::raw::c_char,
    c: *mut c_char,
}

#[derive(Destruct)]
struct TestB {
    a: String,
    b: *const String,
}

#[derive(Destruct)]
struct TestC {
    #[nullable]
    a: *const c_char,
    b: u8,
}

#[derive(Destruct)]
struct TestD {
    #[nullable]
    a: *mut TestC,
}

struct TestE();

impl Drop for TestE {
    fn drop(&mut self) {
        println!("Dropping TestE");
    }
}

#[derive(Destruct)]
struct TestF {
    a: *mut TestE,
    #[nullable]
    b: *mut TestE,
}

#[test]
fn test() {
    let a = TestA {
        a: CString::into_raw(CString::new("123123").unwrap()),
        b: CString::into_raw(CString::new("123123").unwrap()),
        c: CString::into_raw(CString::new("123123").unwrap()),
    };
    let b = TestB {
        a: String::from("test"),
        b: Box::into_raw(Box::new(String::from("test"))),
    };
    let c = TestC {
        a: std::ptr::null(),
        b: 0,
    };
    let d = TestD {
        a: Box::into_raw(Box::new(c)),
    };
    let e = TestE();
    let f = TestF {
        a: Box::into_raw(Box::new(e)),
        b: std::ptr::null_mut(),
    };
}
