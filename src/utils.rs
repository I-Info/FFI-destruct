/// Check if the attribute exist.
pub fn get_attribute(attrs: &Vec<syn::Attribute>, ident: &str) -> bool {
    let mut exist = false;
    for attr in attrs {
        if attr.path.is_ident(ident) {
            exist = true;
        }
    }
    exist
}
