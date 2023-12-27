use proc_macro::TokenStream;

mod define_symbols;
mod trace;

#[proc_macro_derive(Trace)]
pub fn trace(tt: TokenStream) -> TokenStream {
    trace::trace_impl(tt)
}

#[proc_macro]
pub fn define_symbols(tt: TokenStream) -> TokenStream {
    define_symbols::define_symbols_impl(tt)
}
