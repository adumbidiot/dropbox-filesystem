use anyhow::ensure;
use anyhow::Context;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;
use tracing::info;
use tracing::warn;

/// A filesystem for dropbox
#[derive(Clone)]
pub struct DropboxFileSystem {
    client: dropbox::Client,
    state: Arc<DropboxFileSystemState>,
}

impl DropboxFileSystem {
    /// A Dropbox filesystem
    pub fn new(token: &str) -> Self {
        let tokio_handle = tokio::runtime::Handle::current();
        Self {
            client: dropbox::Client::new(token),
            state: Arc::new(DropboxFileSystemState {
                tokio_handle,
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
    tokio_handle: tokio::runtime::Handle,
    mount_point: std::sync::Mutex<Option<PathBuf>>,
}

impl dokany::FileSystem for DropboxFileSystem {
    fn create_file(
        &self,
        file_name: &[u16],
        _access_mask: dokany::AccessMask,
        is_dir: &mut bool,
    ) -> dokany::sys::NTSTATUS {
        let file_name = PathBuf::from(OsString::from_wide(file_name));
        info!("CreateFile(file_name=\"{}\")", file_name.display());

        if file_name.starts_with("\\System Volume Information")
            || file_name.starts_with("\\$RECYCLE.BIN")
        {
            return dokany::sys::STATUS_NO_SUCH_FILE;
        }

        if file_name == Path::new("/") {
            *is_dir = true;
        }

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
        info!("GetVolumeInformation");

        volume_name.write("DropboxFileSystem");
        *maximum_component_length = 255;
        file_system_name.write("NTFS");

        dokany::sys::STATUS_SUCCESS
    }

    fn find_files(
        &self,
        file_name: &[u16],
        mut fill_find_data: dokany::FillFindData,
    ) -> dokany::sys::NTSTATUS {
        let file_name = OsString::from_wide(file_name);
        info!(
            "FindFiles(file_name=\"{}\")",
            Path::new(&file_name).display()
        );

        let mut file_name = match file_name.into_string() {
            Ok(file_name) => file_name,
            Err(_e) => {
                warn!("Could not convert into unicode");
                return dokany::sys::STATUS_INTERNAL_ERROR;
            }
        };

        if file_name == "\\" {
            file_name = String::new();
        }

        let result =
            self.state
                .tokio_handle
                .block_on(self.client.list_folder(&dropbox::ListFolderArg {
                    path: file_name.replace('\\', "/").into(),
                }));

        let result = match result {
            Ok(result) => result,
            Err(e) => {
                error!("failed to list folder: {e}");
                return dokany::sys::STATUS_INTERNAL_ERROR;
            }
        };

        let mut find_data = dokany::FindData::new();
        for entry in result.entries.iter() {
            let name = entry["name"].as_str().unwrap();
            let size = entry["size"].as_u64().unwrap();

            find_data.set_file_name(name);
            find_data.set_size(size);

            fill_find_data.fill(&mut find_data);
        }

        dokany::sys::STATUS_SUCCESS
    }

    fn mounted(&self, mount_point: &[u16]) -> dokany::sys::NTSTATUS {
        let mount_point = PathBuf::from(OsString::from_wide(mount_point));
        info!("Mounted at \"{}\"", mount_point.display());

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
                info!("Unmounted from \"{}\"", mount_point.display());
            }
            None => {
                error!("Unmounted, missing internal mount point");
            }
        }

        dokany::sys::STATUS_SUCCESS
    }
}
