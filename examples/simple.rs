#![allow(dead_code)]

use ffi_destruct::{extern_c_destructor, Destruct};
use std::ffi::*;

#[derive(Destruct)]
pub struct MyStruct {
    field: *mut std::ffi::c_char,
}

pub struct AnyOther(u32, u32);

// Struct definition here, with deriving Destruct and nullable attributes.
#[derive(Destruct)]
pub struct Structure {
    // Default is non-null.
    c_string: *const c_char,
    #[nullable]
    c_string_nullable: *mut c_char,

    other: *mut MyStruct,
    #[nullable]
    other_nullable: *mut MyStruct,

    // Do not drop this field.
    #[no_drop]
    not_dropped: *const AnyOther,

    // Raw pointer for any other things
    any: *mut AnyOther,

    // Non-pointer types are still available, and will not be added to drop().
    pub normal_int: u32,
    pub normal_string: String,
}

// (Optional) The macro here generates the destructor: destruct_structure()
extern_c_destructor!(Structure);

fn main() {
    // Some resources manually managed
    let tmp = AnyOther(1, 1);
    let tmp_ptr = Box::into_raw(Box::new(tmp));

    let my_struct = Structure {
        c_string: CString::new("Hello").unwrap().into_raw(),
        c_string_nullable: std::ptr::null_mut(),
        other: Box::into_raw(Box::new(MyStruct {
            field: CString::new("Hello").unwrap().into_raw(),
        })),
        other_nullable: std::ptr::null_mut(),
        not_dropped: tmp_ptr,
        any: Box::into_raw(Box::new(AnyOther(1, 1))),
        normal_int: 114514,
        normal_string: "Hello".to_string(),
    };

    let my_struct_ptr = Box::into_raw(Box::new(my_struct));
    // FFI calling
    unsafe {
        destruct_structure(my_struct_ptr);
    }

    // Drop the manually managed resources
    unsafe {
        let _ = Box::from_raw(tmp_ptr);
    }
}
