use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use impls::*;

mod impls;

#[proc_macro_derive(Execute, attributes(handler))]
pub fn derive_execute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    execute_derive_macro(input).into()
}

#[proc_macro_derive(ExecuteWith, attributes(handler, execute_with))]
pub fn derive_execute_with(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    execute_with_derive_macro(input).into()
}

#[proc_macro_derive(ExecuteWithMut, attributes(handler, execute_with))]
pub fn derive_execute_with_mut(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    execute_with_mut_derive_macro(input).into()
}
