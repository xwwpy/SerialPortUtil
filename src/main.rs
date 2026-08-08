fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread().build()?;
    let _guard = runtime.enter();
    ww_ui::run();
    Ok(())
}
