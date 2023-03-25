use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::crate_name;
use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::Data;
use syn::DataStruct;
use syn::Fields;
use syn::Ident;

macro_rules! error {
    ($msg:expr) => {
        return quote! { compile_error!($msg); }.into()
    };
}

#[proc_macro_derive(Trace)]
pub fn trace(tt: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tt as syn::DeriveInput);
    let ident = input.ident;

    let generics = &input.generics;
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => fields
            .named
            .iter()
            .map(|x| x.ident.as_ref().unwrap())
            .map(|x| quote! { self.#x.trace(); }),
        _ => error!("#[derive(Trace)] can only be used on structs"),
    };

    let found_crate = match crate_name("dash_vm").unwrap() {
        FoundCrate::Itself => quote!(crate::gc2::trace::Trace),
        FoundCrate::Name(crate_name) => {
            let ident = Ident::new(&crate_name, Span::call_site());
            quote!(::#ident::gc2::trace::Trace)
        }
    };

    let generics_names = generics.type_params().map(|x| &x.ident);
    let expanded = quote! {
        unsafe impl #generics #found_crate for #ident <#(#generics_names),*> {
            fn trace(&self) {
                #(#fields)*
            }
        }
    };

    expanded.into()
}
