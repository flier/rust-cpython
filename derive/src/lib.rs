#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
extern crate cpython_gen;

use proc_macro::TokenStream;

use cpython_gen::{Builder, format};

lazy_static! {
    static ref LOGGER: () = env_logger::init().unwrap();
}

#[proc_macro_derive(PyClass)]
pub fn py_class(input: TokenStream) -> TokenStream {
    let _ = *LOGGER;

    let source = input.to_string();

    debug!("parsing source\n{}", source);

    let tokens = Builder::parse(&source).unwrap().build();

    debug!("generated code\n{}", format::code(tokens.as_str()));

    tokens.parse().unwrap()
}
