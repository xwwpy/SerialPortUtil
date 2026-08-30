use encoding_rs::{Decoder, Encoder};
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyWindowHandle, AppContext, ClickEvent, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, ParentElement, Render, Styled, Subscription, Window, actions, blue, div,
    green, rgb, white,
};
use gpui_component::button::Button;
use gpui_component::checkbox::Checkbox;
use gpui_component::dialog::DialogButtonProps;
use gpui_component::input::{Copy, Input, InputState, RopeExt, SelectAll};
use gpui_component::label::Label;
use gpui_component::{Disableable, StyledExt, WindowExt, gray};
use ww_protocol::SerialPort;

use crate::model::config_panel::{AutoAppendItem, Supported};
use crate::ui_config;

actions!([ClearPortInputText, ClearUserInputText, ClearUserShowText]);

pub struct IoPanel {
    window: AnyWindowHandle,
    pub port_handle: Option<Box<dyn SerialPort>>,
    pub last_send_completed: bool,             // 上次发送是否完成
    port_input_state: Entity<InputState>,      // 端口输入内容展示面板
    user_input_show_state: Entity<InputState>, // 用户输入内容展示面板
    user_input_state: Entity<InputState>,

    show_port_input_content: bool,
    show_user_input_content: bool,

    port_input_focus_handle: FocusHandle,
    user_input_focus_handle: FocusHandle,
    info_config_focus_handle: FocusHandle,
    whether_auto_scroll_to_bottom: bool,
    whether_add_timestamp: bool,
    pub port_open_state: bool,
    pub encoding: Supported,

    pub encoder: Option<Encoder>,
    pub decoding: Supported,
    // 当前的流式解码器；Hex 模式为 None
    pub decoder: Option<Decoder>,

    pub auto_tx_append: AutoAppendItem,

    pub _receive_data_subscription: Option<Subscription>,
    pub _open_state_observer_subscription: Option<Subscription>,
    port_input_new_line: bool,
    port_input_received_bytes: u64,
    port_input_max_lines: usize,
    // Hex 模式下当前行是否已有内容，用于控制字节间的空格
    port_input_line_has_bytes: bool,

    user_input_max_line: usize,
    user_input_line_has_bytes: bool,
    simple_time_show: bool,
    pub _encoding_changed_subscription: Option<Subscription>,
    pub _decoding_changed_subscription: Option<Subscription>,
    pub _auto_tx_append_observer_subscription: Option<Subscription>,
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
                                    .flex_wrap()
                                    .child(Label::new("Rx："))
                                    .child(
                                        Checkbox::new("WhetherShowRxData")
                                            .checked(self.show_port_input_content)
                                            .on_click(cx.listener(|this, checked, window, cx| {
                                                this.show_port_input_content = *checked;
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
                                    .flex_wrap()
                                    .child(Label::new("Tx："))
                                    .child(
                                        Checkbox::new("WhetherShowTxData")
                                            .checked(self.show_user_input_content)
                                            .on_click(cx.listener(|this, checked, window, cx| {
                                                this.show_user_input_content = *checked;
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
                                    .flex_wrap()
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
                                    .flex_wrap()
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
                                    .flex_wrap()
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
                                    .flex_wrap()
                                    .child(
                                        Label::new(
                                            // 去掉默认的一行
                                            (self.port_input_state.read(cx).text().lines_len() - 1)
                                                .to_string(),
                                        )
                                        .text_color(green()),
                                    )
                                    .child(Label::new(format!("/{}-", self.port_input_max_lines)))
                                    .child(
                                        Label::new(self.port_input_received_bytes.to_string())
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
                                    .text_color(blue())
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
                                        self.port_open_state && self.last_send_completed,
                                        |btn| btn.cursor_pointer(),
                                        |btn| btn.disabled(true).cursor_not_allowed(),
                                    )
                                    .on_click(cx.listener(submit_user_input)),
                            ),
                    ),
            )
    }
}

pub fn submit_user_input(
    this: &mut IoPanel,
    _event: &ClickEvent,
    window: &mut Window,
    cx: &mut Context<IoPanel>,
) {
    if !this.last_send_completed {
        window.open_alert_dialog(cx, move |alert, _, _cx| {
            alert
                .title("发送的过于频繁...")
                .description("请等待上次发送完成后再发送")
                .button_props(DialogButtonProps::default().ok_text("关闭"))
                .on_ok(|_, _window, _cx| true)
        });
        return;
    }

    this.last_send_completed = false;

    this.focus_user_input(window, cx);

    let user_input = this.user_input_state.read(cx).value();

    let mut content_to_show = user_input.to_string();

    match this.auto_tx_append {
        AutoAppendItem::None => {}
        AutoAppendItem::Lf => {
            content_to_show.push('\n');
        }
        AutoAppendItem::Cr => {
            content_to_show.push('\r');
        }
        AutoAppendItem::CrLf => {
            content_to_show.push_str("\r\n");
        }
        AutoAppendItem::LfCr => {
            content_to_show.push_str("\n\r");
        }
    };

    let current_encoding = this.encoding;

    let content_to_send = content_to_show.clone();

    let mut port_handle = this.port_handle.take();

    cx.spawn(async move |io_panel, cx| {
        let datas = IoPanel::resolve_user_input_data(current_encoding, &content_to_send);

        let total_bytes = datas.len();

        let chunk_size = 64;

        let mut send_bytes = 0;

        if let Some(ref mut port_handle) = port_handle {
            while send_bytes < total_bytes {
                if send_bytes + chunk_size > total_bytes {
                    let chunk = &datas[send_bytes..];
                    let res = port_handle.write(chunk);
                    if res.is_ok() {
                        send_bytes += res.unwrap();
                    } else {
                        tracing::error!("Failed to write chunk: {:?}", res);
                        break;
                    }
                } else {
                    let chunk = &datas[send_bytes..send_bytes + chunk_size];
                    let res = port_handle.write(chunk);
                    if res.is_ok() {
                        send_bytes += res.unwrap();
                    } else {
                        tracing::error!("Failed to write chunk: {:?}", res);
                        break;
                    }
                }
            }
        }

        let _ = io_panel.update(cx, move |panel, cx| {
            panel.port_handle = port_handle;
            panel.last_send_completed = true;
            cx.notify();
        });
    })
    .detach();

    if this.whether_add_timestamp {
        if this.simple_time_show {
            content_to_show = format!(
                "{}{}",
                jiff::Zoned::now().strftime("%H:%M:%S%.3f: "),
                content_to_show
            );
        } else {
            content_to_show = format!(
                "{}{}",
                jiff::Zoned::now().strftime("%Y-%m-%d %H:%M:%S%.3f: "),
                content_to_show
            );
        }
    }

    // 窗口已经在更新栈中，不能再使用window_handle.update
    IoPanel::insert_text_without_window_handle(
        content_to_show,
        cx,
        this.user_input_show_state.clone(),
        this.whether_auto_scroll_to_bottom,
        window,
        this.user_input_max_line,
    );
    cx.notify();
}

impl IoPanel {
    pub fn new(cx: &mut gpui::prelude::Context<Self>, window: &mut Window) -> Self {
        let config = ui_config::get().get_common_config();

        let encoding: Supported = config.get_encoding().into();
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
            last_send_completed: true,
            port_handle: None,
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
            encoding,
            encoder: encoding.encoding().map(|e| e.new_encoder()),
            decoding,
            decoder: decoding.encoding().map(|e| e.new_decoder()),

            auto_tx_append: ui_config::get()
                .get_common_config()
                .get_default_auto_tx_append_item()
                .into(),

            _receive_data_subscription: None,
            _open_state_observer_subscription: None,
            port_open_state: false,
            _encoding_changed_subscription: None,
            _decoding_changed_subscription: None,
            _auto_tx_append_observer_subscription: None,

            port_input_new_line: true,
            port_input_received_bytes: 0,
            port_input_max_lines: ui_config::get().get_common_config().get_max_lines(),
            port_input_line_has_bytes: false,

            user_input_max_line: ui_config::get().get_common_config().get_max_lines(),
            user_input_line_has_bytes: false,
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
        self.port_input_line_has_bytes = false;
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
        _: &ClearUserShowText,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 清空显示内容
        let user_input_show_state = self.user_input_show_state.clone();
        user_input_show_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        self.user_input_line_has_bytes = false;
    }

    fn focus_info_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.info_config_focus_handle.focus(window, cx);
    }

    fn focus_user_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.user_input_focus_handle.focus(window, cx);
    }

    pub fn resolve_port_input_data(&mut self, datas: &[u8], cx: &mut Context<Self>) {
        self.port_input_received_bytes += datas.len() as u64;

        let mut pending = String::new();

        for &byte in datas {
            if self.whether_add_timestamp && self.port_input_new_line {
                pending.push_str(&format!(
                    "{} ",
                    if self.simple_time_show {
                        jiff::Zoned::now().strftime("%H:%M:%S%.3f: ")
                    } else {
                        jiff::Zoned::now().strftime("%Y-%m-%d %H:%M:%S%.3f: ")
                    }
                ));
                self.port_input_new_line = false;
            }
            match byte {
                b'\n' => {
                    // 当前行完成：刷新解码器残留内容，然后追加换行
                    self.finish_port_input_line(&mut pending);
                    pending.push('\n');
                    self.port_input_line_has_bytes = false;
                }
                b'\r' => {
                    self.port_input_new_line = false;
                }
                _ => {
                    self.port_input_new_line = false;
                    // 追加到当前未完成行
                    match &mut self.decoder {
                        Some(decoder) => {
                            // decode_to_string 不会自行扩容，需要预留输出空间
                            pending.reserve(16);
                            let _ = decoder.decode_to_string(&[byte], &mut pending, false);
                        }
                        None => {
                            // Hex 显示：字节之间增加一个空格
                            if self.port_input_line_has_bytes {
                                pending.push(' ');
                            }
                            pending.push_str(&format!("{:02X}", byte));
                            self.port_input_line_has_bytes = true;
                        }
                    }
                }
            }
        }

        if !pending.is_empty() {
            Self::insert_text(
                pending,
                cx,
                self.port_input_state.clone(),
                self.whether_auto_scroll_to_bottom,
                self.window,
                self.port_input_max_lines,
            );
            // 限制最大行数，超出后释放最前面的文本
        }
        cx.notify();
    }

    /// 结束当前未完成行：将解码器中残留的不完整字节刷新为文本，并创建新的解码器
    fn finish_port_input_line(&mut self, pending: &mut String) {
        if let Some(ref mut decoder) = self.decoder {
            // flush 可能写入 U+FFFD（3 字节），预留空间避免越界 panic
            pending.reserve(16);
            let _ = decoder.decode_to_string(&[], pending, true);
        }
        // last=true 会把解码器置为 Finished，必须重新创建新解码器，
        // 否则下一次 decode_to_string 会 panic（Must not use a decoder that has finished）
        self.decoder = self.decoding.encoding().map(|e| e.new_decoder());
        self.port_input_new_line = true;
    }

    /// 将用户输入按发送编码解析为字节数据并返回。
    ///
    /// - 非 Hex 编码：直接按对应编码把文本编码成字节。
    /// - Hex 编码：输入是十六进制字节串，空格等空白字符仅作为分隔符，不参与解析。
    /// TODO 将解析好的数据发送给串口
    pub fn resolve_user_input_data(encoding: Supported, datas: &str) -> Vec<u8> {
        match encoding.encoding() {
            Some(enc) => enc.encode(datas).0.into_owned(),
            None => {
                let mut out = Vec::new();
                let mut high: Option<u8> = None;
                for c in datas.chars() {
                    // 空白字符作为分隔符，直接跳过
                    if c.is_whitespace() {
                        continue;
                    }
                    if let Some(n) = c.to_digit(16) {
                        match high.take() {
                            Some(h) => out.push((h << 4) | n as u8),
                            None => high = Some(n as u8),
                        }
                    }
                    // 其他非十六进制字符忽略
                }
                if let Some(h) = high {
                    out.push(h << 4);
                }
                out
            }
        }
    }

    fn trim_to_max_lines(
        state: &mut InputState,
        window: &mut Window,
        cx: &mut Context<InputState>,
        max_lines: usize,
    ) {
        let start_byte = {
            let text = state.text();
            // 去掉默认的一行
            let total_lines = text.lines_len() - 1;
            if total_lines > max_lines {
                let excess = total_lines - max_lines;
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

    /// 向指定的编辑器追加文本（增量插入，避免全量重建）
    fn insert_text(
        text: String,
        cx: &mut Context<Self>,
        target_state: Entity<InputState>,
        whether_auto_scroll_to_bottom: bool,
        window: AnyWindowHandle,
        max_lines: usize,
    ) {
        let _ = window.update(cx, |_, window, cx| {
            target_state.update(cx, |state, cx| {
                let saved_scroll = state.scroll_offset();

                state.insert(text, window, cx);
                Self::trim_to_max_lines(state, window, cx, max_lines);

                let len = state.text().len();

                // 修复原来的 set_cursor_position 一直主动聚焦自身的bug
                state.set_selected_range(len..len, cx);

                if !whether_auto_scroll_to_bottom {
                    // 裁剪会把光标移到开头，这里恢复裁剪前的滚动位置；
                    // 光标已移回末尾，避免下次 insert 插到开头。
                    state.set_scroll_offset(saved_scroll, cx);
                }
                cx.notify();
            });
        });
    }

    fn insert_text_without_window_handle(
        text: String,
        cx: &mut Context<Self>,
        target_state: Entity<InputState>,
        whether_auto_scroll_to_bottom: bool,
        window: &mut Window,
        max_lines: usize,
    ) {
        target_state.update(cx, |state, cx| {
            let saved_scroll = state.scroll_offset();

            state.insert(text, window, cx);
            Self::trim_to_max_lines(state, window, cx, max_lines);

            let len = state.text().len();

            // 修复原来的 set_cursor_position 一直主动聚焦自身的bug
            state.set_selected_range(len..len, cx);

            if !whether_auto_scroll_to_bottom {
                // 裁剪会把光标移到开头，这里恢复裁剪前的滚动位置；
                // 光标已移回末尾，避免下次 insert 插到开头。
                state.set_scroll_offset(saved_scroll, cx);
            }
            cx.notify();
        });
    }
}
