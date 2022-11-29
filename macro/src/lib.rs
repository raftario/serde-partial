use proc_macro::{Span, TokenStream};
use quote::ToTokens;
use serde_derive_internals::{
    ast::{Container, Data, Style},
    Ctxt, Derive,
};
use syn::{DeriveInput, Error};

#[proc_macro_derive(SerializePartial, attributes(serde))]
pub fn serialize_partial(input: TokenStream) -> TokenStream {
    let cx = Ctxt::new();
    let item = syn::parse_macro_input!(input as DeriveInput);
    let Container {
        data,
        attrs,
        ident,
        original,
        ..
    } = match Container::from_ast(&cx, &item, Derive::Serialize) {
        Some(c) => c,
        None => return item.to_token_stream().into(),
    };
    let ident = &ident;
    let vis = &original.vis;

    if cx.check().is_err() {
        return item.to_token_stream().into();
    }

    let mut fields = match data {
        Data::Struct(Style::Struct, f) => f,
        _ => {
            return Error::new(
                Span::call_site().into(),
                "SerializePartial only supports structs",
            )
            .to_compile_error()
            .into()
        }
    };
    for f in fields.iter_mut() {
        f.attrs.rename_by_rules(attrs.rename_all_rules());
    }
    fields.retain(|f| !f.attrs.skip_serializing());

    let fields_struct_ident = &quote::format_ident!("{}Fields", ident);
    let fields_struct_idents = fields.iter().map(|f| f.original.ident.clone().unwrap());
    let fields_struct_idents_impl = fields_struct_idents.clone();
    let fields_struct_idents_iter = fields_struct_idents.clone();
    let fields_len = fields.len();
    let fields_struct_names = fields.iter().map(|f| f.attrs.name().serialize_name());

    let filter_struct_ident = &quote::format_ident!("{}Filter", ident);
    let filter_struct_idents = fields_struct_idents.clone();
    let filter_struct_idents_impl = filter_struct_idents.clone();
    let filter_struct_names = fields_struct_names.clone();

    let trait_impl_idents = filter_struct_idents.clone();
    let trait_impl_names = filter_struct_names.clone();

    let fields_struct = quote::quote! {
        #[derive(Debug, Clone, Copy)]
        #vis struct #fields_struct_ident {
            #(
                pub #fields_struct_idents: ::serde_partial::Field<'static, #ident>,
            )*
        }

        impl #fields_struct_ident {
            pub const FIELDS: Self = Self {
                #(
                    #fields_struct_idents_impl: ::serde_partial::Field::new(#fields_struct_names),
                )*
            };
        }

        impl ::core::iter::IntoIterator for #fields_struct_ident {
            type Item = ::serde_partial::Field<'static, #ident>;
            type IntoIter = ::core::array::IntoIter<Self::Item, #fields_len>;

            fn into_iter(self) -> Self::IntoIter {
                #[allow(deprecated)]
                ::core::array::IntoIter::new([
                    #(
                        self.#fields_struct_idents_iter,
                    )*
                ])
            }
        }
    };

    let filter_struct = quote::quote! {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        #vis struct #filter_struct_ident {
            #(
                #filter_struct_idents: bool,
            )*
        }

        impl ::serde_partial::SerializeFilter<#ident> for #filter_struct_ident {
            fn skip(&self, field: ::serde_partial::Field<'_, #ident>) -> bool {
                match field.name() {
                    #(
                        #filter_struct_names => !self.#filter_struct_idents_impl,
                    )*
                    _ => panic!("unknown field"),
                }
            }
        }
    };

    let trait_impl = quote::quote! {
        impl<'a> ::serde_partial::SerializePartial<'a> for #ident {
            type Fields = #fields_struct_ident;
            type Filter = #filter_struct_ident;

            fn with_fields<F, I>(&'a self, select: F) -> ::serde_partial::Partial<'a, Self>
            where
                F: ::core::ops::FnOnce(Self::Fields) -> I,
                I: ::core::iter::IntoIterator<Item = ::serde_partial::Field<'a, Self>>,
            {
                let fields = Self::Fields::FIELDS;
                let mut filter = <Self::Filter as ::core::default::Default>::default();

                for filtered in select(fields) {
                    match filtered.name() {
                        #(
                            #trait_impl_names => { filter.#trait_impl_idents = true }
                        )*
                        _ => panic!("unknown field"),
                    }
                }

                ::serde_partial::Partial {
                    value: self,
                    filter,
                }
            }
        }
    };

    let derive = quote::quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, non_camel_case_types)]
        const _: () = {
            #fields_struct
            #filter_struct
            #trait_impl
        };
    };
    derive.into()
}
