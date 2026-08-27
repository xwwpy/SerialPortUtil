use gpui::prelude::FluentBuilder;
use gpui::{
    Context, Entity, FocusHandle, InteractiveElement, ParentElement, Render, Styled, Subscription,
    Window, actions, div, green, px, rgb, size, white,
};
use gpui_component::checkbox::Checkbox;
use gpui_component::label::Label;
use gpui_component::scroll::ScrollableElement;
use std::ops::Range;
use std::rc::Rc;

use gpui_component::menu::ContextMenuExt;
use gpui_component::{StyledExt, VirtualListScrollHandle, v_virtual_list};

use crate::ui_config;
use crate::{event::ReceivedData, model::io_panel::Line, ui::port_panel::PortPanel};

actions!([ClearText]);

/// 每行高度（含间距）
const LINE_HEIGHT: f32 = 24.0;

pub struct IoPanel {
    received_datas: Vec<Line>,
    tmp_line: Vec<u8>,
    output_focus_handle: FocusHandle,
    input_focus_handle: FocusHandle,
    whether_auto_scroll_to_bottom: bool,
    scroll_handle: VirtualListScrollHandle,
    _subscription: Subscription,
}

impl Render for IoPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        // v_virtual_list 需要为每一行提供大小，这里统一行高
        let item_sizes = Rc::new(
            (0..self.received_datas.len())
                .map(|_| size(px(100.), px(LINE_HEIGHT)))
                .collect::<Vec<_>>(),
        );
        let config = ui_config::get().get_common_config();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_2()
            .size_full()
            .on_action(cx.listener(Self::clear_text))
            // output view
            .child(
                div()
                    .h_2_3()
                    .p_2()
                    .bg(white())
                    .shadow_md()
                    .rounded_md()
                    .child(
                        div()
                            .size_full()
                            .flex_grow_0()
                            .rounded_md()
                            .p_2()
                            .track_focus(&self.output_focus_handle)
                            .border_1()
                            .when_else(
                                self.output_focus_handle.is_focused(window),
                                |div| div.border_color(rgb(config.get_focus_border_color())),
                                |div| div.border_color(rgb(config.get_default_border_color())),
                            )
                            // .bg(white())
                            // .shadow_md()
                            .child(
                                v_virtual_list(
                                    cx.entity().clone(),
                                    "monitor",
                                    item_sizes,
                                    move |this, range: Range<usize>, _window, _cx| {
                                        range
                                            .map(|ix| {
                                                div().h(px(LINE_HEIGHT)).py(px(2.)).child(
                                                    this.received_datas[ix].text().to_string(),
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                )
                                .track_scroll(&self.scroll_handle),
                            )
                            .vertical_scrollbar(&self.scroll_handle)
                            .context_menu(|menu, _window, _cx| {
                                menu.menu("清空文本", Box::new(ClearText))
                            }),
                    ),
            )
            // info view
            .child(
                div()
                    .h(window.rem_size() * 2.)
                    .flex_grow_0()
                    .p_2()
                    .bg(white())
                    .shadow_md()
                    .rounded_md()
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .p_2()
                            .text_sm()
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .child(Label::new("自动滚动到底部："))
                                    .child(
                                        Checkbox::new("auto_scroll_to_button")
                                            .checked(self.whether_auto_scroll_to_bottom)
                                            .on_click(cx.listener(|this, checked, _window, cx| {
                                                this.whether_auto_scroll_to_bottom = *checked;
                                                cx.notify();
                                            }))
                                            .cursor_pointer(),
                                    ),
                            )
                            .rounded_md(),
                    ),
            )
            // input view
            .child(
                div()
                    .flex_1()
                    .p_2()
                    .bg(white())
                    .shadow_md()
                    .rounded_md()
                    .child(
                        div()
                            .size_full()
                            .border_1()
                            .border_color(green())
                            .rounded_md()
                            .track_focus(&self.input_focus_handle)
                            .when_else(
                                self.input_focus_handle.is_focused(window),
                                |div| div.border_color(rgb(config.get_focus_border_color())),
                                |div| div.border_color(rgb(config.get_default_border_color())),
                            ),
                    ),
            )
    }
}

impl IoPanel {
    pub fn new(cx: &mut gpui::prelude::Context<Self>, port_panel: Entity<PortPanel>) -> Self {
        let subscription = cx.subscribe(
            &port_panel,
            |this, _port_panel, datas: &ReceivedData, cx| {
                this.resolve_data(&datas.data, cx);
            },
        );

        IoPanel {
            received_datas: vec![],
            tmp_line: vec![],
            scroll_handle: VirtualListScrollHandle::new(),
            _subscription: subscription,
            whether_auto_scroll_to_bottom: true,
            output_focus_handle: cx.focus_handle(),
            input_focus_handle: cx.focus_handle(),
        }
    }

    fn clear_text(&mut self, _: &ClearText, _: &mut Window, cx: &mut Context<Self>) {
        self.received_datas.clear();
        cx.notify();
    }

    pub fn resolve_data(&mut self, datas: &[u8], cx: &mut Context<Self>) {
        for &byte in datas {
            if byte == b'\n' {
                // 遇到换行符：取出行数据并清空 tmp_line
                let line_bytes = std::mem::take(&mut self.tmp_line);

                // 去掉行尾的 \r（处理 \r\n）
                let line_bytes = if line_bytes.last() == Some(&b'\r') {
                    &line_bytes[..line_bytes.len() - 1]
                } else {
                    &line_bytes[..]
                };

                // UTF-8 解码完整的一行
                let text = String::from_utf8_lossy(line_bytes);
                self.received_datas.push(Line::new(text.into_owned()));
                if self.whether_auto_scroll_to_bottom {
                    self.scroll_handle.scroll_to_bottom();
                }
            } else {
                // 未遇到换行符：累积到 tmp_line
                self.tmp_line.push(byte);
            }
        }

        cx.notify();
    }
}
