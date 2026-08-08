use crate::ui::title_bar::TitleBar;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, blue, div, px, rgb,
};

pub struct MainView {
    focus_handle: FocusHandle,
    title_bar: Entity<TitleBar>,
}

impl MainView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            title_bar: cx.new(|cx| TitleBar::new(cx)),
        }
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_around()
            .gap_1()
            .child(
                div()
                    .id("TitleBar")
                    .w_full()
                    .flex()
                    .flex_grow(0.)
                    .justify_center()
                    .items_center()
                    .border_color(blue())
                    .border_1()
                    .rounded_md()
                    .child(self.title_bar.clone()),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_grow(1.)
                    .justify_center()
                    .items_center()
                    .track_focus(&self.focus_handle)
                    .when_else(
                        self.focus_handle.is_focused(window),
                        |div| div.border_color(rgb(0x8A2BE2)),
                        |div| div.border_color(blue()),
                    )
                    .border(px(1.3))
                    .rounded_md()
                    .child("content"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_grow(0.)
                    .justify_center()
                    .items_center()
                    .border_color(blue())
                    .border_1()
                    .rounded_md()
                    .child("info"),
            )
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
