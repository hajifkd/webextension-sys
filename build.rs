extern crate heck;
extern crate tokio;
extern crate webext_parser;

use heck::SnakeCase;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::prelude::*;
use webext_parser::api::{Namespace, TypeKind};

fn get_dir() -> PathBuf {
    Path::new("src").join("ext")
}

async fn create_subnamespace(names: &[&str]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = get_dir();
    for i in 0..names.len() {
        if i == names.len() - 1 {
            path.push(format!("{}.rs", names[i]))
        } else {
            let mut path_mod_dir = get_dir();
            for j in 0..=i {
                path_mod_dir.push(names[j]);
            }

            if !path_mod_dir.exists() {
                fs::create_dir(&path_mod_dir).await?;
            }

            let mut path_mod_file = get_dir();
            for j in 0..i {
                path_mod_file.push(names[j]);
            }
            path_mod_file.push(format!("{}.rs", names[i]));

            let mut file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path_mod_file)
                .await?;

            file.write_all(format!("pub mod {};\n", names[i + 1]).as_bytes())
                .await?;

            path.push(names[i])
        }
    }

    Ok(path)
}

fn construct_module(namespace: &Namespace) -> String {
    let mut outside = String::new();
    let mut result = String::new();
    result.push_str(
        r#"use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use crate::ext as chrome;
#[allow(unused_imports)]
use js_sys::*;

#[wasm_bindgen]
extern "C" {
"#,
    );

    for js_type in namespace.types().iter() {
        match js_type.kind() {
            TypeKind::Enum => {
                outside.push_str("pub type ");
                outside.push_str(js_type.name());
                outside.push_str(" = String;\n");
            }

            TypeKind::Data => {
                outside.push_str("pub type ");
                outside.push_str(js_type.name());
                outside.push_str(" = JsValue;\n");
            }

            TypeKind::Struct {
                ref elements,
                ref optional_elements,
                ref methods,
            } => {
                result.push_str("    type ");
                result.push_str(js_type.name());
                result.push_str(";\n");
                for element in elements.iter() {
                    result.push_str(&format!(
                        r#"    #[wasm_bindgen(structural, catch, method, getter, js_class = "{}", js_name = {})]"#,
                        js_type.name(), element.name()
                    ));
                    result.push_str("\n");
                    result.push_str("    fn get_");
                    result.push_str(&element.name().to_snake_case());
                    result.push_str("(this: &");
                    result.push_str(js_type.name());
                    result.push_str(") -> Result<");
                    if element.is_array() {
                        result.push_str("Box<[JsValue]>");
                    } else {
                        result.push_str(element.rustify_type());
                    }
                    result.push_str(", JsValue>;\n");
                }
            }
        }
    }

    result.push_str("}\n\n");
    result.push_str(&outside);

    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut namespaces = HashSet::new();

    let path_ext_dir = get_dir();
    if path_ext_dir.exists() {
        fs::remove_dir_all(&path_ext_dir).await?;
    }
    fs::create_dir(&path_ext_dir).await?;

    for (space, url) in webext_parser::api_pages().await?.iter() {
        /* if space != "windows" {
            continue;
        } */
        let namespace = webext_parser::parse_apis(&space, &url).await?;
        let name = namespace.name().to_snake_case();
        let path = if name.contains('.') {
            let names = name.split('.').collect::<Vec<_>>();
            namespaces.insert(names[0].to_owned());
            create_subnamespace(&names).await?
        } else {
            let path = get_dir().join(format!("{}.rs", &name));
            namespaces.insert(name);
            path
        };

        fs::write(&path, &construct_module(&namespace)).await?;
    }

    let path_root = Path::new("src").join("ext.rs");
    fs::write(
        &path_root,
        namespaces
            .iter()
            .map(|s| format!("#[cfg(feature=\"{}\")]\npub mod {};", &s, &s))
            .fold(String::new(), |acc, x| format!("{}{}\n", acc, x)),
    )
    .await?;

    Ok(())
}
