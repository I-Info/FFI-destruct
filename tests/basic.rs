#![allow(dead_code, unused)]

use ffi_destruct::{extern_c_destructor, Destruct};
use std::ffi::*;

#[derive(Destruct)]
pub struct TestA {
    a: *const std::ffi::c_char,
    b: *const std::os::raw::c_char,
    c: *mut c_char,
}

extern_c_destructor!(TestA);

#[derive(Destruct)]
pub struct MyStruct {
    field: *mut std::ffi::c_char,
}

extern_c_destructor!(MyStruct);

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
    #[no_drop]
    b: *const TestC,
}

struct TestE();

impl Drop for TestE {
    fn drop(&mut self) {
        println!("Dropping TestE");
    }
}

#[derive(Destruct)]
pub struct TestF {
    a: *mut TestE,
    #[nullable]
    b: *mut TestE,
}

extern_c_destructor!(TestF);

#[derive(Destruct)]
pub struct Structure {
    c_string: *const c_char,
    #[nullable]
    c_string_nullable: *mut c_char,

    other: *mut MyStruct,
    #[nullable]
    other_nullable: *mut MyStruct,
}

extern_c_destructor!(Structure);

#[test]
fn test_struct() {
    let my_struct = Structure {
        c_string: CString::new("Hello").unwrap().into_raw(),
        c_string_nullable: std::ptr::null_mut(),
        other: Box::into_raw(Box::new(MyStruct {
            field: CString::new("Hello").unwrap().into_raw(),
        })),
        other_nullable: std::ptr::null_mut(),
    };

    let my_struct_ptr = Box::into_raw(Box::new(my_struct));
    unsafe {
        destruct_structure(my_struct_ptr);
    }
}

#[test]
fn test() {
    let a = TestA {
        a: CString::into_raw(CString::new("123123").unwrap()),
        b: CString::into_raw(CString::new("123123").unwrap()),
        c: CString::into_raw(CString::new("123123").unwrap()),
    };
    let ptr_a = Box::into_raw(Box::new(a));
    unsafe {
        destruct_test_a(ptr_a);
    }
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
        b: std::ptr::null_mut(),
    };
    let e = TestE();
    let f = TestF {
        a: Box::into_raw(Box::new(e)),
        b: std::ptr::null_mut(),
    };
    let ptr_f = Box::into_raw(Box::new(f));
    unsafe {
        destruct_test_f(ptr_f);
    }
}
