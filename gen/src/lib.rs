#![recursion_limit = "128"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate regex;
extern crate cargo;
extern crate syntex_syntax;
extern crate syntex_pos;
extern crate syntex_errors;
#[macro_use]
extern crate quote;
extern crate aster;
extern crate rustfmt;

mod errors;
mod options;
mod builder;
mod extractor;
mod generator;
pub mod format;

pub use builder::Builder;
pub use generator::Generator;
