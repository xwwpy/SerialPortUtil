use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    WindowControlArea, div,
};
use gpui_component::button::{Button, ButtonVariant, ButtonVariants};
use gpui_component::dialog::DialogButtonProps;
use gpui_component::{Icon, WindowExt};

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
                            .rounded_sm()
                            .cursor_pointer(),
                    )
                    .child(
                        Button::new("max_btn")
                            .ghost()
                            .on_click(|_event, window, _cx| {
                                window.toggle_fullscreen();
                            })
                            .icon(Icon::default().path("maximize.svg"))
                            .rounded_sm()
                            .cursor_pointer(),
                    )
                    .child(
                        Button::new("close_btn")
                            .ghost()
                            .icon(Icon::default().path("close.svg"))
                            .on_click(|_event, window, cx| {
                                window.open_alert_dialog(cx, |alert, _, _cx| {
                                    alert
                                        .title("确定要关闭窗口？")
                                        .description("关闭窗口后会关闭已打开的串口的连接")
                                        .button_props(
                                            DialogButtonProps::default()
                                                .ok_variant(ButtonVariant::Danger) // 危险红色按钮
                                                .ok_text("关闭")
                                                .cancel_text("取消")
                                                .show_cancel(true),
                                        )
                                        .on_ok(|_, window, _cx| {
                                            window.remove_window();
                                            true
                                        })
                                });
                            })
                            .rounded_sm()
                            .cursor_pointer(),
                    ),
            )
    }
}
