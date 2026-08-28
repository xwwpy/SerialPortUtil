use encoding_rs::Decoder;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    ParentElement, Render, Styled, Subscription, Window, actions, blue, div, green, rgb, white,
};
use gpui_component::button::Button;
use gpui_component::checkbox::Checkbox;
use gpui_component::input::{Copy, Input, InputState, RopeExt, SelectAll};
use gpui_component::label::Label;
use gpui_component::{Disableable, StyledExt, gray};

use crate::event::OpenStateChanged;
use crate::model::config_panel::Supported;
use crate::ui_config;
use crate::{event::ReceivedData, ui::port_panel::PortPanel};

actions!([ClearPortInputText, ClearUserInputText, ClearUserShowText]);

pub struct IoPanel {
    window: AnyWindowHandle,
    port_input_state: Entity<InputState>, // 端口输入内容展示面板
    user_input_show_state: Entity<InputState>, // 用户输入内容展示面板
    user_input_state: Entity<InputState>,

    show_port_input_content: bool,
    show_user_input_content: bool,

    port_input_focus_handle: FocusHandle,
    user_input_focus_handle: FocusHandle,
    info_config_focus_handle: FocusHandle,
    whether_auto_scroll_to_bottom: bool,
    whether_add_timestamp: bool,
    port_open_state: bool,
    pub encoding: Supported,
    pub decoding: Supported,
    // 当前未完成行的流式解码器；Hex 模式为 None
    decoder: Option<Decoder>,
    // 解码器对应的编码方式，用于检测解码方式是否发生变化
    decoder_encoding: Supported,
    // Hex 模式下当前行是否已有内容，用于控制字节间的空格
    line_has_bytes: bool,
    _receive_data_subscription: Option<Subscription>,
    _open_state_observer_subscription: Option<Subscription>,
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
            .on_action(cx.listener(Self::clear_port_input_text))
            .on_action(cx.listener(Self::clear_user_input_text))
            .on_action(cx.listener(Self::clear_user_input_show_text))
            // output view
            .child(
                div()
                    .flex()
                    .gap_2()
                    .h_2_3()
                    .p_2()
                    .bg(white())
                    .shadow_md()
                    .rounded_md()
                    .child(
                        div()
                            .flex()
                            .size_full()
                            .flex_grow_0()
                            .rounded_md()
                            .p_2()
                            .gap_2()
                            .track_focus(&self.port_input_focus_handle)
                            .border_1()
                            .when_else(
                                self.port_input_focus_handle.is_focused(window)
                                    || self.port_input_state.focus_handle(cx).is_focused(window)
                                    || self
                                        .user_input_show_state
                                        .focus_handle(cx)
                                        .is_focused(window),
                                |div| div.border_color(rgb(config.get_focus_border_color())),
                                |div| div.border_color(rgb(config.get_default_border_color())),
                            )
                            .when(self.show_port_input_content, |div| {
                                div.child(
                                    Input::new(&self.port_input_state)
                                        .h_full()
                                        .disabled(true)
                                        .text_color(green())
                                        .context_menu(|menu, _window, _cx| {
                                            menu.menu("复制", Box::new(Copy))
                                                .separator()
                                                .menu("全选", Box::new(SelectAll))
                                                .separator()
                                                .menu("清空文本", Box::new(ClearPortInputText))
                                        }),
                                )
                            })
                            .when(self.show_user_input_content, |div| {
                                div.child(
                                    Input::new(&self.user_input_show_state)
                                        .h_full()
                                        .disabled(true)
                                        .text_color(blue())
                                        .context_menu(|menu, _window, _cx| {
                                            menu.menu("复制", Box::new(Copy))
                                                .separator()
                                                .menu("全选", Box::new(SelectAll))
                                                .separator()
                                                .menu("清空文本", Box::new(ClearUserShowText))
                                        }),
                                )
                            }),
                    ),
            )
            // info view
            .child(
                div()
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
                            .border_1()
                            .track_focus(&self.info_config_focus_handle)
                            .when_else(
                                self.info_config_focus_handle.is_focused(window),
                                |div| div.border_color(rgb(config.get_focus_border_color())),
                                |div| div.border_color(rgb(config.get_default_border_color())),
                            )
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
                                            .on_click(cx.listener(|this, checked, window, cx| {
                                                this.whether_auto_scroll_to_bottom = *checked;
                                                this.focus_info_config(window, cx);
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
                                            .on_click(cx.listener(|this, checked, window, cx| {
                                                this.whether_add_timestamp = *checked;
                                                this.focus_info_config(window, cx);
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
                                            .on_click(cx.listener(|this, checked, window, cx| {
                                                this.simple_time_show = *checked;
                                                this.focus_info_config(window, cx);
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
                                            (self.port_input_state.read(cx).text().lines_len() - 1)
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
                            .flex()
                            .size_full()
                            .border_1()
                            .rounded_md()
                            .p_2()
                            .gap_4()
                            .items_center()
                            .overflow_hidden()
                            .justify_evenly()
                            .track_focus(&self.user_input_focus_handle)
                            .when_else(
                                self.user_input_focus_handle.is_focused(window)
                                    || self.user_input_state.focus_handle(cx).is_focused(window),
                                |div| div.border_color(rgb(config.get_focus_border_color())),
                                |div| div.border_color(rgb(config.get_default_border_color())),
                            )
                            .child(
                                Input::new(&self.user_input_state)
                                    .size_full()
                                    .text_color(green())
                                    .border_color(gray(100))
                                    .context_menu(|menu, _window, _cx| {
                                        menu.menu("复制", Box::new(Copy))
                                            .separator()
                                            .menu("全选", Box::new(SelectAll))
                                            .separator()
                                            .menu("清空文本", Box::new(ClearUserInputText))
                                    }),
                            )
                            .child(
                                Button::new("submit")
                                    .label("发送")
                                    .when_else(
                                        self.port_open_state,
                                        |btn| btn.cursor_pointer(),
                                        |btn| btn.disabled(true).cursor_not_allowed(),
                                    )
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.focus_user_input(window, cx);
                                        // TODO
                                    })),
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

        let open_state_subscription = cx.subscribe(
            &port_panel,
            |this, _port_panel, open_state: &OpenStateChanged, cx| {
                this.port_open_state = open_state.open_state;
                cx.notify();
            },
        );

        let config = ui_config::get().get_common_config();
        let decoding: Supported = config.get_decoding().into();

        let io_panel_config = ui_config::get().get_io_panel_config();

        // 创建只读的文本编辑器，用于展示接收数据
        let port_input_state = cx.new(|cx| InputState::new(window, cx).multi_line(true));
        port_input_state.update(cx, |state, cx| {
            state.set_placeholder(io_panel_config.get_port_input_placeholder(), window, cx);

            state.set_soft_wrap(true, window, cx);
        });

        let user_input_show_state = cx.new(|cx| InputState::new(window, cx).multi_line(true));
        user_input_show_state.update(cx, |state, cx| {
            state.set_placeholder(io_panel_config.get_user_input_placeholder(), window, cx);

            state.set_soft_wrap(true, window, cx);
        });

        let user_input_state = cx.new(|cx| InputState::new(window, cx).multi_line(true));
        user_input_state.update(cx, |state, cx| {
            state.set_value(io_panel_config.get_default_output(), window, cx);

            state.set_soft_wrap(true, window, cx);
        });

        IoPanel {
            window: window.window_handle(),
            port_input_state,
            user_input_show_state,
            user_input_state,
            port_input_focus_handle: cx.focus_handle(),
            show_port_input_content: true,
            show_user_input_content: false,
            user_input_focus_handle: cx.focus_handle(),
            info_config_focus_handle: cx.focus_handle(),
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
            _open_state_observer_subscription: Some(open_state_subscription),
            port_open_state: false,
            _encoding_changed_subscription: None,
            _decoding_changed_subscription: None,
            received_bytes: 0,
            max_lines: ui_config::get().get_common_config().get_max_lines(),
        }
    }

    fn clear_port_input_text(
        &mut self,
        _: &ClearPortInputText,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 清空显示内容，并重置当前行的解码状态
        let port_input_state = self.port_input_state.clone();
        port_input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        self.decoder = self.decoding.encoding().map(|e| e.new_decoder());
        self.decoder_encoding = self.decoding;
        self.line_has_bytes = false;
    }

    fn clear_user_input_text(
        &mut self,
        _: &ClearUserInputText,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 清空显示内容
        let user_input_state = self.user_input_state.clone();
        user_input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
    }

    fn clear_user_input_show_text(
        &mut self,
        _: &ClearUserInputText,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 清空显示内容
        let user_input_show_state = self.user_input_show_state.clone();
        user_input_show_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
    }

    fn focus_info_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.info_config_focus_handle.focus(window, cx);
    }

    fn focus_user_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.user_input_focus_handle.focus(window, cx);
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
        let port_input_state = self.port_input_state.clone();

        let _ = window.update(cx, |_, window, cx| {
            port_input_state.update(cx, |state, cx| {
                let saved_scroll = state.scroll_offset();

                state.insert(text, window, cx);
                self.trim_to_max_lines(state, window, cx);

                let len = state.text().len();

                // 修复原来的 set_cursor_position 一直主动聚焦自身的bug
                state.set_selected_range(len..len, cx);

                if !auto_scroll {
                    // 裁剪会把光标移到开头，这里恢复裁剪前的滚动位置；
                    // 光标已移回末尾，避免下次 insert 插到开头。
                    state.set_scroll_offset(saved_scroll, cx);
                }
            });
        });
    }
}
