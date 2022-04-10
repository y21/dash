use proc_macro::TokenStream;
use proc_macro_crate::crate_name;
use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::Data;
use syn::Fields;

macro_rules! error {
    ($msg:expr) => {
        return quote!(compile_error!($msg)).into();
    };
}

#[proc_macro_derive(Trace)]
pub fn trace(tt: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tt as syn::DeriveInput);
    let ident = input.ident;

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .map(|x| x.ident.as_ref().unwrap())
                .map(|x| quote! { self.#x.trace(); })
        } else {
            error!("Trace macro can only be used on structs with named fields");
        }
    } else {
        error!("#[derive(Trace)] only works on structs");
    };

    let found_crate = match crate_name("dash-core").unwrap() {
        FoundCrate::Itself => quote!(crate::gc::trace::Trace),
        FoundCrate::Name(crate_name) => quote!(::#crate_name::gc::trace::Trace),
    };

    let expanded = quote! {
        unsafe impl #found_crate for #ident {
            fn trace(&self) {
                #(#fields)*
            }
        }
    };

    expanded.into()
}
