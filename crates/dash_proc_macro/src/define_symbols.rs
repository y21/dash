use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, Lit, Member};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Keyword = 0,
    Symbol = 1,
}

pub fn define_symbols_impl(tt: TokenStream) -> TokenStream {
    let Expr::Array(arr) = syn::parse_macro_input!(tt as syn::Expr) else {
        panic!("must be an array");
    };

    let mut symbols = Vec::new();

    for expr in arr.elems {
        if let Expr::Struct(strukt) = expr {
            let kind = match strukt.path.segments.last().unwrap().ident.to_string().as_ref() {
                "Keywords" => Kind::Keyword,
                "Symbols" => Kind::Symbol,
                other => panic!("unknown struct: {}", other),
            };

            for field in strukt.fields {
                let Member::Named(name) = field.member else {
                    todo!("must be a named member")
                };

                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(string), ..
                }) = field.expr
                {
                    // alias, e.g. `true_: "true"`
                    symbols.push((kind, name, string.value()));
                } else {
                    let v = name.to_string();
                    symbols.push((kind, name, v));
                }
            }
        } else {
            panic!("must be a struct")
        }
    }

    let mut rust_idents = HashSet::new();
    let mut js_idents = HashSet::new();
    for (_, rust_sym, js_sym) in &symbols {
        if !rust_idents.insert(rust_sym) {
            panic!("duplicate rust ident: {}", rust_sym);
        }
        if !js_idents.insert(js_sym) {
            panic!("duplicate js ident: {}", js_sym);
        }
    }

    symbols.sort_by(|a, b| a.0.cmp(&b.0));

    // defines all the const symbols
    let consts = symbols
        .iter()
        .enumerate()
        .map(|(index, (_, rust_sym, _))| {
            let index = index as u32;
            quote! {
                pub const #rust_sym: Symbol = Symbol(#index);
            }
        })
        .collect::<proc_macro2::TokenStream>();

    // defines the symbol array
    let preinterned_array = symbols
        .iter()
        .map(|(_, rust_sym, js_sym)| {
            quote! {
                (#js_sym, #rust_sym),
            }
        })
        .collect::<proc_macro2::TokenStream>();

    let keyword_start = symbols.iter().position(|&(k, ..)| k == Kind::Symbol).unwrap();
    let (keyword_end, ..) = symbols
        .iter()
        .enumerate()
        .filter(|&(_, &(k, ..))| k == Kind::Symbol)
        .next_back()
        .unwrap();

    let keyword_start = keyword_start as u32;
    let keyword_end = keyword_end as u32;

    quote! {
        #consts

        pub const KEYWORD_START: u32 = #keyword_start;
        pub const KEYWORD_END: u32 = #keyword_end;

        pub const PREINTERNED: &[(&str, Symbol)] = &[
            #preinterned_array
        ];
    }
    .into()
}
