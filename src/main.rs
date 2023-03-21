fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    tokio_rt.block_on(async_main())?;

    Ok(())
}

async fn async_main() -> anyhow::Result<()> {
    println!("Hello, world!");

    Ok(())
}
