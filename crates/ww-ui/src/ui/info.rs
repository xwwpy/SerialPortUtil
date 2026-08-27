use gpui::{Context, ParentElement, Render, SharedString, Styled, div};
use gpui_component::gray;

use crate::ui_config::get;

pub struct Info {
    auther_info: SharedString,
    author_email: SharedString,
}

impl Info {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let config = get();
        Self {
            auther_info: config.get_author_info().get_author_name().into(),
            author_email: config.get_author_info().get_author_email().into(),
        }
    }
}

impl Render for Info {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        div()
            .flex()
            .size_full()
            .gap_2()
            .bg(gray(300))
            .rounded_md()
            .items_center()
            .justify_center()
            .child(self.auther_info.clone())
            .child(self.author_email.clone())
    }
}
