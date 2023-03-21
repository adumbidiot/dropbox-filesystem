use std::path::Path;

/// Config
#[derive(Debug, serde::Deserialize)]
pub struct Config {}

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
