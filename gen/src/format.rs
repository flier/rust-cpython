use std::io::Cursor;

use rustfmt::{Input, format_input};
use rustfmt::config::{Config, WriteMode};

pub fn code(src: &str) -> String {
    let mut config: Config = Default::default();

    config.write_mode = WriteMode::Plain;

    let mut cur = Cursor::new(Vec::new());

    format_input(Input::Text(String::from(src)), &config, Some(&mut cur)).unwrap();

    String::from_utf8(cur.into_inner()).unwrap()
}
