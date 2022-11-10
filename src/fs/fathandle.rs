use anyhow::Result;
use esp_idf_sys::*;
use std::ffi::CString;
use std::path::PathBuf;

pub struct FATHandle {
    wl_handle: wl_handle_t,
    base_path: PathBuf,
}

impl FATHandle {
    /// Creates a new `FATHandle` that registers all operations on the path `base_path` to the
    /// `partition_label`.
    ///
    /// The returned `FATHandle` will unmount the given partition when it's dropped.
    pub fn new<P, L>(
        base_path: P,
        partition_label: L,
        max_files: usize,
        format_if_mount_failed: bool,
    ) -> Result<Self>
    where
        P: Into<PathBuf>,
        L: AsRef<str>,
    {
        let base_path = base_path.into();
        let pl = CString::new(partition_label.as_ref())?;
        let bp = CString::new(base_path.to_str().expect("'to_str' on 'base_path' failed"))?;
        let mut wl = WL_INVALID_HANDLE;

        let conf = esp_vfs_fat_mount_config_t {
            max_files: max_files as _,
            format_if_mount_failed,
            allocation_unit_size: CONFIG_WL_SECTOR_SIZE,
        };

        esp!(unsafe {
            esp_vfs_fat_spiflash_mount(bp.as_ptr(), pl.as_ptr(), &conf as _, &mut wl as _)
        })?;

        Ok(Self {
            wl_handle: wl,
            base_path,
        })
    }
}

impl Drop for FATHandle {
    // the FAT partition is mounted as long as the FATHandle isn't dropped
    fn drop(&mut self) {
        let bp = CString::new(
            self.base_path
                .to_str()
                .expect("base_path was not valid utf8"),
        )
        .expect("base_path was faulty");
        esp!(unsafe { esp_vfs_fat_spiflash_unmount(bp.as_ptr(), self.wl_handle as _) }).unwrap();
    }
}
