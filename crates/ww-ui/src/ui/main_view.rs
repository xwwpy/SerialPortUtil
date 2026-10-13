use crate::ui_config;
use gpui::StatefulInteractiveElement;
use std::time::Duration;

use crate::event::{OpenStateChanged, ReceivedData, UpdatePortsInfo};
use crate::preferences::{Layout, PreferenceStore, Preset};
use crate::ui::config_panel::{FontConfigPanel, TxRxConfigPanel};
use crate::ui::info::Info;
use crate::ui::io_panel::IoPanel;
use crate::ui::port_panel::PortPanel;
use crate::ui::title_bar::TitleBar;

use gpui::prelude::FluentBuilder;
use gpui::{
    AppContext, AsyncApp, Context, Entity, EventEmitter, FocusHandle, InteractiveElement,
    IntoElement, ParentElement, Render, Styled, WeakEntity, Window, div, rgb, white,
};
use gpui_component::{Disableable, StyledExt};

impl EventEmitter<UpdatePortsInfo> for MainView {}

pub struct MainView {
    title_bar: Entity<TitleBar>,
    left_focus_handle: FocusHandle,
    right_focus_handle: FocusHandle,
    port_panel: Entity<PortPanel>,
    io_panel: Entity<IoPanel>,
    font_config_panel: Entity<FontConfigPanel>,
    tx_rx_config_panel: Entity<TxRxConfigPanel>,
    info: Entity<Info>,
    view_options_open: bool,
}

impl MainView {
    pub fn new(cx: &mut Context<Self>, window: &mut gpui::Window) -> Self {
        cx.observe_global::<PreferenceStore>(|_, cx| cx.notify())
            .detach();
        let main_view_entity = cx.entity();

        let io_panel = cx.new(|cx| IoPanel::new(cx, window));

        let io_panel_weak = io_panel.downgrade();

        let port_panel = cx.new(|cx| {
            let update_info_sub = cx.subscribe(
                &main_view_entity,
                |this: &mut PortPanel, _main_view, _event: &UpdatePortsInfo, cx| {
                    this.update_info(_event, cx);
                },
            );
            PortPanel::new(io_panel_weak, window, cx, update_info_sub)
        });

        cx.spawn(update_ports_info).detach();

        // 订阅port_panel的事件
        io_panel.update(cx, |io_panel, cx| {
            let receive_data_subscription = cx.subscribe(
                &port_panel,
                |this, _port_panel, datas: &ReceivedData, cx| {
                    this.resolve_port_input_data(&datas.data, cx);
                },
            );

            let open_state_subscription = cx.subscribe(
                &port_panel,
                |this, _port_panel, open_state: &OpenStateChanged, cx| {
                    this.port_open_state = open_state.open_state;
                    if open_state.open_state {
                        this.session_id = this.session_id.wrapping_add(1);
                        this.send_cancel =
                            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                        this.last_send_completed = true;
                        this.last_error = None;
                        this.decoder = this
                            .decoding
                            .encoding()
                            .map(|encoding| encoding.new_decoder());
                    }
                    if !open_state.open_state {
                        this.stop_timer(cx);
                        this.send_cancel
                            .store(true, std::sync::atomic::Ordering::Release);
                        drop(this.port_handle.take());
                    }
                    cx.notify();
                },
            );

            io_panel._receive_data_subscription = Some(receive_data_subscription);
            io_panel._open_state_observer_subscription = Some(open_state_subscription);
        });

        let info_entity = cx.new(|cx| Info::new(cx, io_panel.clone()));
        cx.observe(&port_panel, |_, _, cx| cx.notify()).detach();
        let status = info_entity.clone();
        cx.observe(&io_panel, move |_, _, cx| {
            status.update(cx, |_, cx| cx.notify());
            cx.notify();
        })
        .detach();

        Self {
            title_bar: cx.new(|cx| TitleBar::new(cx)),
            left_focus_handle: cx.focus_handle(),
            right_focus_handle: cx.focus_handle(),
            port_panel: port_panel,
            font_config_panel: cx.new(|cx| FontConfigPanel::new(window, cx)),
            tx_rx_config_panel: cx.new(|cx| TxRxConfigPanel::new(window, cx, io_panel.clone())),
            io_panel: io_panel,
            info: info_entity,
            view_options_open: false,
        }
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui::px;
        use gpui_component::{
            button::{Button, ButtonVariants},
            checkbox::Checkbox,
        };
        let settings = PreferenceStore::get(cx).clone();
        let layout = settings.layout;
        let narrow = window.viewport_size().width < px(900.);
        let connected = self.port_panel.read(cx).open_state;
        let closing = self.port_panel.read(cx).closing;
        let opening = self.port_panel.read(cx).opening;
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xf4f5f8))
            .text_color(rgb(0x243047))
            .child(
                div()
                    .w_full()
                    .h(px(34.))
                    .flex_shrink_0()
                    .bg(white())
                    .child(self.title_bar.clone()),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .flex_wrap()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .bg(white())
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(div().font_semibold().child("SerialPortUtil"))
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_sm()
                            .text_color(if connected {
                                rgb(0x16803c)
                            } else {
                                rgb(0x64748b)
                            })
                            .child(if opening {
                                "正在连接"
                            } else if closing {
                                "正在关闭连接"
                            } else if connected {
                                "● 串口已连接"
                            } else {
                                "○ 未连接"
                            }),
                    )
                    .child(
                        Button::new("quick-connect")
                            .label(if connected { "断开" } else { "连接" })
                            .disabled(closing || opening)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.port_panel.update(cx, |panel, cx| {
                                    if panel.open_state {
                                        panel.close_port(cx);
                                    } else {
                                        panel.open_port(cx, window);
                                    }
                                });
                            })),
                    )
                    .child(
                        Button::new("view-options")
                            .label("视图 / 布局")
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.view_options_open = !this.view_options_open;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("restore-layout")
                            .label("恢复默认")
                            .ghost()
                            .on_click(|_, _, cx| {
                                PreferenceStore::update(cx, |s| s.layout = Layout::default())
                            }),
                    ),
            )
            .when(self.view_options_open, |view| {
                let mut options = div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .bg(white())
                    .border_b_1()
                    .border_color(rgb(0xe5e7eb))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .items_center()
                            .child("工作区预设")
                            .children(
                                [
                                    (Preset::Standard, "标准调试"),
                                    (Preset::Analysis, "数据分析"),
                                    (Preset::Compact, "精简"),
                                ]
                                .into_iter()
                                .enumerate()
                                .map(|(i, (preset, label))| {
                                    Button::new(("preset", i)).label(label).on_click(
                                        move |_, _, cx| {
                                            PreferenceStore::update(cx, |s| {
                                                s.layout = Layout::preset(preset)
                                            })
                                        },
                                    )
                                }),
                            ),
                    );
                let mut toggles = div().flex().flex_wrap().gap_4().text_sm();
                for (index, label, checked) in [
                    (0, "串口配置", layout.port),
                    (1, "收发编码", layout.encoding),
                    (2, "字体设置", layout.font),
                    (3, "接收日志", layout.receive),
                    (4, "发送日志", layout.transmit),
                    (5, "发送编辑器", layout.editor),
                    (6, "数据波形", layout.waveform),
                    (7, "状态栏", layout.status),
                ] {
                    toggles = toggles.child(
                        Checkbox::new(("visible", index as usize))
                            .label(label)
                            .checked(checked)
                            .on_click(move |checked, _, cx| {
                                PreferenceStore::update(cx, |s| match index {
                                    0 => s.layout.port = *checked,
                                    1 => s.layout.encoding = *checked,
                                    2 => s.layout.font = *checked,
                                    3 => s.layout.receive = *checked,
                                    4 => s.layout.transmit = *checked,
                                    5 => s.layout.editor = *checked,
                                    6 => s.layout.waveform = *checked,
                                    _ => s.layout.status = *checked,
                                })
                            }),
                    );
                }
                options = options.child(toggles);
                view.child(options)
            })
            .when_some(
                cx.global::<PreferenceStore>().message.clone(),
                |view, message| {
                    view.child(
                        div()
                            .p_2()
                            .text_sm()
                            .text_color(rgb(0xb42318))
                            .child(message),
                    )
                },
            )
            .child(
                div()
                    .id("workspace")
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .flex()
                    .gap_3()
                    .p_3()
                    .when(narrow, |view| view.flex_col().overflow_y_scroll())
                    .when(layout.has_config(), |view| {
                        view.child(
                            div()
                                .id("config-column")
                                .flex()
                                .flex_col()
                                .gap_3()
                                .min_w_0()
                                .when_else(
                                    narrow,
                                    |view| view.w_full().flex_shrink_0(),
                                    |view| {
                                        view.w(px(310.))
                                            .h_full()
                                            .flex_shrink_0()
                                            .overflow_y_scroll()
                                    },
                                )
                                .track_focus(&self.left_focus_handle)
                                .when(layout.port, |view| view.child(self.port_panel.clone()))
                                .when(layout.encoding, |view| {
                                    view.child(self.tx_rx_config_panel.clone())
                                })
                                .when(layout.font, |view| {
                                    view.child(self.font_config_panel.clone())
                                }),
                        )
                    })
                    .when(layout.has_work(), |view| {
                        view.child(
                            div()
                                .id("work-column")
                                .flex_1()
                                .min_w_0()
                                .min_h_0()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .when(narrow, |view| view.flex_shrink_0().min_h(px(420.)))
                                .track_focus(&self.right_focus_handle)
                                .when(layout.receive || layout.transmit || layout.editor, |view| {
                                    view.child(self.io_panel.clone())
                                })
                                .when(layout.waveform, |view| {
                                    view.child(
                                        div()
                                            .p_4()
                                            .bg(white())
                                            .rounded_md()
                                            .child("数据波形面板将在数据分析阶段接入"),
                                    )
                                }),
                        )
                    })
                    .when(!layout.has_work() && !layout.has_config(), |view| {
                        view.child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .items_center()
                                .justify_center()
                                .child("当前工作区的所有面板均已隐藏")
                                .child(
                                    Button::new("empty-restore").label("恢复标准布局").on_click(
                                        |_, _, cx| {
                                            PreferenceStore::update(cx, |s| {
                                                s.layout = Layout::default()
                                            })
                                        },
                                    ),
                                ),
                        )
                    }),
            )
            .when(layout.status, |view| {
                view.child(
                    div()
                        .w_full()
                        .flex_shrink_0()
                        .bg(white())
                        .py_1()
                        .border_t_1()
                        .border_color(rgb(0xe5e7eb))
                        .child(self.info.clone()),
                )
            })
    }
}

async fn update_ports_info(main_view_entity: WeakEntity<MainView>, cx: &mut AsyncApp) {
    let interval = ui_config::get()
        .get_port_panel_config()
        .get_port_update_interval();
    loop {
        let ports = match tokio::task::spawn_blocking(ww_protocol::get_ports).await {
            Ok(result) => result.map_err(|e| e.to_string()),
            Err(error) => Err(error.to_string()),
        };
        let emit_res = main_view_entity.update(cx, move |_, cx| {
            cx.emit(UpdatePortsInfo { ports });
        });
        if emit_res.is_err() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(interval)).await;
    }
}
