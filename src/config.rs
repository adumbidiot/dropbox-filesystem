use std::path::Path;
use std::path::PathBuf;

/// Config
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// The suggested mount point
    #[serde(rename = "mount-point")]
    pub mount_point: PathBuf,
}

impl Config {
    /// Load a config
    pub fn load<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let data = std::fs::read_to_string(path.as_ref())?;
        Ok(toml::from_str(&data)?)
    }
}
