use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataEnum, Variant, Meta, Expr, ExprLit, Lit, Attribute};

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
        let (doc_string, tooltip_string) = attrs_description_tooltip(&v.attrs);
        let desc_str =  if doc_string.is_empty() {
            v_str.clone()
        } else {
            doc_string
        };
        let maybe_tooltip = if tooltip_string.is_empty() {
            quote! { None }
        } else {
            quote! { Some( #tooltip_string ) }
        };
        quote! {
            ::ugly_widget::radio_button::RadioButtonOption { value: #name::#v_name, key: #v_str, description: #desc_str, tooltip: #maybe_tooltip }
        }
    });

    let gen = quote! {
        impl #impl_generics RadioButtonOptions for #name #ty_generics #where_clause {
            fn radio_button_options() -> Vec<::ugly_widget::radio_button::RadioButtonOption<'static, Self>> {
                vec![ #(#options ,)* ]
            }
        }
    };
    gen.into()
}

fn attrs_description_tooltip(attrs: &[Attribute]) -> (String, String) {
    let lines: Vec<String> = attrs.into_iter().filter_map(|attr| {
        let Meta::NameValue(nv) = &attr.meta else { return None; };
        let ident = nv.path.get_ident()?;
        if ident != "doc" { return None; }
        let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value else { return None; };
        Some(s.value().trim().to_string())
    }).collect();
    let paragraphs: Vec<String> = lines.split(|value| value.is_empty()).map(|ls| {
        ls.join(" ")
    }).collect();
    match paragraphs.split_first() {
        None => ("".to_string(), "".to_string()),
        Some((description, tooltip_paragraphs)) => {
            (description.to_string(), tooltip_paragraphs.join("\n").trim().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn attrs_none() {
        let v: Variant = parse_quote! {
            Thing
        };
        assert_eq!(attrs_description_tooltip(&v.attrs), (
            "".to_string(),
            "".to_string(),
        ));
    }

    #[test]
    fn attrs_description_single() {
        let v: Variant = parse_quote! {
            /// A one-line thing
            Thing
        };
        assert_eq!(attrs_description_tooltip(&v.attrs), (
            "A one-line thing".to_string(),
            "".to_string(),
        ));
    }

    #[test]
    fn attrs_description_multi_single() {
        let v: Variant = parse_quote! {
            /// This is a description spanning
            /// multiple single-line comments
            /// without any blank lines in between.
            Thing
        };
        assert_eq!(attrs_description_tooltip(&v.attrs), (
            "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
            "".to_string(),
        ));
    }

    #[test]
    fn attrs_description_tooltip_multi_single() {
        let v: Variant = parse_quote! {
            /// This is a description spanning
            /// multiple single-line comments
            /// without any blank lines in between.
            /// 
            /// This is a tooltip, with multiple
            /// lines per paragraph.
            /// 
            /// But a tooltip can also have multiple paragraphs,
            /// which are interpreted as multiple lines.
            Thing
        };
        assert_eq!(attrs_description_tooltip(&v.attrs), (
            "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
            "This is a tooltip, with multiple lines per paragraph.\nBut a tooltip can also have multiple paragraphs, which are interpreted as multiple lines.".to_string(),
        ));
    }

    #[test]
    fn attrs_description_tooltip_multi_blank() {
        let v: Variant = parse_quote! {
            /// This is a description spanning
            /// multiple single-line comments
            /// without any blank lines in between.
            /// 
            /// 
            /// This is a tooltip, after multiple
            /// blank lines.
            /// 
            /// 
            /// And a continuation after even more blank lines.
            Thing
        };
        assert_eq!(attrs_description_tooltip(&v.attrs), (
            "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
            "This is a tooltip, after multiple blank lines.\n\nAnd a continuation after even more blank lines.".to_string(),
        ));
    }
}
