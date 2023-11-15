use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SetHeadingLevel)]
pub fn set_heading_level_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    impl_set_heading_level(&ast)
}

fn impl_set_heading_level(ast: &DeriveInput) -> TokenStream {
    // Used in the quasi-quotation below as `#name`.
    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let gen = quote! {
        impl #impl_generics SetHeadingLevel for #name #ty_generics #where_clause {
            fn set_heading_level(&mut self, heading_level: u32) {
                self.heading_level = heading_level;
            }
        }
    };
    gen.into()
}
