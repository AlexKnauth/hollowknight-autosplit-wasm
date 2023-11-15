use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataEnum, Variant};

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

#[proc_macro_derive(RadioButtonOptions)]
pub fn radio_button_options_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    impl_radio_button_options(&ast)
}

fn impl_radio_button_options(ast: &DeriveInput) -> TokenStream {
    // Used in the quasi-quotation below as `#name`.
    let name = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let variants: Vec<&Variant> = match &ast.data {
        Data::Enum(DataEnum { variants, .. }) => variants.into_iter().collect(),
        _ => vec![],
    };

    let options = variants.into_iter().map(|v| {
        let v_name = &v.ident;
        let v_str = v_name.to_string();
        quote! {
            RadioButtonOption { value: #name::#v_name, key: #v_str, description: #v_str, tooltip: None }
        }
    });

    let gen = quote! {
        impl #impl_generics RadioButtonOptions for #name #ty_generics #where_clause {
            fn radio_button_options() -> Vec<RadioButtonOption<'static, Self>> {
                vec![ #(#options ,)* ]
            }
        }
    };
    gen.into()
}
