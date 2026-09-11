use std::path::{Path, PathBuf};

pub mod game;
pub mod main;

fn expand_vars(path: &Path) -> anyhow::Result<Option<PathBuf>> {
    Ok(if let Some(path) = path.to_str() {
        Some(subst::substitute(path, &subst::Env)?.into())
    } else {
        None
    })
}
