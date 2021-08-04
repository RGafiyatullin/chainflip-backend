use proc_macro;
use quote::quote;
use syn::{self, parse_macro_input};

const MACRO_NAME : &'static str = "PalletEvent";

#[proc_macro_derive(PalletEvent)]
pub fn pallet_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    match ast.data {
        syn::Data::Enum(enumdata) => {
            let unnamed_field_errors = enumdata.variants.iter().filter_map(|variant| {
                match &variant.fields {
                    syn::Fields::Named(_) => None,
                    _ => Some(syn::Error::new_spanned(&variant, format!("The {} attribute can only allow enums with named fields", MACRO_NAME)).to_compile_error())
                }
            }).collect::<Vec<_>>();
            if !unnamed_field_errors.is_empty() {
                proc_macro::TokenStream::from(quote! { #(#unnamed_field_errors)* })
            } else {
                let ident = &ast.ident;
                let (
                    impl_generics,
                    ty_generics,
                    where_clause
                ) = ast.generics.split_for_impl();
                let variant_structs = enumdata.variants.iter().map(|variant| {
                    let variant_ident = &variant.ident;
                    let fields = variant.fields.iter().map(|field| {
                        quote! { #field }
                    });

                    quote! {
                        #[derive(::codec::Encode, ::codec::Decode)]
                        struct #variant_ident {
                            #(#fields),*
                        }
                    }
                });
                let variant_match_branch = enumdata.variants.iter().map(|variant| {
                    let variant_ident = &variant.ident;
                    let fields = variant.fields.iter().map(|field| {
                        let field_ident = &field.ident;
                        quote! { #field_ident : variant_struct.#field_ident }
                    });

                    quote! {
                        stringify!(#variant_ident) => {
                            let variant_struct = #variant_ident ::decode(data)?;
                            Some(Self::#variant_ident {
                                #(#fields),*
                            })
                        }
                    }
                });
                proc_macro::TokenStream::from(quote! {
                    impl #impl_generics #ident #ty_generics #where_clause {
                        fn try_decode<I : ::codec::Input>(variant : &str, data : &mut I) -> ::std::result::Result<::std::option::Option<Self>, ::codec::Error> {
                            #(#variant_structs)*

                            Ok(match variant {
                                #(#variant_match_branch),*
                                _ => None
                            })
                        }
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
