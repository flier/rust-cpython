#[macro_use]
extern crate log;
extern crate env_logger;
extern crate cpython_gen;

use std::env;
use std::path::Path;

use cpython_gen::Generator;

fn main() {
    let _ = env_logger::init().unwrap();

    let out_dir = env::var("OUT_DIR")
        .unwrap_or(String::from(env::current_dir().unwrap().to_str().unwrap()));
    let dest_path = Path::new(&out_dir).join("generated.rs");

    debug!("generate Python wrapper to {}", dest_path.to_str().unwrap());

    Generator::new().generate().unwrap().write_to(&dest_path).unwrap();
}