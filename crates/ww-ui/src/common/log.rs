use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::ui_config;

pub fn init() {
    let config = ui_config::get();
    // ── 文件输出层（非阻塞，自动轮转） ──
    let file_appender = tracing_appender::rolling::daily(
        config.get_log_config().get_log_dir(),
        "serial-port-util.log",
    );

    let file_layer = fmt::layer()
        .with_ansi(false) // 文件不需要 ANSI 颜色
        .with_target(true) // 显示模块路径
        .with_thread_ids(true) // 线程 ID
        .with_span_events(FmtSpan::CLOSE) // span 结束时记录耗时
        .with_writer(file_appender);

    // ── 控制台输出层（开发调试用） ──
    let console_layer = fmt::layer().with_target(true).pretty();

    // ── 按模块过滤日志等级 ──
    let filter = EnvFilter::new(config.get_log_config().get_log_level());

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();
}
