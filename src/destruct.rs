use super::*;

pub fn impl_destruct_macro(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let destructors = field_destructors(&input.data);

    quote! {
        impl ::std::ops::Drop for #name {
            fn drop(&mut self) {
                unsafe {
                    #destructors
                }
            }
        }
    }
}

/// Parsing fields and generating destructors for them.
fn field_destructors(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let attrs = &f.attrs;

                    let nullable = utils::get_attribute(attrs, "nullable");
                    let no_drop = utils::get_attribute(attrs, "no_drop");

                    match f.ty {
                        // Raw pointer destructor
                        syn::Type::Ptr(ref ty) => {
                            let destructor = destruct_type_ptr(name.as_ref().unwrap(), ty);
                            if no_drop {
                                TokenStream::new()
                            } else if nullable {
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
                            if no_drop {
                                panic!("No drop attribute is only supported for raw pointers");
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
