error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }
    foreign_links {
        Io(::std::io::Error) #[doc = "Io"];
        Cargo(Box<::cargo::CargoError>) #[doc = "Cargo"];
    }
}