use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DeriveInput, Expr, ExprLit, Lit, Meta, Variant,
};

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
        impl #impl_generics ::ugly_widget::args::SetHeadingLevel for #name #ty_generics #where_clause {
            fn set_heading_level(&mut self, heading_level: u32) {
                self.heading_level = heading_level;
            }
        }
    };
    gen.into()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OptionAttrs {
    rename: Option<String>,
    alias: Option<String>,
    description: String,
    tooltip: String,
}

#[proc_macro_derive(RadioButtonOptions, attributes(rename, alias))]
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
        let OptionAttrs { rename, alias, description: doc_string, tooltip: tooltip_string } = option_attrs(&v.attrs);
        let alias2 = alias.or_else(|| if rename.is_some() { Some(v_name.to_string()) } else { None });
        let v_str = rename.unwrap_or(v_name.to_string());
        let maybe_alias = if let Some(a) = alias2 {
            quote! { Some( #a ) }
        } else {
            quote! { None }
        };
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
            ::ugly_widget::radio_button::RadioButtonOption { value: #name::#v_name, key: #v_str, alias: #maybe_alias, description: #desc_str, tooltip: #maybe_tooltip }
        }
    });

    let gen = quote! {
        impl #impl_generics ::ugly_widget::radio_button::RadioButtonOptions for #name #ty_generics #where_clause {
            fn radio_button_options() -> Vec<::ugly_widget::radio_button::RadioButtonOption<'static, Self>> {
                vec![ #(#options ,)* ]
            }
        }
    };
    gen.into()
}

fn option_attrs(attrs: &[Attribute]) -> OptionAttrs {
    let mut rename: Option<String> = None;
    let mut alias: Option<String> = None;
    let mut doc_lines: Vec<String> = Vec::new();
    for attr in attrs {
        if let Meta::NameValue(nv) = &attr.meta {
            if let Some(ident) = nv.path.get_ident() {
                if ident == "doc" {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &nv.value
                    {
                        doc_lines.push(s.value().trim().to_string());
                    }
                } else if ident == "rename" {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &nv.value
                    {
                        rename.get_or_insert(s.value().trim().to_string());
                    }
                } else if ident == "alias" {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &nv.value
                    {
                        alias.get_or_insert(s.value().trim().to_string());
                    }
                }
            }
        }
    }
    let (description, tooltip) = lines_description_tooltip(&doc_lines);
    OptionAttrs {
        rename,
        alias,
        description,
        tooltip,
    }
}

fn lines_description_tooltip(lines: &[String]) -> (String, String) {
    let paragraphs: Vec<String> = lines
        .split(|value| value.is_empty())
        .map(|ls| ls.join(" "))
        .collect();
    match paragraphs.split_first() {
        None => ("".to_string(), "".to_string()),
        Some((description, tooltip_paragraphs)) => (
            description.to_string(),
            tooltip_paragraphs.join("\n").trim().to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn attrs_rename() {
        let v: Variant = parse_quote! {
            #[rename = "mapCrossroads"]
            Thing
        };
        assert_eq!(
            option_attrs(&v.attrs),
            OptionAttrs {
                rename: Some("mapCrossroads".to_string()),
                alias: None,
                description: "".to_string(),
                tooltip: "".to_string(),
            }
        );
    }

    #[test]
    fn attrs_alias() {
        let v: Variant = parse_quote! {
            #[alias = "LegacyEnd"]
            Thing
        };
        assert_eq!(
            option_attrs(&v.attrs),
            OptionAttrs {
                rename: None,
                alias: Some("LegacyEnd".to_string()),
                description: "".to_string(),
                tooltip: "".to_string(),
            }
        );
    }

    #[test]
    fn attrs_rename_alias() {
        let v: Variant = parse_quote! {
            #[rename = "mapCrossroads"]
            #[alias = "MapCrossroads"]
            Thing
        };
        assert_eq!(
            option_attrs(&v.attrs),
            OptionAttrs {
                rename: Some("mapCrossroads".to_string()),
                alias: Some("MapCrossroads".to_string()),
                description: "".to_string(),
                tooltip: "".to_string(),
            }
        );
    }

    #[test]
    fn attrs_none() {
        assert_eq!(
            lines_description_tooltip(&[]),
            ("".to_string(), "".to_string(),)
        );
        let v: Variant = parse_quote! {
            Thing
        };
        assert_eq!(
            option_attrs(&v.attrs),
            OptionAttrs {
                rename: None,
                alias: None,
                description: "".to_string(),
                tooltip: "".to_string(),
            }
        );
    }

    #[test]
    fn attrs_description_single() {
        assert_eq!(
            lines_description_tooltip(&["A one-line thing".to_string()]),
            ("A one-line thing".to_string(), "".to_string(),)
        );
        let v: Variant = parse_quote! {
            /// A one-line thing
            Thing
        };
        assert_eq!(
            option_attrs(&v.attrs),
            OptionAttrs {
                rename: None,
                alias: None,
                description: "A one-line thing".to_string(),
                tooltip: "".to_string(),
            }
        );
    }

    #[test]
    fn attrs_description_multi_single() {
        assert_eq!(
            lines_description_tooltip(&[
                "This is a description spanning".to_string(),
                "multiple single-line comments".to_string(),
                "without any blank lines in between.".to_string(),
            ]),
            (
                "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
                "".to_string(),
            ),
        );
        let v: Variant = parse_quote! {
            /// This is a description spanning
            /// multiple single-line comments
            /// without any blank lines in between.
            Thing
        };
        assert_eq!(option_attrs(&v.attrs), OptionAttrs {
            rename: None,
            alias: None,
            description: "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
            tooltip: "".to_string(),
        });
    }

    #[test]
    fn attrs_description_tooltip_multi_single() {
        assert_eq!(
            lines_description_tooltip(&[
                "This is a description spanning".to_string(),
                "multiple single-line comments".to_string(),
                "without any blank lines in between.".to_string(),
                "".to_string(),
                "This is a tooltip, with multiple".to_string(),
                "lines per paragraph.".to_string(),
                "".to_string(),
                "But a tooltip can also have multiple paragraphs,".to_string(),
                "which are interpreted as multiple lines.".to_string(),
            ]),
            (
                "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
                "This is a tooltip, with multiple lines per paragraph.\nBut a tooltip can also have multiple paragraphs, which are interpreted as multiple lines.".to_string(),
            ),
        );
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
        assert_eq!(option_attrs(&v.attrs), OptionAttrs {
            rename: None,
            alias: None,
            description: "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
            tooltip: "This is a tooltip, with multiple lines per paragraph.\nBut a tooltip can also have multiple paragraphs, which are interpreted as multiple lines.".to_string(),
        });
    }

    #[test]
    fn attrs_description_tooltip_multi_blank() {
        assert_eq!(
            lines_description_tooltip(&[
                "This is a description spanning".to_string(),
                "multiple single-line comments".to_string(),
                "without any blank lines in between.".to_string(),
                "".to_string(),
                "".to_string(),
                "This is a tooltip, after multiple".to_string(),
                "blank lines.".to_string(),
                "".to_string(),
                "".to_string(),
                "And a continuation after even more blank lines.".to_string(),
            ]),
            (
                "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
                "This is a tooltip, after multiple blank lines.\n\nAnd a continuation after even more blank lines.".to_string(),
            ),
        );
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
        assert_eq!(option_attrs(&v.attrs), OptionAttrs {
            rename: None,
            alias: None,
            description: "This is a description spanning multiple single-line comments without any blank lines in between.".to_string(),
            tooltip: "This is a tooltip, after multiple blank lines.\n\nAnd a continuation after even more blank lines.".to_string(),
        });
    }
}
