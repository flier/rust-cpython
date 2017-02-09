use std::path::Path;
use std::fs::OpenOptions;
use std::io::prelude::*;

use cargo::core::{Workspace, Package, Target, TargetKind, LibKind};
use cargo::util::Config;
use cargo::util::important_paths::find_root_manifest_for_wd;

use syntex_syntax::ast;
use syntex_syntax::symbol::Symbol;
use syntex_syntax::ptr::P;
use syntex_syntax::codemap::{Spanned, DUMMY_SP};
use syntex_syntax::parse::{ParseSess, parse_crate_from_file};

use quote;

use super::errors::*;

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct GenerateOptions {
    manifest_path: Option<String>,
    packages: Vec<String>,
}

impl GenerateOptions {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for GenerateOptions {
    fn default() -> Self {
        GenerateOptions {
            manifest_path: None,
            packages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Generator {
    options: GenerateOptions,
}

impl Generator {
    pub fn new() -> Self {
        Generator { options: Default::default() }
    }

    pub fn manifest_path<T: Into<String>>(&mut self, path: T) -> &mut Self {
        self.options.manifest_path = Some(path.into());
        self
    }

    pub fn package<T: Into<String>>(&mut self, path: T) -> &mut Self {
        self.options.packages.push(path.into());
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
    fn generate(options: &GenerateOptions) -> Result<Self> {
        let config = Config::default()?;
        let root = find_root_manifest_for_wd(options.manifest_path.clone(), config.cwd())?;
        let workspace = Workspace::new(&root, &config)?;

        debug!("found workspace @ `{}` with members: [{}]",
               workspace.root().to_str().unwrap(),
               workspace.members().map(|member| member.name()).collect::<Vec<&str>>().join(","));

        let packages = if options.packages.is_empty() {
            vec![workspace.current()?]
        } else {
            workspace.members()
                .filter(|ref p| options.packages.contains(&String::from(p.name())))
                .collect()
        };
        let items =
            packages.iter().flat_map(|p| Generated::process_package(p)).flat_map(|p| p).collect();

        let module = ast::Mod {
            inner: DUMMY_SP,
            items: items,
        };

        Ok(Generated {
            module: module,
            attributes: Vec::new(),
        })
    }

    fn process_package(package: &Package) -> Result<Vec<P<ast::Item>>> {
        debug!("processing package `{}` @ {}",
               package.name(),
               package.root().to_str().unwrap());

        let manifest = package.manifest();
        let targets = manifest.targets()
            .iter()
            .filter(|ref target| match *target.kind() {
                TargetKind::Lib(ref kinds) => {
                    kinds.iter().any(|kind| *kind == LibKind::Other(String::from("cdylib")))
                }
                _ => false,
            });

        let parse_session = ParseSess::new();

        for target in targets {
            debug!("parsing `cdylib` crate `{}` @ {}",
                   target.crate_name(),
                   target.src_path().to_str().unwrap());

            let c = parse_crate_from_file(target.src_path(), &parse_session)?;

            let structures = Generated::find_py_class(&c.module.items);

            debug!("found {} PyClass: {:?}", structures.len(), structures);
        }

        let mut items = Vec::new();

        Ok(items)
    }

    fn find_py_class(items: &Vec<P<ast::Item>>) -> Vec<&P<ast::Item>> {
        items.iter()
            .filter(|item| match item.node {
                ast::ItemKind::Struct(..) => true,
                _ => false,
            })
            .filter(|item| {
                item.attrs.iter().any(|ref attr| {
                    attr.value.name == Symbol::intern("derive") &&
                    match attr.value.node {
                        // `derive(..)` as in `#[derive(..)]`
                        ast::MetaItemKind::List(ref items) => {
                            items.iter().any(|item| match item.node {
                                ast::NestedMetaItemKind::MetaItem(ref item) => {
                                    item.name == Symbol::intern("PyClass")
                                }
                                ast::NestedMetaItemKind::Literal(_) => false,
                            })
                        }
                        // `test` as in `#[test]`
                        ast::MetaItemKind::Word => false,
                        // `feature = "foo"` as in `#[feature = "foo"]`
                        ast::MetaItemKind::NameValue(_) => false,
                    }
                })
            })
            .collect::<Vec<&P<ast::Item>>>()
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