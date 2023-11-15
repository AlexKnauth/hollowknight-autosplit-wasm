use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(SetHeadingLevel)]
pub fn set_heading_level_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_set_heading_level(&ast)
}

fn impl_set_heading_level(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl SetHeadingLevel for #name {
            fn set_heading_level(&mut self, heading_level: u32) {
                self.heading_level = heading_level;
            }
        }
    };
    gen.into()
}
