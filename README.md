# FFI Destruct
[![crates badge]][crates.io] [![docs badge]][docs.rs] [![build badge]][build]

[crates badge]: https://img.shields.io/crates/v/ffi-destruct.svg?logo=rust
[crates.io]: https://crates.io/crates/ffi-destruct
[docs badge]: https://img.shields.io/docsrs/ffi-destruct/latest?label=docs.rs&logo=docs.rs
[docs.rs]: https://docs.rs/ffi-destruct
[build badge]: https://github.com/I-Info/FFI-destruct/actions/workflows/build.yml/badge.svg
[build]: https://github.com/I-Info/FFI-destruct/actions/workflows/build.yml

Generates destructors for structures that contain raw pointers in the FFI.

## About
The `Destruct` derive macro will implement `Drop` trait and free(drop) memory for structures containing raw pointers.
It may be a common procedure for FFI structure memory operations.

## Supported types
Both `*const` and `*mut` are acceptable. 
But currently, only single-level pointers are supported.

- `* c_char`: c-style string, using `std::ffi::CString::from_raw()` to handle `std::ffi::CString::into_raw()`
- `* <T>`: Using `std::boxed::Box::from_raw()` to handle `std::boxed::Box::into_raw()`

## Example
Provides a structure with several raw pointers that need to be dropped manually.
```rust
use std::ffi::*;
use ffi_destruct::{extern_c_destructor, Destruct};

#[derive(Destruct)]
pub struct MyStruct {
    field: *mut std::ffi::c_char,
}

// Struct definition here, with deriving Destruct and nullable attributes.
#[derive(Destruct)]
pub struct Structure {
    c_string: *const c_char,
    // Default is non-null.
    #[nullable]
    c_string_nullable: *mut c_char,

    other: *mut MyStruct,
    #[nullable]
    other_nullable: *mut MyStruct,

    // Non-pointer types are still available, and will not be added to drop().
    // normal_int: u8,
    // normal_string: String,
}

// (Optional) The macro here generates the destructor: destruct_structure()
extern_c_destructor!(Structure);

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
    // FFI calling
    unsafe {
        destruct_structure(my_struct_ptr);
    }
}
```

After expanding the macro:
```rust
// derive(Destruct)
impl ::std::ops::Drop for MyStruct {
    fn drop(&mut self) {
        unsafe {
            let _ = ::std::ffi::CString::from_raw(self.field as *mut ::std::ffi::c_char);
        }
    }
}

// derive(Destruct)
impl ::std::ops::Drop for Structure {
    fn drop(&mut self) {
        unsafe {
            let _ = ::std::ffi::CString::from_raw(self.c_string as *mut ::std::ffi::c_char);
            if !self.c_string_nullable.is_null() {
                let _ = ::std::ffi::CString::from_raw(
                    self.c_string_nullable as *mut ::std::ffi::c_char,
                );
            }
            let _ = ::std::boxed::Box::from_raw(self.other as *mut MyStruct);
            if !self.other_nullable.is_null() {
                let _ = ::std::boxed::Box::from_raw(self.other_nullable as *mut MyStruct);
            }
        }
    }
}

// extern_c_destructor!() generates snake_case named destructor
#[no_mangle]
pub unsafe extern "C" fn destruct_structure(ptr: *mut Structure) {
    if ptr.is_null() {
        return;
    }
    let _ = ::std::boxed::Box::from_raw(ptr);
}
```