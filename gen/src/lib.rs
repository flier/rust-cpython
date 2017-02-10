#![recursion_limit = "128"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate syntex_syntax;
extern crate syntex_errors;
extern crate cargo;
extern crate rustfmt;

mod errors;
mod builder;
mod extractor;
mod generator;
pub mod format;

pub use builder::Builder;
pub use generator::Generator;
