use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{Data, DataStruct, Fields, Ident};

macro_rules! error {
    ($msg:expr) => {
        return quote! { compile_error!($msg); }.into()
    };
}

pub fn trace_impl(tt: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tt as syn::DeriveInput);
    let ident = input.ident;

    let generics = &input.generics;
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => Box::new(
            fields
                .named
                .iter()
                .map(|x| x.ident.as_ref().unwrap())
                .map(|x| quote! { self.#x.trace(cx); }),
        ) as Box<dyn Iterator<Item = _>>,
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(ref fields),
            ..
        }) => Box::new(
            fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(x, _)| syn::Index::from(x))
                .map(|x| quote! { self.#x.trace(cx); }),
        ) as Box<dyn Iterator<Item = _>>,
        _ => error!("#[derive(Trace)] can only be used on structs"),
    };

    let (trace_trait, trace_ctxt) = match crate_name("dash_vm").unwrap() {
        FoundCrate::Itself => (quote!(crate::gc::trace::Trace), quote!(crate::gc::trace::TraceCtxt<'_>)),
        FoundCrate::Name(crate_name) => {
            let ident = Ident::new(&crate_name, Span::call_site());
            (
                quote!(::#ident::gc::trace::Trace),
                quote!(::#ident::gc::trace::TraceCtxt<'_>),
            )
        }
    };

    let generics_names = generics.type_params().map(|x| &x.ident);
    let expanded = quote! {
        unsafe impl #generics #trace_trait for #ident <#(#generics_names),*> {
            #[allow(unused_variables)]
            fn trace(&self, cx: &mut #trace_ctxt) {
                #(#fields)*
            }
        }
    };

    expanded.into()
}
