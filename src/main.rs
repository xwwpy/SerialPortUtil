// #![cfg_attr(windows, windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()?;
    let _guard = runtime.enter();
    ww_ui::run();
    Ok(())
}
