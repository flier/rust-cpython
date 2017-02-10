use cargo::core::{Workspace, Manifest, Package, Target, TargetKind};
use cargo::util::Config;
use cargo::util::important_paths::find_root_manifest_for_wd;

use syntex_syntax::ast;
use syntex_syntax::symbol::Symbol;
use syntex_syntax::symbol::keywords;
use syntex_syntax::ptr::P;

use errors::Result;

pub struct Extractor {}

impl Extractor {
    pub fn find_workspace(manifest_path: Option<String>, config: &Config) -> Result<Workspace> {
        let root = find_root_manifest_for_wd(manifest_path, config.cwd())?;
        let workspace = Workspace::new(&root, &config)?;

        debug!("found workspace @ `{}` with members: [{}]",
               workspace.root().to_str().unwrap(),
               workspace.members().map(|member| member.name()).collect::<Vec<&str>>().join(","));

        Ok(workspace)
    }

    pub fn find_packages<'a>(workspace: &'a Workspace, packages: &Vec<String>) -> Vec<&'a Package> {
        if packages.is_empty() {
            vec![workspace.current().unwrap()]
        } else {
            workspace.members()
                .filter(|ref p| packages.contains(&String::from(p.name())))
                .collect()
        }
    }

    pub fn find_targets<'a>(manifest: &'a Manifest, lib_kind: &str) -> Vec<&'a Target> {
        let targets = manifest.targets()
            .iter()
            .filter(|ref target| match *target.kind() {
                TargetKind::Lib(ref kinds) => {
                    kinds.iter().any(|kind| kind.crate_type() == lib_kind)
                }
                _ => false,
            })
            .collect::<Vec<&Target>>();

        debug!("found {} targets to `{}`: [{}]",
               targets.len(),
               lib_kind,
               targets.iter().map(|target| target.crate_name()).collect::<Vec<String>>().join(","));

        targets
    }

    pub fn find_classes(items: &Vec<P<ast::Item>>, derive_attr: Symbol) -> Vec<&P<ast::Item>> {
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
                                    item.name == derive_attr
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
               derive_attr,
               classes.iter()
                   .map(|clazz| clazz.ident.to_string())
                   .collect::<Vec<String>>()
                   .join(","));

        classes
    }

    pub fn find_fields(item: &P<ast::Item>) -> Option<&Vec<ast::StructField>> {
        match item.node {
            ast::ItemKind::Struct(ref data, _) |
            ast::ItemKind::Union(ref data, _) => {
                match *data {
                    ast::VariantData::Struct(ref fields, _) |
                    ast::VariantData::Tuple(ref fields, _) => {
                        debug!("found {} fields: [{}]",
                               fields.len(),
                               fields.iter()
                                   .map(|field| {
                                       field.ident
                                           .map(|ident| ident.name)
                                           .unwrap_or(keywords::Invalid.name())
                                           .to_string()
                                   })
                                   .collect::<Vec<String>>()
                                   .join(","));

                        Some(fields)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn find_members(items: &Vec<P<ast::Item>>, clazz: &P<ast::Item>) -> Vec<ast::ImplItem> {
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