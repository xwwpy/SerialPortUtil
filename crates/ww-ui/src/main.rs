use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, Bounds, size, px, WindowOptions, WindowBounds, TitlebarOptions, WindowKind, WindowBackgroundAppearance, AppContext, Focusable, FocusHandle};

struct App {
    focus: FocusHandle,
}


impl App {
    fn new(cx: &mut Context<Self>) -> Self {
        App {
            focus: cx.focus_handle()
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
            .child("Hello world")
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        let bounds = Bounds::centered(None, size(px(400.), px(400.)), cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("SerialPortUtil".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
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

        let window_handle = cx.open_window(options, |window, cx| {
            cx.new(|cx| App::new(cx))
        }).unwrap();

        window_handle.update(cx, |view, window, cx| {
            window.activate_window();
            window.focus(&view.focus_handle(cx), cx);
        }).unwrap();
    });
}
