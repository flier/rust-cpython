use syntex_errors::DiagnosticBuilder;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }
    foreign_links {
        Io(::std::io::Error) #[doc = "Io"];
        Cargo(Box<::cargo::CargoError>) #[doc = "Cargo"];
    }
    errors {
        Parse
    }
}

impl<'a> From<DiagnosticBuilder<'a>> for Error {
    fn from(mut diagnostic: DiagnosticBuilder<'a>) -> Self {
        diagnostic.emit();
        ErrorKind::Parse.into()
    }
}