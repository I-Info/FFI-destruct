# FFI Destruct

Macros generate destructors for structures containing raw pointers in FFI.

## About
The `Destruct` derive macro will implement `Drop` trait and free(drop) memory for structures containing raw pointers.
It may be a common procedure for FFI structure memory operations.

Supported types of raw pointers: 
- `*const/mut c_char`: Using `std::ffi::CString::from_raw()`
- `*const/mut struct`: Using `std::boxed::Box::from_raw()`

## Example
Provides a structure with several raw pointers that need to be dropped manually.
```rust
#[derive(Destruct)]
struct Structure {
    c_string: *const c_char,
    #[nullable]
    c_string_nullable: *mut c_char,

    other: *mut TestA,
    #[nullable]
    other_nullable: *mut TestA,
}
```

The macros will be expanded:
```rust
struct Structure {
    c_string: *const c_char,
    #[nullable]
    c_string_nullable: *mut c_char,
    other: *mut TestA,
    #[nullable]
    other_nullable: *mut TestA,
}

impl ::std::ops::Drop for Structure {
    fn drop(&mut self) {
        unsafe {
            let _ = ::std::ffi::CString::from_raw(
                self.c_string as *mut ::std::ffi::c_char,
            );
            if !self.c_string_nullable.is_null() {
                let _ = ::std::ffi::CString::from_raw(
                    self.c_string_nullable as *mut ::std::ffi::c_char,
                );
            }
            let _ = ::std::boxed::Box::from_raw(self.other as *mut TestA);
            if !self.other_nullable.is_null() {
                let _ = ::std::boxed::Box::from_raw(self.other_nullable as *mut TestA);
            }
        }
    }
}
```