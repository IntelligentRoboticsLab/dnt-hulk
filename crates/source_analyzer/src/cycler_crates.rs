use std::path::{Path, PathBuf};

use color_eyre::{eyre::WrapErr, Result};
use glob::glob;
use syn::Item;

use crate::parse::parse_rust_file;

pub fn cycler_crates_from_crates_directory(
    crates_directory: impl AsRef<Path>,
) -> Result<Vec<PathBuf>> {
    glob(
        crates_directory
            .as_ref()
            .join("*/src/lib.rs")
            .to_str()
            .unwrap(),
    )
    .wrap_err_with(|| {
        format!(
            "failed to find lib.rs files from crates directory {:?}",
            crates_directory.as_ref()
        )
    })?
    .filter_map(|file_path| {
        let file_path = match file_path {
            Ok(file_path) => file_path,
            Err(error) => return Some(Err(error.into())),
        };
        let file = match parse_rust_file(&file_path) {
            Ok(file) => file,
            Err(error) => return Some(Err(error)),
        };
        let has_cycler_instance = file.items.into_iter().any(|item| match item {
            Item::Enum(enum_item) => enum_item.ident == "CyclerInstance",
            _ => false,
        });
        has_cycler_instance
            .then(|| {
                file_path
                    .parent()
                    .and_then(|source_directory| source_directory.parent())
                    .map(|crate_directory| Ok(crate_directory.to_path_buf()))
            })
            .flatten()
    })
    .collect()
}
