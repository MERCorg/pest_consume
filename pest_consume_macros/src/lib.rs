#![doc(html_root_url = "https://docs.rs/pest_consume_macros/1.1.0")]

//! This crate contains the code-generation primitives for the [pest_consume](https://docs.rs/pest_consume) crate.
//! See there for documentation.

extern crate proc_macro;

mod declare_parser;
mod make_parser;
mod match_nodes;
mod parser_methods;

use proc_macro::TokenStream;

/// See [pest_consume](https://docs.rs/pest_consume) for documentation.
#[proc_macro_attribute]
pub fn parser(attrs: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(match make_parser::make_parser(attrs, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    })
}

/// One of several `impl` blocks (possibly in different files) implementing rule methods for a
/// parser declared with [`declare_parser!`]. See [pest_consume](https://docs.rs/pest_consume)
/// for documentation.
#[proc_macro_attribute]
pub fn parser_methods(attrs: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(match parser_methods::make_parser_methods(attrs, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    })
}

/// Declares the `Parser` impl for a type whose rule methods are implemented across multiple
/// [`parser_methods`]-annotated impl blocks. See [pest_consume](https://docs.rs/pest_consume)
/// for documentation.
#[proc_macro]
pub fn declare_parser(input: TokenStream) -> TokenStream {
    TokenStream::from(match declare_parser::declare_parser(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    })
}

/// See [pest_consume](https://docs.rs/pest_consume) for documentation.
#[proc_macro]
pub fn match_nodes(input: TokenStream) -> TokenStream {
    TokenStream::from(match match_nodes::match_nodes(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    })
}
