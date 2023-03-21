mod config;
mod file_system;

use self::config::Config;
use self::file_system::DropboxFileSystem;
use anyhow::Context;
use tracing::info;

fn main() -> anyhow::Result<()> {
    let config = Config::load("config.toml").context("failed to load config.toml")?;

    tracing_subscriber::fmt::init();

    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    tokio_rt.block_on(async_main(config))?;

    Ok(())
}

async fn async_main(_config: Config) -> anyhow::Result<()> {
    let dokany_version = dokany::version();
    let dokany_driver_version = dokany::driver_version();
    info!("Dokany Version: {dokany_version}");
    info!("Dokany Driver Version: {dokany_driver_version}");

    let file_system = DropboxFileSystem::new();

    let mut file_system_handle: tokio::task::JoinHandle<anyhow::Result<()>> = {
        let file_system = file_system.clone();

        tokio::task::spawn_blocking(|| {
            let mut options = dokany::Options::new();
            options.set_version(209);
            options.set_mount_point("M");
            options.set_option_flags(dokany::OptionFlags::MOUNT_MANAGER);

            dokany::main(options, file_system)?;

            Ok(())
        })
    };

    let mut ctrl_c_handle = tokio::spawn(tokio::signal::ctrl_c());

    tokio::select! {
        result = &mut ctrl_c_handle => {
            result??;
            file_system.unmount()?;
            file_system_handle.await??;
        },
        result = &mut file_system_handle => {
            result??;
            ctrl_c_handle.abort();
        },
    };

    Ok(())
}
