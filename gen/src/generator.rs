use std::path::Path;
use std::fs::OpenOptions;
use std::io::prelude::*;

use cargo::core::Package;
use cargo::util::Config;

use syntex_syntax::ast;
use syntex_syntax::parse::{ParseSess, parse_crate_from_file};

use quote::{Tokens, ToTokens};

use errors::*;
use options::Options;
use extractor::Extractor;
use builder::Builder;
use format;

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

struct PyClass(Builder);

impl ToTokens for PyClass {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(self.0.build().as_str())
    }
}

pub struct Generated {
    classes: Vec<PyClass>,
}

impl ToTokens for Generated {
    fn to_tokens(&self, tokens: &mut Tokens) {
        for clazz in self.classes.iter() {
            clazz.to_tokens(tokens)
        }
    }
}

impl Generated {
    fn generate(options: &Options) -> Result<Self> {
        let extractor = Extractor::new(&options)?;
        let config = Config::default()?;
        let workspace = extractor.find_workspace(&config)?;
        let packages = extractor.find_packages(&workspace);
        let classes = packages.iter()
            .flat_map(|package| Generated::process_package(package, &extractor))
            .collect();

        Ok(Generated { classes: classes })
    }

    fn process_package(package: &Package, extractor: &Extractor) -> Vec<PyClass> {
        debug!("processing package `{}` @ {}",
               package.name(),
               package.root().to_str().unwrap());

        let parse_session = ParseSess::new();

        extractor.find_targets(package.manifest())
            .iter()
            .flat_map(|target| {
                debug!("parsing `cdylib` crate `{}` @ {}",
                       target.crate_name(),
                       target.src_path().to_str().unwrap());

                match parse_crate_from_file(target.src_path(), &parse_session) {
                    Ok(krate) => {
                        extractor.find_classes(&krate.module.items)
                            .iter()
                            .map(|clazz| {
                                let fields = extractor.find_properties(&clazz)
                                    .unwrap_or(Vec::new())
                                    .iter()
                                    .map(|p| (*p).clone())
                                    .collect::<Vec<ast::StructField>>();
                                let methods = extractor.find_members(&krate.module.items, &clazz)
                                    .iter()
                                    .map(|m| (*m).clone())
                                    .collect::<Vec<ast::ImplItem>>();

                                PyClass(Builder {
                                    ident: clazz.ident,
                                    fields: fields,
                                    methods: methods,
                                })
                            })
                            .collect()
                    }
                    Err(err) => {
                        warn!("fail to parse crate `{}`, {:?}", target.crate_name(), err);

                        Vec::new()
                    }
                }
            })
            .collect()
    }

    pub fn write_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = OpenOptions::new().write(true).truncate(true).create(true).open(path)?;

        self.write(&file)?;

        Ok(())
    }

    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        write!(w, "/* automatically generated by cpython-gen */\n\n")?;

        let mut tokens = Tokens::new();

        self.to_tokens(&mut tokens);

        write!(w, "{}", format::code(tokens.as_str()))?;

        Ok(())
    }
}