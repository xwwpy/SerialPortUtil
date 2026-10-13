use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use encoding_rs::{Decoder, Encoder};
use gpui::IntoElement;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyWindowHandle, AppContext, ClickEvent, Context, Entity, FocusHandle, InteractiveElement,
    ParentElement, Render, Styled, Subscription, Window, actions, blue, div, green, rgb, white,
};
use gpui_component::button::Button;
use gpui_component::checkbox::Checkbox;
use gpui_component::input::{Copy, Input, InputState, RopeExt, SelectAll};
use gpui_component::{Disableable, StyledExt};
use ww_protocol::SerialPort;

use crate::model::config_panel::{AutoAppendItem, Supported};
use crate::preferences::PreferenceStore;
use crate::ui_config;

actions!([ClearPortInputText, ClearUserInputText, ClearUserShowText]);

pub struct IoPanel {
    window: AnyWindowHandle,
    pub port_handle: Option<Box<dyn SerialPort>>,
    pub last_send_completed: bool,                 // 上次发送是否完成
    pub port_input_state: Entity<InputState>,      // 端口输入内容展示面板
    pub user_input_show_state: Entity<InputState>, // 用户输入内容展示面板
    user_input_state: Entity<InputState>,
    interval_state: Entity<InputState>,
    timer_payload: Option<(Vec<u8>, String)>,
    timer_task: Option<gpui::Task<()>>,

    port_input_focus_handle: FocusHandle,
    user_input_focus_handle: FocusHandle,
    info_config_focus_handle: FocusHandle,
    whether_auto_scroll_to_bottom: bool,
    whether_add_timestamp: bool,
    pub port_open_state: bool,
    pub session_id: u64,
    pub send_cancel: Arc<AtomicBool>,
    pub last_error: Option<String>,
    pub encoding: Supported,

    pub encoder: Option<Encoder>,
    pub decoding: Supported,
    // 当前的流式解码器；Hex 模式为 None
    pub decoder: Option<Decoder>,

    pub auto_tx_append: AutoAppendItem,

    pub _receive_data_subscription: Option<Subscription>,
    pub _open_state_observer_subscription: Option<Subscription>,
    port_input_new_line: bool,
    pub port_input_received_bytes: u64,
    pub port_input_max_lines: usize,
    // Hex 模式下当前行是否已有内容，用于控制字节间的空格
    port_input_line_has_bytes: bool,

    pub user_input_max_lines: usize,
    pub user_transmit_bytes: u64,
    user_input_line_has_bytes: bool,
    simple_time_show: bool,
    pub _encoding_changed_subscription: Option<Subscription>,
    pub _decoding_changed_subscription: Option<Subscription>,
    pub _auto_tx_append_observer_subscription: Option<Subscription>,
}

impl Render for IoPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let layout = PreferenceStore::get(cx).layout.clone();
        let narrow = window.viewport_size().width < gpui::px(1000.);
        let mut logs = div()
            .flex_1()
            .min_h(gpui::px(180.))
            .min_w_0()
            .flex()
            .gap_3()
            .when(narrow, |view| view.flex_col());
        if layout.receive {
            logs = logs.child(self.log_view(true, cx));
        }
        if layout.transmit {
            logs = logs.child(self.log_view(false, cx));
        }
        div()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_3()
            .on_action(cx.listener(Self::clear_port_input_text))
            .on_action(cx.listener(Self::clear_user_input_text))
            .on_action(cx.listener(Self::clear_user_input_show_text))
            .when_some(self.last_error.clone(), |view, message| {
                view.child(
                    div()
                        .p_2()
                        .rounded_md()
                        .bg(rgb(0xfef3f2))
                        .text_color(rgb(0xb42318))
                        .child(message),
                )
            })
            .when(layout.receive || layout.transmit, |view| view.child(logs))
            .when(layout.receive || layout.transmit, |view| {
                view.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .text_sm()
                        .px_2()
                        .track_focus(&self.info_config_focus_handle)
                        .child(
                            Checkbox::new("auto-scroll")
                                .label("自动滚动")
                                .checked(self.whether_auto_scroll_to_bottom)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.whether_auto_scroll_to_bottom = *checked;
                                    PreferenceStore::update(cx, |s| s.auto_scroll = *checked);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("timestamp")
                                .label("时间戳")
                                .checked(self.whether_add_timestamp)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.whether_add_timestamp = *checked;
                                    PreferenceStore::update(cx, |s| s.timestamp = *checked);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("simple-time")
                                .label("精简时间")
                                .checked(self.simple_time_show)
                                .on_click(cx.listener(|this, checked, _, cx| {
                                    this.simple_time_show = *checked;
                                    PreferenceStore::update(cx, |s| s.simple_time = *checked);
                                    cx.notify();
                                })),
                        ),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .items_center()
                    .text_sm()
                    .child(format!(
                        "Rx {} B · Tx {} B",
                        self.port_input_received_bytes, self.user_transmit_bytes
                    ))
                    .child(div().flex_1())
                    .child(
                        Button::new("reset-statistics")
                            .label("重置计数")
                            .disabled(!self.last_send_completed)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.port_input_received_bytes = 0;
                                this.user_transmit_bytes = 0;
                                cx.notify();
                            })),
                    ),
            )
            .when(layout.editor, |view| {
                view.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_3()
                        .bg(white())
                        .rounded_md()
                        .shadow_sm()
                        .flex_shrink_0()
                        .track_focus(&self.user_input_focus_handle)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().font_semibold().child("发送编辑器"))
                                .child(div().flex_1())
                                .child(Button::new("clear-editor").label("清空").on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.clear_user_input_text(&ClearUserInputText, window, cx)
                                    }),
                                ))
                                .child(
                                    Button::new("submit")
                                        .label(if self.last_send_completed {
                                            "发送"
                                        } else {
                                            "正在发送…"
                                        })
                                        .disabled(
                                            !self.port_open_state || !self.last_send_completed,
                                        )
                                        .on_click(cx.listener(submit_user_input)),
                                ),
                        )
                        .child(
                            Input::new(&self.user_input_state)
                                .h(gpui::px(100.))
                                .w_full()
                                .text_color(blue())
                                .context_menu(|menu, _, _| {
                                    menu.menu("复制", Box::new(Copy))
                                        .menu("全选", Box::new(SelectAll))
                                        .menu("清空文本", Box::new(ClearUserInputText))
                                }),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap_2()
                                .text_sm()
                                .child("定时发送间隔（ms）")
                                .child(
                                    div().w(gpui::px(100.)).child(
                                        Input::new(&self.interval_state)
                                            .w_full()
                                            .disabled(self.timer_payload.is_some()),
                                    ),
                                )
                                .child(
                                    Button::new("timer-toggle")
                                        .label(if self.timer_payload.is_some() {
                                            "停止定时"
                                        } else {
                                            "启动定时"
                                        })
                                        .disabled(
                                            !self.port_open_state
                                                || (!self.last_send_completed
                                                    && self.timer_payload.is_none()),
                                        )
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            if this.timer_payload.is_some() {
                                                this.stop_timer(cx);
                                            } else {
                                                this.start_timer(window, cx);
                                            }
                                        })),
                                )
                                .when(self.timer_payload.is_some(), |view| {
                                    view.child("运行中 · 使用启动时内容")
                                }),
                        ),
                )
            })
    }
}

impl IoPanel {
    fn log_view(&self, receive: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let (label, state, focus) = if receive {
            (
                "接收日志 · Rx",
                &self.port_input_state,
                &self.port_input_focus_handle,
            )
        } else {
            (
                "发送日志 · Tx",
                &self.user_input_show_state,
                &self.user_input_focus_handle,
            )
        };
        div()
            .flex_1()
            .min_h(gpui::px(160.))
            .min_w_0()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .bg(white())
            .rounded_md()
            .shadow_sm()
            .track_focus(focus)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().font_semibold().child(label))
                    .child(div().flex_1())
                    .child(
                        Button::new(if receive { "copy-rx" } else { "copy-tx" })
                            .label("复制")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let state = if receive {
                                    &this.port_input_state
                                } else {
                                    &this.user_input_show_state
                                };
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    state.read(cx).value().to_string(),
                                ));
                            })),
                    )
                    .child(
                        Button::new(if receive { "clear-rx" } else { "clear-tx" })
                            .label("清空")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                if receive {
                                    this.clear_port_input_text(&ClearPortInputText, window, cx);
                                } else {
                                    this.clear_user_input_show_text(&ClearUserShowText, window, cx);
                                }
                            })),
                    )
                    .child(
                        Button::new(if receive { "export-rx" } else { "export-tx" })
                            .label("导出")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let state = if receive {
                                    &this.port_input_state
                                } else {
                                    &this.user_input_show_state
                                };
                                crate::export::request_export(
                                    state.read(cx).value().to_string(),
                                    if receive {
                                        "receive.txt"
                                    } else {
                                        "transmit.txt"
                                    },
                                    window,
                                    cx,
                                );
                            })),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    Input::new(state)
                        .size_full()
                        .disabled(true)
                        .text_color(if receive { green() } else { blue() }),
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
    this.send(window, cx);
}

impl IoPanel {
    fn prepare_payload(&self, cx: &gpui::App) -> Result<(Vec<u8>, String), String> {
        let input = self.user_input_state.read(cx).value().to_string();
        let mut bytes = Self::resolve_user_input_data(self.encoding, &input)?;
        let suffix = match self.auto_tx_append {
            AutoAppendItem::None => "",
            AutoAppendItem::Lf => "\n",
            AutoAppendItem::Cr => "\r",
            AutoAppendItem::CrLf => "\r\n",
            AutoAppendItem::LfCr => "\n\r",
        };
        bytes.extend_from_slice(suffix.as_bytes());
        if bytes.is_empty() {
            return Err("发送内容不能为空".into());
        }
        if bytes.len() > 1_048_576 {
            return Err("单次发送不能超过 1 MiB".into());
        }
        let shown = if self.encoding == Supported::Hex {
            bytes
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            format!("{input}{suffix}")
        };
        Ok((bytes, shown))
    }

    pub fn stop_timer(&mut self, cx: &mut Context<Self>) {
        self.timer_task.take();
        self.timer_payload = None;
        cx.notify();
    }

    fn start_timer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.port_open_state || !self.last_send_completed {
            return;
        }
        let interval = self
            .interval_state
            .read(cx)
            .value()
            .parse::<u64>()
            .ok()
            .filter(|value| (10..=3_600_000).contains(value));
        let Some(interval) = interval else {
            self.last_error = Some("定时发送间隔应为 10–3600000 ms".into());
            cx.notify();
            return;
        };
        let payload = match self.prepare_payload(cx) {
            Ok(payload) => payload,
            Err(error) => {
                self.last_error = Some(error);
                cx.notify();
                return;
            }
        };
        PreferenceStore::update(cx, |s| s.interval_ms = interval);
        self.timer_payload = Some(payload);
        self.send(window, cx);
        let handle = self.window;
        self.timer_task = Some(cx.spawn(async move |entity, cx| {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
                let mut keep_running = false;
                let result = handle.update(cx, |_, window, cx| {
                    let _ = entity.update(cx, |panel, cx| {
                        keep_running = panel.port_open_state && panel.timer_payload.is_some();
                        if keep_running {
                            panel.send(window, cx);
                        }
                    });
                });
                if result.is_err() || !keep_running {
                    break;
                }
            }
        }));
        cx.notify();
    }

    pub fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.port_open_state || !self.last_send_completed {
            return;
        }
        let (bytes, shown) = match self
            .timer_payload
            .clone()
            .map(Ok)
            .unwrap_or_else(|| self.prepare_payload(cx))
        {
            Ok(payload) => payload,
            Err(message) => {
                self.last_error = Some(message);
                self.stop_timer(cx);
                return;
            }
        };
        let Some(mut handle) = self.port_handle.take() else {
            return;
        };
        if self.timer_payload.is_none() {
            self.focus_user_input(window, cx);
        }
        self.last_send_completed = false;
        self.last_error = None;
        let session_id = self.session_id;
        let cancel = self.send_cancel.clone();
        let timestamp = if self.whether_add_timestamp {
            jiff::Zoned::now()
                .strftime(if self.simple_time_show {
                    "%H:%M:%S%.3f "
                } else {
                    "%Y-%m-%d %H:%M:%S%.3f "
                })
                .to_string()
        } else {
            String::new()
        };
        cx.spawn(async move |entity, cx| {
            let result = tokio::task::spawn_blocking(move || {
                let result = ww_protocol::transfer::write_bytes(&mut handle, &bytes, || {
                    cancel.load(Ordering::Acquire)
                });
                (handle, result)
            })
            .await;
            let _ = entity.update(cx, move |panel, cx| {
                if panel.session_id != session_id {
                    return;
                }
                panel.last_send_completed = true;
                match result {
                    Ok((handle, result)) => {
                        panel.user_transmit_bytes += result.bytes_written as u64;
                        if panel.port_open_state {
                            panel.port_handle = Some(handle);
                        }
                        let status = match result.error {
                            Some(error) => {
                                panel.stop_timer(cx);
                                panel.last_error = Some(format!(
                                    "发送失败（已发送 {} 字节）：{error}",
                                    result.bytes_written
                                ));
                                format!(" [失败，已发送 {} 字节]", result.bytes_written)
                            }
                            None => String::new(),
                        };
                        let log = format!("{timestamp}{shown}{status}\n");
                        Self::insert_text(
                            log,
                            cx,
                            panel.user_input_show_state.clone(),
                            panel.whether_auto_scroll_to_bottom,
                            panel.window,
                            panel.user_input_max_lines,
                        );
                    }
                    Err(error) => {
                        panel.stop_timer(cx);
                        panel.last_error = Some(format!("发送任务失败：{error}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

impl IoPanel {
    pub fn new(cx: &mut gpui::prelude::Context<Self>, window: &mut Window) -> Self {
        let settings = PreferenceStore::get(cx).clone();
        cx.observe_global::<PreferenceStore>(|_, cx| cx.notify())
            .detach();
        let encoding = settings.encoding;
        let decoding = settings.decoding;

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

        let interval_state = cx.new(|cx| InputState::new(window, cx));
        interval_state.update(cx, |state, cx| {
            state.set_value(settings.interval_ms.to_string(), window, cx)
        });
        IoPanel {
            interval_state,
            timer_payload: None,
            timer_task: None,
            window: window.window_handle(),
            last_send_completed: true,
            port_handle: None,
            port_input_state,
            user_input_show_state,
            user_input_state,
            port_input_focus_handle: cx.focus_handle(),

            user_input_focus_handle: cx.focus_handle(),
            info_config_focus_handle: cx.focus_handle(),
            whether_auto_scroll_to_bottom: settings.auto_scroll,
            whether_add_timestamp: settings.timestamp,
            simple_time_show: settings.simple_time,
            encoding,
            encoder: encoding.encoding().map(|e| e.new_encoder()),
            decoding,
            decoder: decoding.encoding().map(|e| e.new_decoder()),

            auto_tx_append: settings.append,

            _receive_data_subscription: None,
            _open_state_observer_subscription: None,
            port_open_state: false,
            session_id: 0,
            send_cancel: Arc::new(AtomicBool::new(false)),
            last_error: None,
            _encoding_changed_subscription: None,
            _decoding_changed_subscription: None,
            _auto_tx_append_observer_subscription: None,

            port_input_new_line: true,
            port_input_received_bytes: 0,
            port_input_max_lines: ui_config::get().get_common_config().get_max_lines(),
            port_input_line_has_bytes: false,

            user_input_max_lines: ui_config::get().get_common_config().get_max_lines(),
            user_transmit_bytes: 0,
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
    pub fn resolve_user_input_data(encoding: Supported, data: &str) -> Result<Vec<u8>, String> {
        match encoding.encoding() {
            Some(enc) => {
                let (bytes, _, errors) = enc.encode(data);
                if errors {
                    Err("发送文本包含当前编码无法表示的字符".into())
                } else {
                    Ok(bytes.into_owned())
                }
            }
            None => ww_protocol::transfer::parse_hex(data),
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
            } else if text.len() > 1_048_576 {
                let value = state.value();
                let mut offset = value.len().saturating_sub(1_048_576);
                while !value.is_char_boundary(offset) {
                    offset += 1;
                }
                Some(offset)
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
}

impl Drop for IoPanel {
    fn drop(&mut self) {
        self.send_cancel.store(true, Ordering::Release);
    }
}
