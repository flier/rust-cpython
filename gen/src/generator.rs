use std::path::Path;
use std::fs::OpenOptions;
use std::io::prelude::*;

use cargo::core::Package;
use cargo::util::Config;

use syntex_syntax::ast;
use syntex_syntax::ptr::P;
use syntex_syntax::codemap::DUMMY_SP;
use syntex_syntax::parse::{ParseSess, parse_crate_from_file};

use errors::*;
use options::Options;
use extractor::Extractor;

#[derive(Debug, Clone)]
pub struct Generator {
    options: Options,
}

impl Generator {
    pub fn new() -> Self {
        Generator { options: Options::new() }
    }

    pub fn manifest_path<T: Into<String>>(&mut self, path: T) -> &mut Self {
        self.options.manifest_path = Some(path.into());
        self
    }

    pub fn package<T: Into<String>>(&mut self, path: T) -> &mut Self {
        self.options.packages.push(path.into());
        self
    }

    pub fn whitelist_type<T: Into<String>>(&mut self, pattern: T) -> &mut Self {
        self.options.whitelist_types.push(pattern.into());
        self
    }

    pub fn whitelist_field<T: Into<String>>(&mut self, pattern: T) -> &mut Self {
        self.options.whitelist_fields.push(pattern.into());
        self
    }

    pub fn whitelist_method<T: Into<String>>(&mut self, pattern: T) -> &mut Self {
        self.options.whitelist_methods.push(pattern.into());
        self
    }

    pub fn blacklist_type<T: Into<String>>(&mut self, pattern: T) -> &mut Self {
        self.options.blacklist_types.push(pattern.into());
        self
    }

    pub fn blacklist_field<T: Into<String>>(&mut self, pattern: T) -> &mut Self {
        self.options.blacklist_fields.push(pattern.into());
        self
    }

    pub fn blacklist_method<T: Into<String>>(&mut self, pattern: T) -> &mut Self {
        self.options.blacklist_methods.push(pattern.into());
        self
    }

    pub fn generate(&self) -> Result<Generated> {
        Generated::generate(&self.options)
    }
}

#[derive(Debug, Clone)]
pub struct Generated {
    module: ast::Mod,
    attributes: Vec<ast::Attribute>,
}

impl Generated {
    fn generate(options: &Options) -> Result<Self> {
        let extractor = Extractor::new(&options)?;
        let config = Config::default()?;
        let workspace = extractor.find_workspace(&config)?;
        let packages = extractor.find_packages(&workspace);

        let module = ast::Mod {
            inner: DUMMY_SP,
            items: packages.iter()
                .flat_map(|p| Generated::process_package(p, &extractor))
                .flat_map(|p| p)
                .collect(),
        };

        Ok(Generated {
            module: module,
            attributes: Vec::new(),
        })
    }

    fn process_package(package: &Package, extractor: &Extractor) -> Result<Vec<P<ast::Item>>> {
        debug!("processing package `{}` @ {}",
               package.name(),
               package.root().to_str().unwrap());

        let targets = extractor.find_targets(package.manifest());
        let parse_session = ParseSess::new();

        for target in targets {
            debug!("parsing `cdylib` crate `{}` @ {}",
                   target.crate_name(),
                   target.src_path().to_str().unwrap());

            let c = parse_crate_from_file(target.src_path(), &parse_session)?;

            let py_classes = extractor.find_classes(&c.module.items);

            for clazz in py_classes {
                if let Some(py_properties) = extractor.find_properties(&clazz) {}

                let py_members = extractor.find_members(&c.module.items, clazz);
            }
        }

        let mut items = Vec::new();

        Ok(items)
    }

    pub fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = OpenOptions::new().write(true).truncate(true).create(true).open(path)?;

        self.write(&file)?;

        Ok(())
    }

    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        write!(w, "/* automatically generated by cpython-gen */\n\n")?;

        Ok(())
    }
}