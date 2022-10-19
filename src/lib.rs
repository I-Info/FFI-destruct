//! # FFI Destruct
//! Generates destructors for structures that contain raw pointers in the FFI.
//!
//! ## Example
//! Provides a structure with several raw pointers that need to be dropped manually.
//! ```
//! use ffi_destruct::{extern_c_destructor, Destruct};
//! use std::ffi::*;
//!
//! #[derive(Destruct)]
//! pub struct MyStruct {
//!     field: *mut std::ffi::c_char,
//! }
//!
//! pub struct AnyOther(u32, u32);
//!
//! // Struct definition here, with deriving Destruct and nullable attributes.
//! #[derive(Destruct)]
//! pub struct Structure {
//!     // Default is non-null.
//!     c_string: *const c_char,
//!     #[nullable]
//!     c_string_nullable: *mut c_char,
//!
//!     other: *mut MyStruct,
//!     #[nullable]
//!     other_nullable: *mut MyStruct,
//!
//!     // Do not drop this field.
//!     #[no_drop]
//!     not_dropped: *const AnyOther,
//!
//!     // Raw pointer for any other things
//!     any: *mut AnyOther,
//!
//!     // Non-pointer types are still available, and will not be added to drop().
//!     pub normal_int: u32,
//!     pub normal_string: String,
//! }
//!
//! // (Optional) The macro here generates the destructor: destruct_structure()
//! extern_c_destructor!(Structure);
//!
//! fn test() {
//!     // Some resources manually managed
//!     let tmp = AnyOther(1, 1);
//!     let tmp_ptr = Box::into_raw(Box::new(tmp));
//!
//!     let my_struct = Structure {
//!         c_string: CString::new("Hello").unwrap().into_raw(),
//!         c_string_nullable: std::ptr::null_mut(),
//!         other: Box::into_raw(Box::new(MyStruct {
//!             field: CString::new("Hello").unwrap().into_raw(),
//!         })),
//!         other_nullable: std::ptr::null_mut(),
//!         not_dropped: tmp_ptr,
//!         any: Box::into_raw(Box::new(AnyOther(1, 1))),
//!         normal_int: 114514,
//!         normal_string: "Hello".to_string(),
//!     };
//!
//!     let my_struct_ptr = Box::into_raw(Box::new(my_struct));
//!     // FFI calling
//!     unsafe {
//!         destruct_structure(my_struct_ptr);
//!     }
//!
//!     // Drop the manually managed resources
//!     unsafe {
//!         let _ = Box::from_raw(tmp_ptr);
//!     }
//! }
//! ```
//!
//! After expanding the macros:
//! ```ignore
//! #[macro_use]
//! #![feature(prelude_import)]
//! #![allow(dead_code)]
//! #[prelude_import]
//! use std::prelude::rust_2021::*;
//! #[macro_use]
//! extern crate std;
//! use ffi_destruct::{extern_c_destructor, Destruct};
//! use std::ffi::*;
//! pub struct MyStruct {
//!     field: *mut std::ffi::c_char,
//! }
//! impl ::std::ops::Drop for MyStruct {
//!     fn drop(&mut self) {
//!         unsafe {
//!             let _ = ::std::ffi::CString::from_raw(self.field as *mut ::std::ffi::c_char);
//!         }
//!     }
//! }
//! pub struct AnyOther(u32, u32);
//! pub struct Structure {
//!     c_string: *const c_char,
//!     #[nullable]
//!     c_string_nullable: *mut c_char,
//!     other: *mut MyStruct,
//!     #[nullable]
//!     other_nullable: *mut MyStruct,
//!     #[no_drop]
//!     not_dropped: *const AnyOther,
//!     any: *mut AnyOther,
//!     pub normal_int: u32,
//!     pub normal_string: String,
//! }
//! impl ::std::ops::Drop for Structure {
//!     fn drop(&mut self) {
//!         unsafe {
//!             let _ = ::std::ffi::CString::from_raw(
//!                 self.c_string as *mut ::std::ffi::c_char,
//!             );
//!             if !self.c_string_nullable.is_null() {
//!                 let _ = ::std::ffi::CString::from_raw(
//!                     self.c_string_nullable as *mut ::std::ffi::c_char,
//!                 );
//!             }
//!             let _ = ::std::boxed::Box::from_raw(self.other as *mut MyStruct);
//!             if !self.other_nullable.is_null() {
//!                 let _ = ::std::boxed::Box::from_raw(
//!                     self.other_nullable as *mut MyStruct,
//!                 );
//!             }
//!             let _ = ::std::boxed::Box::from_raw(self.any as *mut AnyOther);
//!         }
//!     }
//! }
//! #[no_mangle]
//! pub unsafe extern "C" fn destruct_structure(ptr: *mut Structure) {
//!     if ptr.is_null() {
//!         return;
//!     }
//!     let _ = ::std::boxed::Box::from_raw(ptr);
//! }
//! fn test() {
//!     let tmp = AnyOther(1, 1);
//!     let tmp_ptr = Box::into_raw(Box::new(tmp));
//!     let my_struct = Structure {
//!         c_string: CString::new("Hello").unwrap().into_raw(),
//!         c_string_nullable: std::ptr::null_mut(),
//!         other: Box::into_raw(
//!             Box::new(MyStruct {
//!                 field: CString::new("Hello").unwrap().into_raw(),
//!             }),
//!         ),
//!         other_nullable: std::ptr::null_mut(),
//!         not_dropped: tmp_ptr,
//!         any: Box::into_raw(Box::new(AnyOther(1, 1))),
//!         normal_int: 114514,
//!         normal_string: "Hello".to_string(),
//!     };
//!     let my_struct_ptr = Box::into_raw(Box::new(my_struct));
//!     unsafe {
//!         destruct_structure(my_struct_ptr);
//!     }
//!     unsafe {
//!         let _ = Box::from_raw(tmp_ptr);
//!     }
//! }
//! ```

mod destruct;
mod utils;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput};

/// The [`Destruct`] derive macro.
///
/// Generate a destructor for the structure.
///
/// ## Field Attributes
/// - `#[nullable]` - The field is nullable, the destructor will check if the pointer is null before
/// - `#[no_drop]` - The field will not be added to the destructor
#[proc_macro_derive(Destruct, attributes(nullable, no_drop))]
pub fn destruct_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expand = destruct::impl_destruct_macro(&input);

    proc_macro::TokenStream::from(expand)
}

/// Generate extern "C" destructor for provide type
///
/// Provide the function name: "destruct_" + snake_case name of the type.
///
/// ## Usage
///
/// ```
/// // Definition of struct here
/// # use ffi_destruct::{Destruct, extern_c_destructor};
/// #[derive(Destruct)]
/// pub struct MyStruct {
///     field: *mut std::ffi::c_char,
/// }
/// // destructor macro here
/// extern_c_destructor!(MyStruct);
/// ```
/// The macro will be expanded to:
/// ```
/// # use ffi_destruct::Destruct;
/// # #[derive(Destruct)]
/// # pub struct MyStruct {
/// #    field: *mut std::ffi::c_char,
/// # }
/// #[no_mangle]
/// pub unsafe extern "C" fn destruct_my_struct(ptr: *mut MyStruct) {
///     if ptr.is_null() {
///         return;
///     }
///     let _ = ::std::boxed::Box::from_raw(ptr);
/// }
/// ```
#[proc_macro]
pub fn extern_c_destructor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ty: syn::Type = parse_macro_input!(input);
    match ty {
        syn::Type::Path(v) => {
            let ident = v.path.get_ident().expect("Only support single ident.");
            let mut name = ident.to_string().to_case(Case::Snake);
            name.insert_str(0, "destruct_");
            let fn_ident = Ident::new(&name, ident.span());
            quote! {
                #[no_mangle]
                pub unsafe extern "C" fn #fn_ident(ptr: *mut #ident) {
                    if ptr.is_null() {
                        return;
                    }
                    let _ = ::std::boxed::Box::from_raw(ptr);
                }
            }
            .into()
        }
        _ => panic!("Not supported type"),
    }
}
