extern crate inflector;
extern crate tokio;
extern crate webext_parser;

use inflector::Inflector;
use std::path::Path;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut namespaces = vec![];

    let path_ext_dir = Path::new("src").join("ext");
    if !path_ext_dir.exists() {
        fs::create_dir(&path_ext_dir).await?;
    }

    for (space, url) in webext_parser::api_pages().await?.iter() {
        /* if space != "windows" {
            continue;
        } */
        let namespace = webext_parser::parse_apis(&space, &url).await?;
        let name = namespace.name().to_snake_case();
        let path = Path::new("src").join("ext").join(format!("{}.rs", &name));
        fs::write(&path, "\n").await?;
        namespaces.push(name)
    }

    let path_root = Path::new("src").join("ext.rs");
    fs::write(
        &path_root,
        namespaces
            .iter()
            .map(|s| format!("pub mod {};", s))
            .fold(String::new(), |acc, x| format!("{}{}\n", acc, x)),
    )
    .await?;

    Ok(())
}
