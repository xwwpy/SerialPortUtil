use gpui::{
    Context, Entity, ParentElement, Render, SharedString, Styled, Subscription, div, green, px,
    size,
};
use std::ops::Range;
use std::rc::Rc;

use gpui_component::{VirtualListScrollHandle, scroll::ScrollableElement, v_virtual_list};

use crate::{event::ReceivedData, model::io_panel::Line, ui::port_panel::PortPanel};

const LINE_HEIGHT: f32 = 24.0;

pub struct IoPanel {
    received_datas: Vec<Line>,
    tmp_line: Vec<u8>,
    scroll_handle: VirtualListScrollHandle,
    _subscription: Subscription,
}

impl Render for IoPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        // v_virtual_list 需要为每一行提供大小，这里统一行高
        let item_sizes = Rc::new(
            (0..self.received_datas.len())
                .map(|_| size(px(100.), px(LINE_HEIGHT)))
                .collect::<Vec<_>>(),
        );

        div().w_2_3().h_full().p_2().child(
            div()
                .size_full()
                .border_1()
                .border_color(green())
                .rounded_md()
                .child(
                    v_virtual_list(
                        cx.entity().clone(),
                        "monitor",
                        item_sizes,
                        move |this, range: Range<usize>, _window, _cx| {
                            range
                                .map(|ix| {
                                    div()
                                        .h(px(LINE_HEIGHT))
                                        .child(SharedString::from(this.received_datas[ix].text()))
                                })
                                .collect::<Vec<_>>()
                        },
                    )
                    .track_scroll(&self.scroll_handle)
                    .h_full(),
                )
                .vertical_scrollbar(&self.scroll_handle),
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
        }
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
            } else {
                // 未遇到换行符：累积到 tmp_line
                self.tmp_line.push(byte);
            }
        }

        cx.notify();
    }
}
