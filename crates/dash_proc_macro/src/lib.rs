use proc_macro::TokenStream;
use proc_macro_crate::crate_name;
use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::Data;
use syn::DataStruct;
use syn::Fields;

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
        FoundCrate::Itself => quote!(crate::gc::trace::Trace),
        FoundCrate::Name(crate_name) => quote!(::#crate_name::gc::trace::Trace),
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
