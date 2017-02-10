#![recursion_limit = "128"]

#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate lazy_static;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate cpython_gen;

use proc_macro::TokenStream;

use cpython_gen::format;

lazy_static! {
    static ref LOGGER: () = env_logger::init().unwrap();
}

#[proc_macro_derive(PyClass)]
pub fn py_class(input: TokenStream) -> TokenStream {
    let _ = *LOGGER;

    let ast = syn::parse_macro_input(&input.to_string()).unwrap();

    let class_name = ast.ident;
    let func_name = quote::Ident::new(format!("test_{}", class_name).to_lowercase());

    let tokens = quote!{
        #[test]
        fn #func_name() {
            use cpython::Python;

            let gil = Python::acquire_gil();
            let py = gil.python();

            assert!(!py.get_type::<#class_name>().as_type_ptr().is_null());
        }
    };

    debug!("generated code\n{}", format::code(tokens.as_str()));

    tokens.parse().unwrap()
}
