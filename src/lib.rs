//! # FFI Destruct
//! Generates destructors for structures that contain raw pointers in the FFI.
//!
//! ## Example
//! Provides a structure with several raw pointers that need to be dropped manually.
//! ```no_run
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
//!     let my_struct = Structure {
//!         c_string: CString::new("Hello").unwrap().into_raw(),
//!         c_string_nullable: std::ptr::null_mut(),
//!         other: Box::into_raw(Box::new(MyStruct {
//!             field: CString::new("Hello").unwrap().into_raw(),
//!         })),
//!         other_nullable: std::ptr::null_mut(),
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
//! }
//! ```
//!
//! After expanding the macros:
//! ```ignore
//! #[macro_use]
//! extern crate std;
//! use ffi_destruct::{extern_c_destructor, Destruct};
//! use std::ffi::*;
//!
//! pub struct MyStruct {
//!     field: *mut std::ffi::c_char,
//! }
//!
//! impl ::std::ops::Drop for MyStruct {
//!     fn drop(&mut self) {
//!         unsafe {
//!             let _ = ::std::ffi::CString::from_raw(self.field as *mut ::std::ffi::c_char);
//!         }
//!     }
//! }
//!
//! pub struct AnyOther(u32, u32);
//!
//! pub struct Structure {
//!     c_string: *const c_char,
//!     #[nullable]
//!     c_string_nullable: *mut c_char,
//!     other: *mut MyStruct,
//!     #[nullable]
//!     other_nullable: *mut MyStruct,
//!     any: *mut AnyOther,
//!     pub normal_int: u32,
//!     pub normal_string: String,
//! }
//!
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
//!
//! #[no_mangle]
//! pub unsafe extern "C" fn destruct_structure(ptr: *mut Structure) {
//!     if ptr.is_null() {
//!         return;
//!     }
//!     let _ = ::std::boxed::Box::from_raw(ptr);
//! }
//!
//! fn main() {
//!     let my_struct = Structure {
//!         c_string: CString::new("Hello").unwrap().into_raw(),
//!         c_string_nullable: std::ptr::null_mut(),
//!         other: Box::into_raw(
//!             Box::new(MyStruct {
//!                 field: CString::new("Hello").unwrap().into_raw(),
//!             }),
//!         ),
//!         other_nullable: std::ptr::null_mut(),
//!         any: Box::into_raw(Box::new(AnyOther(1, 1))),
//!         normal_int: 114514,
//!         normal_string: "Hello".to_string(),
//!     };
//!     let my_struct_ptr = Box::into_raw(Box::new(my_struct));
//!     unsafe {
//!         destruct_structure(my_struct_ptr);
//!     }
//! }
//! ```

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
#[proc_macro_derive(Destruct, attributes(nullable))]
pub fn destruct_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let destructors = field_destructors(&input.data);

    let expand = quote! {
        impl ::std::ops::Drop for #name {
            fn drop(&mut self) {
                unsafe {
                    #destructors
                }
            }
        }
    };

    proc_macro::TokenStream::from(expand)
}

/// Parsing fields and generating destructors for them.
fn field_destructors(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let attrs = &f.attrs;

                    let nullable = get_attribute_nullable(attrs);

                    match f.ty {
                        // Raw pointer destructor
                        syn::Type::Ptr(ref ty) => {
                            let destructor = destruct_type_ptr(name.as_ref().unwrap(), ty);
                            if nullable {
                                quote_spanned! { f.span() =>
                                    if !self.#name.is_null() {
                                        #destructor
                                    }
                                }
                            } else {
                                quote_spanned! { f.span() =>
                                    #destructor
                                }
                            }
                        }
                        // Other types don't require manual destructors
                        _ => {
                            if nullable {
                                panic!("Nullable attribute is only supported for raw pointers");
                            }
                            TokenStream::new() // Empty
                        }
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            syn::Fields::Unnamed(_) => unimplemented!("Unnamed fields are not supported"),
            syn::Fields::Unit => panic!("Unit structs cannot be destructed"),
        },
        _ => panic!("Destruct can only be derived for structs"),
    }
}

/// Check if the field is nullable.
fn get_attribute_nullable(attrs: &Vec<syn::Attribute>) -> bool {
    let mut nullable = false;
    for attr in attrs {
        if attr.path.is_ident("nullable") {
            nullable = true;
        }
    }
    nullable
}

/// Generate destructor for raw pointer types
fn destruct_type_ptr(name: &Ident, ty: &syn::TypePtr) -> TokenStream {
    /// Some variant of `c_char` type paths: `std::ffi:c_char`,`libc::c_char`, `std::os::raw::c_char`,`c_char`,
    fn is_c_char(path: &str) -> bool {
        path.contains("c_char")
    }

    match *ty.elem {
        syn::Type::Path(ref path) => {
            let ts = path.path.to_token_stream();
            let path_string = ts.to_string();
            if is_c_char(&path_string) {
                // Drop c-string
                quote_spanned! { ty.span()=>
                    let _ = ::std::ffi::CString::from_raw(self.#name as *mut ::std::ffi::c_char);
                }
            } else {
                // Drop other raw pointer
                quote_spanned! { ty.span()=>
                    let _ = ::std::boxed::Box::from_raw(self.#name as *mut #ts);
                }
            }
        }
        _ => panic!("Only single level raw pointers are supported"),
    }
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
