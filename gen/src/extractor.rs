use cargo::core::{Workspace, Manifest, Package, Target, TargetKind};
use cargo::util::Config;
use cargo::util::important_paths::find_root_manifest_for_wd;

use syntex_syntax::ast;
use syntex_syntax::symbol::Symbol;
use syntex_syntax::symbol::keywords;
use syntex_syntax::ptr::P;

use errors::Result;
use options::Options;

pub struct Extractor {
    manifest_path: Option<String>,
    packages: Vec<String>,
    lib_kind: &'static str,
    derive_attr: Symbol,
}

impl Extractor {
    pub fn new(options: &Options) -> Self {
        Extractor {
            manifest_path: options.manifest_path.clone(),
            packages: options.packages.clone(),
            lib_kind: "cdylib",
            derive_attr: Symbol::intern("PyClass"),
        }
    }

    pub fn find_workspace<'a>(&self, config: &'a Config) -> Result<Workspace<'a>> {
        let root = find_root_manifest_for_wd(self.manifest_path.clone(), config.cwd())?;
        let workspace = Workspace::new(&root, &config)?;

        debug!("found workspace @ `{}` with members: [{}]",
               workspace.root().to_str().unwrap(),
               workspace.members().map(|member| member.name()).collect::<Vec<&str>>().join(","));

        Ok(workspace)
    }

    pub fn find_packages<'a>(&self, workspace: &'a Workspace) -> Vec<&'a Package> {
        if self.packages.is_empty() {
            vec![workspace.current().unwrap()]
        } else {
            workspace.members()
                .filter(|ref p| self.packages.contains(&String::from(p.name())))
                .collect()
        }
    }

    pub fn find_targets<'a>(&self, manifest: &'a Manifest) -> Vec<&'a Target> {
        let targets = manifest.targets()
            .iter()
            .filter(|ref target| match *target.kind() {
                TargetKind::Lib(ref kinds) => {
                    kinds.iter().any(|kind| kind.crate_type() == self.lib_kind)
                }
                _ => false,
            })
            .collect::<Vec<&Target>>();

        debug!("found {} targets to `{}`: [{}]",
               targets.len(),
               self.lib_kind,
               targets.iter().map(|target| target.crate_name()).collect::<Vec<String>>().join(","));

        targets
    }

    pub fn find_classes<'a>(&self, items: &'a Vec<P<ast::Item>>) -> Vec<&'a P<ast::Item>> {
        let classes = items.iter()
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
                                    item.name == self.derive_attr
                                }
                                ast::NestedMetaItemKind::Literal(_) => false,
                            })
                        }
                        _ => false,
                    }
                })
            })
            .collect::<Vec<&P<ast::Item>>>();

        debug!("found {} classes with derive({}) attribute: [{}]",
               classes.len(),
               self.derive_attr,
               classes.iter()
                   .map(|clazz| clazz.ident.to_string())
                   .collect::<Vec<String>>()
                   .join(","));

        classes
    }

    pub fn find_properties<'a>(&self, item: &'a P<ast::Item>) -> Option<Vec<&'a ast::StructField>> {
        match item.node {
            ast::ItemKind::Struct(ref data, _) |
            ast::ItemKind::Union(ref data, _) => {
                match *data {
                    ast::VariantData::Struct(ref fields, _) |
                    ast::VariantData::Tuple(ref fields, _) => {
                        let properties = fields.iter()
                            .filter(|field| field.vis == ast::Visibility::Public)
                            .collect::<Vec<&ast::StructField>>();

                        debug!("found {} properties: [{}]",
                               properties.len(),
                               properties.iter()
                                   .map(|field| {
                                       field.ident
                                           .map(|ident| ident.name)
                                           .unwrap_or(keywords::Invalid.name())
                                           .to_string()
                                   })
                                   .collect::<Vec<String>>()
                                   .join(","));

                        Some(properties)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn find_members(&self,
                        items: &Vec<P<ast::Item>>,
                        clazz: &P<ast::Item>)
                        -> Vec<ast::ImplItem> {
        let members = items.iter()
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
            .collect::<Vec<ast::ImplItem>>();

        debug!("found {} members from class {}: [{}]",
               members.len(),
               clazz.ident,
               members.iter()
                   .map(|member| member.ident.to_string())
                   .collect::<Vec<String>>()
                   .join(","));

        members
    }
}