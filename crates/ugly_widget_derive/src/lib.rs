use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SetHeadingLevel)]
pub fn set_heading_level_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    impl_set_heading_level(&input)
}

fn impl_set_heading_level(ast: &DeriveInput) -> TokenStream {
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
