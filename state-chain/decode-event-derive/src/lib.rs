use proc_macro;
use quote::quote;
use syn::{self, parse_macro_input};

const MACRO_NAME : &'static str = "DecodeEvent";

#[proc_macro_derive(DecodeEvent)]
pub fn decode_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    match ast.data {
        syn::Data::Enum(enumdata) => {
            let unnamed_field_errors = enumdata.variants.iter().filter(|variant| {"__Ignore" != variant.ident.to_string()}).filter_map(|variant| {
                match &variant.fields {
                    syn::Fields::Named(_) => None,
                    _ => Some(syn::Error::new_spanned(&variant, format!("The {} attribute can only allow enums with named fields", MACRO_NAME)).to_compile_error())
                }
            }).collect::<Vec<_>>();
            if !unnamed_field_errors.is_empty() {
                proc_macro::TokenStream::from(quote! { #(#unnamed_field_errors)* })
            } else {
                if let Some(lt) = ast.generics.lifetimes().next() {
                    syn::Error::new_spanned(&lt, format!("The {} attribute cannot handle enums with lifetimes", MACRO_NAME)).to_compile_error().into()
                } else {
                    let (
                        impl_generics,
                        ty_generics,
                        where_generics
                    ) = ast.generics.split_for_impl();
                    if let Some(where_generics) = where_generics {
                        syn::Error::new_spanned(where_generics, format!("The {} attribute cannot handle enums with where clauses", MACRO_NAME)).to_compile_error().into()
                    } else {
                        let phantomdata = ast.generics.type_params().map(|type_param| {
                            let type_ident = &type_param.ident;
                            let ident = syn::Ident::new(&format!("__{}", type_ident.to_string().to_lowercase()), proc_macro2::Span::call_site());
                            quote! {#ident : ::sp_std::marker::PhantomData<#type_ident> }
                        }).collect::<Vec<_>>();
                        let variants = enumdata.variants.iter().filter(|variant| {"__Ignore" != variant.ident.to_string()});
                        let variant_structs = variants.clone().map(|variant| {
                            let variant_ident = syn::Ident::new(&format!("DontUseThis{}", variant.ident.to_string()), proc_macro2::Span::call_site());
                            let fields = variant.fields.iter().map(|field| {
                                quote! { #field }
                            }).collect::<Vec<_>>();
                            let all_fields = fields.iter().chain(phantomdata.iter());
                            quote! {
                                #[derive(::codec::Encode, ::codec::Decode)]
                                struct #variant_ident #impl_generics {
                                    #(#all_fields),*
                                }
                            }
                        });
                        let variant_match_branch = variants.clone().map(|variant| {
                            let variant_struct_ident = syn::Ident::new(&format!("DontUseThis{}", variant.ident.to_string()), proc_macro2::Span::call_site());
                            let variant_ident = &variant.ident;
                            let fields = variant.fields.iter().map(|field| {
                                let field_ident = &field.ident;
                                quote! { #field_ident : variant_struct.#field_ident }
                            });
                            let turbofish = ty_generics.as_turbofish();
                            quote! {
                                stringify!(#variant_ident) => {
                                    let variant_struct = #variant_struct_ident #turbofish ::decode(data)?;
                                    Some(Self::#variant_ident {
                                        #(#fields),*
                                    })
                                }
                            }
                        });
                        let enum_name = syn::Ident::new(&format!("Decoded{}", ast.ident.to_string()), proc_macro2::Span::call_site());
                        proc_macro::TokenStream::from(quote! {
                            // Enum for engine to use that doesn't contain substrtes additions to decode into.
                            pub enum #enum_name #impl_generics {
                                #(#variants),*
                            }

                            impl #impl_generics ::decode_event::DecodeEvent for #enum_name #ty_generics {
                                // Cannot import substrate_xt into state_chain, so cannot have raw_event as parameter
                                fn decode_event<I : ::codec::Input>(variant : &str, data : &mut I) -> core::result::Result<core::option::Option<Self>, ::codec::Error> {
                                    #(#variant_structs)*

                                    Ok(match variant {
                                        #(#variant_match_branch),*
                                        _ => None
                                    })
                                }
                            }
                        })
                    }
                }
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
