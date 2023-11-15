use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataEnum, Variant, Meta, Expr, ExprLit, Lit};

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
        let mut doc_string = String::new();
        let mut tooltip_string = String::new();
        let mut is_in_tooltip = false;
        for attr in &v.attrs {
            let Meta::NameValue(nv) = &attr.meta else { continue; };
            let Some(ident) = nv.path.get_ident() else { continue; };
            if ident != "doc" { continue; }
            let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value else { continue; };
            let value = s.value();
            let value = value.trim();
            let target_string = if is_in_tooltip {
                &mut tooltip_string
            } else {
                &mut doc_string
            };
            if !target_string.is_empty() {
                if value.is_empty() {
                    if !is_in_tooltip {
                        is_in_tooltip = true;
                        continue;
                    }
                    target_string.push('\n');
                } else if !target_string.ends_with(|c: char| c.is_whitespace()) {
                    target_string.push(' ');
                }
            }
            target_string.push_str(&value);
        }
        if doc_string.is_empty() {
            doc_string = v_str.clone();
        }
        let maybe_tooltip = if tooltip_string.is_empty() {
            quote! { None }
        } else {
            quote! { Some( #tooltip_string ) }
        };
        quote! {
            RadioButtonOption { value: #name::#v_name, key: #v_str, description: #doc_string, tooltip: #maybe_tooltip }
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
