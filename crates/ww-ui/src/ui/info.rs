use gpui::{Context, Entity, ParentElement, Render, SharedString, Styled, div, green};
use gpui_component::{RopeExt, gray, label::Label};

use crate::{ui::io_panel::IoPanel, ui_config::get};

pub struct Info {
    auther_info: SharedString,
    author_email: SharedString,
    io_panel: Entity<IoPanel>,
}

impl Info {
    pub fn new(_cx: &mut Context<Self>, io_panel: Entity<IoPanel>) -> Self {
        let config = get();
        Self {
            auther_info: config.get_author_info().get_author_name().into(),
            author_email: config.get_author_info().get_author_email().into(),
            io_panel,
        }
    }
}

impl Render for Info {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        div()
            .flex()
            .size_full()
            .gap_2()
            .bg(gray(300))
            .rounded_md()
            .items_center()
            .justify_evenly()
            .child(self.auther_info.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .child(Label::new("Rx: "))
                    .child(
                        Label::new(
                            // 去掉默认的一行
                            (self
                                .io_panel
                                .read(cx)
                                .port_input_state
                                .read(cx)
                                .text()
                                .lines_len()
                                - 1)
                            .to_string(),
                        )
                        .text_color(green()),
                    )
                    .child(Label::new(format!(
                        "/{}-共",
                        self.io_panel.read(cx).port_input_max_lines
                    )))
                    .child(
                        Label::new(self.io_panel.read(cx).port_input_received_bytes.to_string())
                            .text_color(green()),
                    )
                    .child(Label::new("个字节")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .child(Label::new("Tx: "))
                    .child(
                        Label::new(
                            // 去掉默认的一行
                            (self
                                .io_panel
                                .read(cx)
                                .user_input_show_state
                                .read(cx)
                                .text()
                                .lines_len()
                                - 1)
                            .to_string(),
                        )
                        .text_color(green()),
                    )
                    .child(Label::new(format!(
                        "/{}-共",
                        self.io_panel.read(cx).user_input_max_lines
                    )))
                    .child(
                        Label::new(self.io_panel.read(cx).user_transmit_bytes.to_string())
                            .text_color(green()),
                    )
                    .child(Label::new("个字节")),
            )
            .child(self.author_email.clone())
    }
}
