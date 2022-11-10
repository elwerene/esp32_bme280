mod fathandle;

use anyhow::Result;
use fathandle::FATHandle;
use once_cell::sync::{Lazy, OnceCell};
use std::path::PathBuf;

static BASE_PATH: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("/flash"));
static FAT_HANDLE: OnceCell<FATHandle> = OnceCell::new();

pub fn init() -> Result<()> {
    let handle = FATHandle::new(&*BASE_PATH, "data", 10, true)?;

    FAT_HANDLE
        .set(handle)
        .map_err(|_| anyhow::format_err!("FAT_HANDLE was already set"))?;

    log::debug!("Initialized fs");

    Ok(())
}

fn path(file: &'static str) -> PathBuf {
    BASE_PATH.join(file)
}

pub fn create_and_open_file(file: &'static str) -> Result<std::fs::File> {
    let path = path(file);

    if let Err(_err) = std::fs::metadata(&path) {
        log::error!("Recreating {file:?} file at {}", path.display());
        std::fs::File::create(&path)?;
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;
    Ok(file)
}
