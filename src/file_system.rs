use anyhow::ensure;
use anyhow::Context;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::Arc;

/// A filesystem for dropbox
#[derive(Clone)]
pub struct DropboxFileSystem {
    state: Arc<DropboxFileSystemState>,
}

impl DropboxFileSystem {
    /// A Dropbox filesystem
    pub fn new() -> Self {
        Self {
            state: Arc::new(DropboxFileSystemState {
                mount_point: std::sync::Mutex::new(None),
            }),
        }
    }

    pub fn unmount(&self) -> anyhow::Result<()> {
        let mount_point = self
            .state
            .mount_point
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .context("missing mount point")?;

        ensure!(
            dokany::remove_mount_point(&mount_point),
            "failed to unmount"
        );

        Ok(())
    }
}

struct DropboxFileSystemState {
    mount_point: std::sync::Mutex<Option<PathBuf>>,
}

impl dokany::Filesystem for DropboxFileSystem {
    fn create_file(
        &self,
        file_name: &[u16],
        _access_mask: dokany::AccessMask,
    ) -> dokany::sys::NTSTATUS {
        let file_name = PathBuf::from(OsString::from_wide(file_name));
        println!("CreateFile(file_name=\"{}\")", file_name.display());

        dokany::sys::STATUS_SUCCESS
    }

    fn get_volume_information(
        &self,
        mut volume_name: dokany::WriteWideCStringCell<'_>,
        _volume_serial_number: &mut u32,
        maximum_component_length: &mut u32,
        _file_system_flags: &mut dokany::FileSystemFlags,
        mut file_system_name: dokany::WriteWideCStringCell<'_>,
    ) -> dokany::sys::NTSTATUS {
        println!("GetVolumeInformation");

        volume_name.write("DropboxFileSystem");
        *maximum_component_length = 255;
        file_system_name.write("NTFS");

        dokany::sys::STATUS_SUCCESS
    }

    fn find_files(&self, file_name: &[u16]) -> dokany::sys::NTSTATUS {
        let file_name = PathBuf::from(OsString::from_wide(file_name));
        println!("FindFiles(file_name=\"{}\")", file_name.display());

        dokany::sys::STATUS_SUCCESS
    }

    fn mounted(&self, mount_point: &[u16]) -> dokany::sys::NTSTATUS {
        let mount_point = PathBuf::from(OsString::from_wide(mount_point));
        println!("Mounted at \"{}\"", mount_point.display());

        *self
            .state
            .mount_point
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(mount_point);

        dokany::sys::STATUS_SUCCESS
    }

    fn unmounted(&self) -> dokany::sys::NTSTATUS {
        let mount_point = self
            .state
            .mount_point
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();

        match mount_point {
            Some(mount_point) => {
                println!("Unmounted from \"{}\"", mount_point.display());
            }
            None => {
                println!("Unmounted, missing internal mount point");
            }
        }

        dokany::sys::STATUS_SUCCESS
    }
}
