use proc_macro;
use quote::quote;
use syn::{self, parse_macro_input};

const MACRO_NAME : &'static str = "DecodeEvent";

#[proc_macro_derive(DecodeEvent)]
pub fn decode_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    match ast.data {
        syn::Data::Enum(enumdata) => {
            let (
                impl_generics,
                _ty_generics,
                where_generics
            ) = ast.generics.split_for_impl();
            if let Some(where_generics) = where_generics {
                syn::Error::new_spanned(where_generics, format!("The {} attribute cannot handle enums with where clauses", MACRO_NAME)).to_compile_error().into()
            } else {
                let variants = enumdata.variants.iter().filter(|variant| {"__Ignore" != variant.ident.to_string()});
                let enum_name = syn::Ident::new(&format!("Decoded{}", ast.ident.to_string()), proc_macro2::Span::call_site());
                proc_macro::TokenStream::from(quote! {
                    // Enum for engine to use that doesn't contain substrtes additions to decode into.
                    #[derive(::codec::Decode)]
                    pub enum #enum_name #impl_generics {
                        #(#variants),*
                    }
                })
            }
        },
        syn::Data::Struct(_) => {
            syn::Error::new_spanned(&ast, format!("{} attribute can only be applied to enums, and {} is a Struct", MACRO_NAME, &ast.ident)).to_compile_error().into()
        },
        syn::Data::Union(_) => {
            syn::Error::new_spanned(&ast, format!("{} attribute can only be applied to enums, and {} is a Union.", MACRO_NAME, &ast.ident)).to_compile_error().into()
        }
    }
}
