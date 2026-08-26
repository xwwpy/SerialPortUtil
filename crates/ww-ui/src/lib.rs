mod assets;
mod common;
mod event;
mod model;
pub mod ui;
mod ui_config;

use std::time::Instant;

use crate::{common::log, ui::main_view::MainView, ui_config::get};

use gpui::{
    AppContext, Bounds, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, Styled, Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
    div, px, size,
};
use gpui_component::Root;

struct App {
    focus: FocusHandle,
    main_view: Entity<MainView>,
}

impl App {
    fn new(cx: &mut Context<Self>, window: &mut gpui::Window) -> Self {
        Self {
            focus: cx.focus_handle(),
            main_view: cx.new(|cx| MainView::new(cx, window)),
        }
    }
}

impl Focusable for App {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(self.main_view.clone())
            // 渲染对话框层
            .children(dialog_layer)
            // 渲染侧边栏层
            .children(sheet_layer)
            // 渲染通知层
            .children(notification_layer)
    }
}

pub fn run() {
    let start = Instant::now();
    log::init();
    let config = get();

    print_banner();
    tracing::info!("Starting WW UI ...");
    gpui_platform::application()
        .with_assets(assets::AppAssets::new("assets"))
        .run(|cx| {
            gpui_component::init(cx);

            let window_size = config.get_window_size();
            let bounds = Bounds::centered(
                None,
                size(px(window_size.width), px(window_size.height)),
                cx,
            );
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                focus: true,
                show: true,
                kind: WindowKind::Normal,
                is_movable: true,
                app_owns_titlebar_drag: true,
                is_resizable: true,
                is_minimizable: true,
                display_id: None,
                window_background: WindowBackgroundAppearance::Opaque,
                app_id: Some("SerialPortUi".to_string()),
                window_min_size: None,
                window_decorations: None,
                icon: None,
                tabbing_identifier: Some("SerialPortUi".to_string()),
            };

            let window_handle = cx
                .open_window(options, |window, cx| {
                    let view = cx.new(|cx| App::new(cx, window));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .unwrap();

            window_handle
                .update(cx, |_view, window, _cx| {
                    window.activate_window();
                })
                .unwrap();
        });

    let elapsed = start.elapsed();
    tracing::info!("WW UI run for {:.2}s", elapsed.as_secs_f64());
}

fn print_banner() {
    match std::fs::read_to_string("banner.txt") {
        Ok(banner) => {
            println!("\x1b[36m{banner}\x1b[0m");
        }
        Err(_) => {
            println!("\x1b[31mFailed to load banner.txt\x1b[0m");
        }
    }
    println!(
        "\x1b[2m                                          v{}\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "\x1b[36m                         \x1b[1mWW: SerialPortUtil\x1b[0m\x1b[36m  Ready to Connect\x1b[0m"
    );
}
