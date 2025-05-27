extern crate proc_macro;

use proc_macro::TokenStream;

mod macros;
mod util;
use macros::*;


#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run_entry(args.into(), item.into()).into()
}