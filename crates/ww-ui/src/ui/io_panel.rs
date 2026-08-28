use encoding_rs::Decoder;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, FocusHandle, InteractiveElement, ParentElement,
    Render, Styled, Subscription, Window, actions, div, green, rgb, white,
};
use gpui_component::StyledExt;
use gpui_component::checkbox::Checkbox;
use gpui_component::input::{Copy, Input, InputState, RopeExt, SelectAll};
use gpui_component::label::Label;

use crate::model::config_panel::Supported;
use crate::ui_config;
use crate::{event::ReceivedData, ui::port_panel::PortPanel};

actions!([ClearText]);

pub struct IoPanel {
    window: AnyWindowHandle,
    input_state: Entity<InputState>,
    output_focus_handle: FocusHandle,
    input_focus_handle: FocusHandle,
    whether_auto_scroll_to_bottom: bool,
    whether_add_timestamp: bool,
    pub encoding: Supported,
    pub decoding: Supported,
    // 当前未完成行的流式解码器；Hex 模式为 None
    decoder: Option<Decoder>,
    // 解码器对应的编码方式，用于检测解码方式是否发生变化
    decoder_encoding: Supported,
    // Hex 模式下当前行是否已有内容，用于控制字节间的空格
    line_has_bytes: bool,
    _receive_data_subscription: Option<Subscription>,
    new_line: bool,
    received_bytes: u64,
    max_lines: usize,
    simple_time_show: bool,
    pub _encoding_changed_subscription: Option<Subscription>,
    pub _decoding_changed_subscription: Option<Subscription>,
}

impl Render for IoPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
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
                            .child(
                                Input::new(&self.input_state)
                                    .size_full()
                                    .disabled(true)
                                    .text_color(green())
                                    .context_menu(|menu, _window, _cx| {
                                        menu.menu("复制", Box::new(Copy))
                                            .separator()
                                            .menu("全选", Box::new(SelectAll))
                                            .separator()
                                            .menu("清空文本", Box::new(ClearText))
                                    }),
                            ),
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
                            .justify_evenly()
                            .p_2()
                            .gap_4()
                            .text_sm()
                            .rounded_md()
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .overflow_hidden()
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
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .overflow_hidden()
                                    .child(Label::new("添加时间戳："))
                                    .child(
                                        Checkbox::new("add_timestamp")
                                            .checked(self.whether_add_timestamp)
                                            .on_click(cx.listener(|this, checked, _window, cx| {
                                                this.whether_add_timestamp = *checked;
                                                cx.notify();
                                            }))
                                            .cursor_pointer(),
                                    ),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .overflow_hidden()
                                    .child(Label::new("精简时间："))
                                    .child(
                                        Checkbox::new("simple_time_show")
                                            .checked(self.simple_time_show)
                                            .on_click(cx.listener(|this, checked, _window, cx| {
                                                this.simple_time_show = *checked;
                                                cx.notify();
                                            }))
                                            .cursor_pointer(),
                                    ),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .items_center()
                                    .overflow_hidden()
                                    .child(
                                        Label::new(
                                            // 去掉默认的一行
                                            (self.input_state.read(cx).text().lines_len() - 1)
                                                .to_string(),
                                        )
                                        .text_color(green()),
                                    )
                                    .child(Label::new(format!("/{}-", self.max_lines)))
                                    .child(
                                        Label::new(self.received_bytes.to_string())
                                            .text_color(green()),
                                    )
                                    .child(Label::new("个字节")),
                            ),
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
    pub fn new(
        cx: &mut gpui::prelude::Context<Self>,
        window: &mut Window,
        port_panel: Entity<PortPanel>,
    ) -> Self {
        let subscription = cx.subscribe(
            &port_panel,
            |this, _port_panel, datas: &ReceivedData, cx| {
                this.resolve_data(&datas.data, cx);
            },
        );

        let config = ui_config::get().get_common_config();
        let decoding: Supported = config.get_decoding().into();

        // 创建只读的文本编辑器，用于展示接收数据
        let input_state = cx.new(|cx| InputState::new(window, cx).multi_line(true));
        input_state.update(cx, |state, cx| {
            state.set_soft_wrap(true, window, cx);
        });

        IoPanel {
            window: window.window_handle(),
            input_state,
            output_focus_handle: cx.focus_handle(),
            input_focus_handle: cx.focus_handle(),
            whether_auto_scroll_to_bottom: true,
            whether_add_timestamp: true,
            simple_time_show: true,
            encoding: config.get_encoding().into(),
            decoding,
            decoder: decoding.encoding().map(|e| e.new_decoder()),
            decoder_encoding: decoding,
            line_has_bytes: false,
            new_line: true,
            _receive_data_subscription: Some(subscription),
            _encoding_changed_subscription: None,
            _decoding_changed_subscription: None,
            received_bytes: 0,
            max_lines: ui_config::get().get_common_config().get_max_lines(),
        }
    }

    fn clear_text(&mut self, _: &ClearText, window: &mut Window, cx: &mut Context<Self>) {
        // 清空显示内容，并重置当前行的解码状态
        let input_state = self.input_state.clone();
        input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        self.decoder = self.decoding.encoding().map(|e| e.new_decoder());
        self.decoder_encoding = self.decoding;
        self.line_has_bytes = false;
    }

    pub fn resolve_data(&mut self, datas: &[u8], cx: &mut Context<Self>) {
        // 解码方式发生变化时，重置解码器（已渲染的内容保持不变，只影响新数据）
        if self.decoder_encoding != self.decoding {
            self.decoder = self.decoding.encoding().map(|e| e.new_decoder());
            self.decoder_encoding = self.decoding;
            self.line_has_bytes = false;
        }
        self.received_bytes += datas.len() as u64;

        let mut pending = String::new();

        for &byte in datas {
            if self.whether_add_timestamp && self.new_line {
                pending.push_str(&format!(
                    "{} ",
                    if self.simple_time_show {
                        jiff::Zoned::now().strftime("%H:%M:%S%.3f: ")
                    } else {
                        jiff::Zoned::now().strftime("%Y-%m-%d %H:%M:%S%.3f: ")
                    }
                ));
                self.new_line = false;
            }
            match byte {
                b'\n' => {
                    // 当前行完成：刷新解码器残留内容，然后追加换行
                    self.finish_line(&mut pending);
                    pending.push('\n');
                    self.line_has_bytes = false;
                }
                b'\r' => {
                    self.new_line = false;
                }
                _ => {
                    self.new_line = false;
                    // 追加到当前未完成行
                    match &mut self.decoder {
                        Some(decoder) => {
                            // decode_to_string 不会自行扩容，需要预留输出空间
                            pending.reserve(16);
                            let _ = decoder.decode_to_string(&[byte], &mut pending, false);
                        }
                        None => {
                            // Hex 显示：字节之间增加一个空格
                            if self.line_has_bytes {
                                pending.push(' ');
                            }
                            pending.push_str(&format!("{:02X}", byte));
                            self.line_has_bytes = true;
                        }
                    }
                }
            }
        }

        if !pending.is_empty() {
            self.insert_text(pending, cx);
            // 限制最大行数，超出后释放最前面的文本
        }
        cx.notify();
    }

    fn trim_to_max_lines(
        &self,
        state: &mut InputState,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        let start_byte = {
            let text = state.text();
            // 去掉默认的一行
            let total_lines = text.lines_len() - 1;
            if total_lines > self.max_lines {
                let excess = total_lines - self.max_lines;
                Some(text.line_start_offset(excess))
            } else {
                None
            }
        };

        if let Some(start_byte) = start_byte {
            // 直接删除最前面的字节范围，而不是 set_value 全量重建
            state.set_selected_range(0..start_byte, cx);
            state.replace("", window, cx);
        }
    }

    /// 结束当前未完成行：将解码器中残留的不完整字节刷新为文本，并创建新的解码器
    fn finish_line(&mut self, pending: &mut String) {
        if let Some(mut decoder) = self.decoder.take() {
            // flush 可能写入 U+FFFD（3 字节），预留空间避免越界 panic
            pending.reserve(16);
            let _ = decoder.decode_to_string(&[], pending, true);
        }
        self.decoder = self.decoding.encoding().map(|e| e.new_decoder());
        self.new_line = true;
    }

    /// 向只读编辑器追加文本（增量插入，避免全量重建）
    fn insert_text(&mut self, text: String, cx: &mut Context<Self>) {
        let auto_scroll = self.whether_auto_scroll_to_bottom;
        let window = self.window;
        let input_state = self.input_state.clone();

        let _ = window.update(cx, |_, window, cx| {
            input_state.update(cx, |state, cx| {
                let saved_scroll = state.scroll_offset();

                state.insert(text, window, cx);
                self.trim_to_max_lines(state, window, cx);

                let len = state.text().len();
                if auto_scroll {
                    // 移动光标到末尾并滚动到底部
                    let position = state.text().offset_to_position(len);
                    state.set_cursor_position(position, window, cx);
                } else {
                    // 裁剪会把光标移到开头，这里移回末尾，避免下次 insert 插到开头；
                    // 同时恢复裁剪前的滚动位置
                    state.set_selected_range(len..len, cx);
                    state.set_scroll_offset(saved_scroll, cx);
                }
            });
        });
    }
}
