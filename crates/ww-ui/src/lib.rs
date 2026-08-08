mod common;
pub mod ui;

use crate::ui::main_view::MainView;

use gpui::{
    AppContext, Bounds, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, Styled, Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
    div, px, size,
};

struct App {
    focus: FocusHandle,
    main_view: Entity<MainView>,
}

impl App {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            main_view: cx.new(|cx| MainView::new(cx)),
        }
    }
}

impl Focusable for App {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(self.main_view.clone())
    }
}

pub fn run() {
    gpui_platform::application().run(|cx| {
        let bounds = Bounds::centered(None, size(px(1200.), px(600.)), cx);
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
            .open_window(options, |_window, cx| cx.new(|cx| App::new(cx)))
            .unwrap();

        window_handle
            .update(cx, |view, window, cx| {
                window.activate_window();
                window.focus(&view.focus_handle(cx), cx);
            })
            .unwrap();
    });
}
