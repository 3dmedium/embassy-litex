use std::str::FromStr;

use darling::export::NestedMeta;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ReturnType, Type};


use crate::util::*;

#[derive(Debug, FromMeta, Default)]
struct Args {
    #[darling(default)]
    entry: Option<String>,
}


pub fn run_entry(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut errors = TokenStream::new();

    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let f: ItemFn = match syn::parse2(item.clone()) {
        Ok(x) => x,
        Err(e) => return token_stream_with_error(item, e),
    };

    let args = match NestedMeta::parse_meta_list(args) {
        Ok(x) => x,
        Err(e) => return token_stream_with_error(item, e),
    };

    let args = match Args::from_list(&args) {
        Ok(x) => x,
        Err(e) => {
            errors.extend(e.write_errors());
            Args::default()
        }
    };

    let fargs = f.sig.inputs.clone();

    if f.sig.asyncness.is_none() {
        error(&mut errors, &f.sig, "main function must be async");
    }
    if !f.sig.generics.params.is_empty() {
        error(&mut errors, &f.sig, "main function must not be generic");
    }
    if !f.sig.generics.where_clause.is_none() {
        error(&mut errors, &f.sig, "main function must not have `where` clauses");
    }
    if !f.sig.abi.is_none() {
        error(&mut errors, &f.sig, "main function must not have an ABI qualifier");
    }
    if !f.sig.variadic.is_none() {
        error(&mut errors, &f.sig, "main function must not be variadic");
    }
    match &f.sig.output {
        ReturnType::Default => {}
        ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => {}
            Type::Never(_) => {}
            _ => error(
                &mut errors,
                &f.sig,
                "main function must either not return a value, return `()` or return `!`",
            ),
        },
    }

    //if fargs.len() != 1 {
    //    error(&mut errors, &f.sig, "main function must have 1 argument: the spawner.");
    //}

    let entry = match args.entry.as_deref().or(Some("riscv_rt::entry")) {
        None => TokenStream::new(),
        Some(x) => match TokenStream::from_str(x) {
            Ok(x) => quote!(#[#x]),
            Err(e) => {
                error(&mut errors, &f.sig, e);
                TokenStream::new()
            }
        },
    };

    let f_body = f.body;
    let out = &f.sig.output;

    let (main_ret, mut main_body) =  (
            quote!(!),
            quote! {

                    
                    EXECUTOR_LIST[15].start().must_spawn(__embassy_main());
                    interrupt::initialize();

                    loop {
                        unsafe {
                            core::arch::asm!("wfi");
                        }
                    }
                
                    loop {}
            
            },
    );

    let mut main_attrs = TokenStream::new();
    for attr in f.attrs {
        main_attrs.extend(quote!(#attr));
    }

    if !errors.is_empty() {
        main_body = quote! {loop{}};
    }

    let result = quote! {
        #[::embassy_executor::task()]
        #[allow(clippy::future_not_send)]
        async fn __embassy_main(#fargs) #out {
            #f_body
        }

        #entry
        #main_attrs
        fn main() -> #main_ret {
            #main_body
        }

        #errors
    };

    result
}

