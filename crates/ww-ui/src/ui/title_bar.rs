use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    WindowControlArea, div,
};
use gpui_component::Icon;
use gpui_component::button::{Button, ButtonVariants};

pub struct TitleBar {}

impl TitleBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        TitleBar {}
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .child(
                div()
                    .flex_grow(1.0)
                    .h_full()
                    .items_center()
                    .window_control_area(WindowControlArea::Drag),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("min_btn")
                            .ghost()
                            .on_click(|_event, window, _cx| {
                                window.minimize_window();
                            })
                            .icon(Icon::default().path("minimize.svg"))
                            .rounded_full()
                            .cursor_pointer(),
                    )
                    .child(
                        Button::new("max_btn")
                            .ghost()
                            .on_click(|_event, window, _cx| {
                                window.toggle_fullscreen();
                            })
                            .icon(Icon::default().path("maximize.svg"))
                            .rounded_full()
                            .cursor_pointer(),
                    )
                    .child(
                        Button::new("close_btn")
                            .ghost()
                            .icon(Icon::default().path("close.svg"))
                            .on_click(|_event, window, _cx| {
                                window.remove_window();
                            })
                            .rounded_full()
                            .cursor_pointer(),
                    ),
            )
    }
}
