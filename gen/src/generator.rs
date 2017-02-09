use std::path::Path;
use std::fs::OpenOptions;
use std::io::prelude::*;

use cargo::core::{Workspace, Manifest, Package, Target, TargetKind, LibKind};
use cargo::util::Config;
use cargo::util::important_paths::find_root_manifest_for_wd;

use syntex_syntax::ast;
use syntex_syntax::symbol::Symbol;
use syntex_syntax::ptr::P;
use syntex_syntax::codemap::DUMMY_SP;
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

        let targets = Generated::find_targets(package.manifest(),
                                              LibKind::Other(String::from("cdylib")));

        debug!("found {} targets to `cdylib`: [{}]",
               targets.len(),
               targets.iter().map(|target| target.crate_name()).collect::<Vec<String>>().join(","));

        let parse_session = ParseSess::new();

        for target in targets {
            debug!("parsing `cdylib` crate `{}` @ {}",
                   target.crate_name(),
                   target.src_path().to_str().unwrap());

            let c = parse_crate_from_file(target.src_path(), &parse_session)?;

            let py_classes = Generated::find_classes(&c.module.items, Symbol::intern("PyClass"));

            debug!("found {} classes with `PyClass`: [{}]",
                   py_classes.len(),
                   py_classes.iter()
                       .map(|clazz| clazz.ident.to_string())
                       .collect::<Vec<String>>()
                       .join(","));

            for clazz in py_classes {
                let py_members = Generated::find_members(&c.module.items, clazz);

                debug!("found {} members: [{}]",
                       py_members.len(),
                       py_members.iter()
                           .map(|member| member.ident.to_string())
                           .collect::<Vec<String>>()
                           .join(","));
            }
        }

        let mut items = Vec::new();

        Ok(items)
    }

    fn find_targets(manifest: &Manifest, lib_kind: LibKind) -> Vec<&Target> {
        manifest.targets()
            .iter()
            .filter(|ref target| match *target.kind() {
                TargetKind::Lib(ref kinds) => kinds.iter().any(|kind| *kind == lib_kind),
                _ => false,
            })
            .collect()
    }

    fn find_classes(items: &Vec<P<ast::Item>>, derive_attr: Symbol) -> Vec<&P<ast::Item>> {
        items.iter()
            .filter(|item| match item.node {
                ast::ItemKind::Struct(..) => true,
                _ => false,
            })
            .filter(|item| {
                item.attrs.iter().any(|ref attr| {
                    attr.value.name == Symbol::intern("derive") &&
                    match attr.value.node {
                        ast::MetaItemKind::List(ref items) => {
                            items.iter().any(|item| match item.node {
                                ast::NestedMetaItemKind::MetaItem(ref item) => {
                                    item.name == derive_attr
                                }
                                ast::NestedMetaItemKind::Literal(_) => false,
                            })
                        }
                        _ => false,
                    }
                })
            })
            .collect()
    }

    fn find_members(items: &Vec<P<ast::Item>>, clazz: &P<ast::Item>) -> Vec<ast::ImplItem> {
        items.iter()
            .flat_map(|item| match item.node {
                ast::ItemKind::Impl(_, _, _, None, ref ty, ref members) => {
                    if ty.id == clazz.id {
                        Some(members)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .flat_map(|members| members.clone())
            .collect()
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