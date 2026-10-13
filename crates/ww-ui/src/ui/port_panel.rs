use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::event::{OpenStateChanged, PortError, ReceivedData, UpdatePortsInfo};
use crate::model::port_model::FlowControlItem;
use crate::model::port_model::{
    BaudRateItem, DataBitsItem, ParityItem, PortInfoItem, PortMessage, StopBitsItem, port_read_loop,
};
use crate::preferences::PreferenceStore;

use crate::ui::io_panel::IoPanel;
use crate::ui_config::{self, LABLE_SIZE};
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, ParentElement, Render, SharedString, Styled, Subscription, Task,
    WeakEntity, Window, div, green, px, rgb, white,
};
use gpui_component::button::Button;
use gpui_component::dialog::DialogButtonProps;
use gpui_component::label::Label;
use gpui_component::select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState};
use gpui_component::{Disableable, IndexPath, WindowExt};

use ww_protocol::model::Ports;

pub struct PortPanel {
    window: AnyWindowHandle,
    ports: Ports,
    io_panel: WeakEntity<IoPanel>,
    port_open_focus_handle: FocusHandle,
    port_config_focus_handle: FocusHandle,
    pub opening: bool,
    pub closing: bool,
    enumeration_error: Option<String>,
    open_task: Option<Task<()>>,
    pub open_state: bool, // 当前串口是否开启

    receive_task: Option<Task<()>>,
    cancel_flag: Arc<AtomicBool>,
    update_info_sub: Option<Subscription>,

    port_info_select: Entity<SelectState<SearchableVec<PortInfoItem>>>,
    band_rate_select: Entity<SelectState<SearchableVec<BaudRateItem>>>,
    parity_select: Entity<SelectState<SearchableVec<ParityItem>>>,
    data_bit_select: Entity<SelectState<SearchableVec<DataBitsItem>>>,
    flow_control_select: Entity<SelectState<SearchableVec<FlowControlItem>>>,
    stop_bit_select: Entity<SelectState<SearchableVec<StopBitsItem>>>,
}

impl EventEmitter<ReceivedData> for PortPanel {}

impl EventEmitter<PortError> for PortPanel {}

impl EventEmitter<OpenStateChanged> for PortPanel {}

impl PortPanel {
    pub fn new(
        io_panel: WeakEntity<IoPanel>,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
        update_info_sub: Subscription,
    ) -> Self {
        let window_handle = window.window_handle();
        cx.subscribe_self(move |_this, event: &PortError, cx| {
            let msg = event.message.clone();
            let _ = window_handle.update(cx, move |_view, window, cx| {
                window.open_alert_dialog(cx, move |alert, _, _cx| {
                    alert
                        .title("出现了错误")
                        .description(SharedString::from(&msg))
                        .button_props(DialogButtonProps::default().ok_text("关闭"))
                        .on_ok(|_, _window, _cx| true)
                });
            });
        })
        .detach();
        let panel = Self {
            window: window.window_handle(),
            io_panel,
            ports: Ports::new(),
            port_open_focus_handle: cx.focus_handle(),
            port_config_focus_handle: cx.focus_handle(),
            opening: false,
            closing: false,
            enumeration_error: None,
            open_task: None,
            open_state: false,
            receive_task: None,
            update_info_sub: Some(update_info_sub),
            port_info_select: cx
                .new(|cx| SelectState::new(SearchableVec::new(vec![]), None, window, cx)),
            band_rate_select: Self::default_baud_rate(window, cx),
            parity_select: Self::get_parity_items(window, cx),
            data_bit_select: Self::get_databits_items(window, cx),
            stop_bit_select: Self::get_stopbits_items(window, cx),
            flow_control_select: cx.new(|cx| {
                let selected = match PreferenceStore::get(cx).flow_control.as_str() {
                    "hardware" => FlowControlItem::Hardware,
                    "software" => FlowControlItem::Software,
                    _ => FlowControlItem::None,
                };
                let mut state = SelectState::new(
                    vec![
                        FlowControlItem::None,
                        FlowControlItem::Hardware,
                        FlowControlItem::Software,
                    ]
                    .into(),
                    None,
                    window,
                    cx,
                );
                state.set_selected_value(&selected, window, cx);
                state
            }),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        };
        cx.subscribe(
            &panel.port_info_select,
            |_, _, event: &SelectEvent<SearchableVec<PortInfoItem>>, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    PreferenceStore::update(cx, |s| s.port_name = value.clone());
                }
            },
        )
        .detach();
        cx.subscribe(
            &panel.band_rate_select,
            |_, _, event: &SelectEvent<SearchableVec<BaudRateItem>>, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    PreferenceStore::update(cx, |s| s.baud_rate = *value);
                }
            },
        )
        .detach();
        cx.subscribe(
            &panel.parity_select,
            |_, _, event: &SelectEvent<SearchableVec<ParityItem>>, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    PreferenceStore::update(cx, |s| {
                        s.parity = match value {
                            ParityItem::None => "none",
                            ParityItem::Odd => "odd",
                            ParityItem::Even => "even",
                        }
                        .into()
                    });
                }
            },
        )
        .detach();
        cx.subscribe(
            &panel.data_bit_select,
            |_, _, event: &SelectEvent<SearchableVec<DataBitsItem>>, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    PreferenceStore::update(cx, |s| {
                        s.data_bits = match value {
                            DataBitsItem::Five => 5,
                            DataBitsItem::Six => 6,
                            DataBitsItem::Seven => 7,
                            DataBitsItem::Eight => 8,
                        }
                    });
                }
            },
        )
        .detach();
        cx.subscribe(
            &panel.stop_bit_select,
            |_, _, event: &SelectEvent<SearchableVec<StopBitsItem>>, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    PreferenceStore::update(cx, |s| {
                        s.stop_bits = match value {
                            StopBitsItem::One => 1,
                            StopBitsItem::Two => 2,
                        }
                    });
                }
            },
        )
        .detach();
        cx.subscribe(
            &panel.flow_control_select,
            |_, _, event: &SelectEvent<SearchableVec<FlowControlItem>>, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    PreferenceStore::update(cx, |s| {
                        s.flow_control = match value {
                            FlowControlItem::None => "none",
                            FlowControlItem::Hardware => "hardware",
                            FlowControlItem::Software => "software",
                        }
                        .into()
                    });
                }
            },
        )
        .detach();
        panel
    }

    pub fn default_baud_rate(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<BaudRateItem>>> {
        let default_select_value = PreferenceStore::get(cx).baud_rate;
        let mut values = ui_config::get()
            .get_port_panel_config()
            .get_baud_rate_default_vec();
        if !values.iter().any(|v| *v.value() == default_select_value) {
            values.push(default_select_value.into());
        }
        let vec = values.into();
        cx.new(move |cx| {
            let mut res = SelectState::new(vec, None, window, cx);
            res.set_selected_value(&default_select_value, window, cx);
            res
        })
    }

    pub fn get_parity_items(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<ParityItem>>> {
        let selected = match PreferenceStore::get(cx).parity.as_str() {
            "odd" => ParityItem::Odd,
            "even" => ParityItem::Even,
            _ => ParityItem::None,
        };
        let vec = vec![ParityItem::None, ParityItem::Odd, ParityItem::Even].into();
        cx.new(move |cx| {
            let mut res = SelectState::new(vec, None, window, cx);
            res.set_selected_value(&selected, window, cx);
            res
        })
    }

    pub fn get_databits_items(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<DataBitsItem>>> {
        let selected = match PreferenceStore::get(cx).data_bits {
            5 => DataBitsItem::Five,
            6 => DataBitsItem::Six,
            7 => DataBitsItem::Seven,
            _ => DataBitsItem::Eight,
        };
        let vec = vec![
            DataBitsItem::Five,
            DataBitsItem::Six,
            DataBitsItem::Seven,
            DataBitsItem::Eight,
        ]
        .into();
        cx.new(move |cx| {
            let mut res = SelectState::new(vec, None, window, cx);
            res.set_selected_value(&selected, window, cx);
            res
        })
    }

    pub fn get_stopbits_items(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<StopBitsItem>>> {
        let selected = if PreferenceStore::get(cx).stop_bits == 2 {
            StopBitsItem::Two
        } else {
            StopBitsItem::One
        };
        let vec = vec![StopBitsItem::One, StopBitsItem::Two].into();
        cx.new(move |cx| {
            let mut res = SelectState::new(vec, None, window, cx);
            res.set_selected_value(&selected, window, cx);
            res
        })
    }

    pub fn update_info(&mut self, event: &UpdatePortsInfo, cx: &mut Context<Self>) {
        let ports = match &event.ports {
            Ok(ports) => {
                if self.enumeration_error.take().is_some() {
                    cx.notify();
                }
                ports.clone()
            }
            Err(error) => {
                if self.enumeration_error.as_ref() != Some(error) {
                    self.enumeration_error = Some(error.clone());
                    cx.notify();
                }
                return;
            }
        };

        // 如果没有发生变化，就直接返回
        let old_ports = self.ports.get_ports().unwrap();
        if old_ports.len() == ports.len()
            && old_ports
                .iter()
                .zip(&ports)
                .all(|(old, new)| old.port_name == new.port_name)
        {
            return;
        } else {
            tracing::info!("ports info changed");
        }

        // 更新内部保存的串口信息数组
        self.ports.update_ports_info(ports.clone());

        // 获取当前选中的串口信息
        let selected_value = self
            .port_info_select
            .read(cx)
            .selected_value()
            .cloned()
            .or_else(|| Some(PreferenceStore::get(cx).port_name.clone()));

        // 根据选中的串口信息计算新的索引和选中值
        // 只有当不存在的时候才会取消选中
        let tmp = match selected_value {
            Some(v) => ports
                .iter()
                .enumerate()
                .find(|(_, port)| port.port_name == v),

            None => None,
        };

        // 计算新的选中值和索引
        let (_, new_index) = match tmp {
            Some(v) => (Some(v.1.clone()), Some(IndexPath::new(v.0))),
            None => (None, None),
        };

        // if let Some(selected_value) = selected_value {
        //     Self::select_port(selected_value);
        // }

        // 构建新的串口信息列表
        let items: SearchableVec<PortInfoItem> = ports
            .iter()
            .map(|item| item.into())
            .collect::<Vec<_>>()
            .into();

        let window = self.window;
        let select = self.port_info_select.clone();

        // 用保存的窗口句柄拿到 &mut Window，再 set_items
        let _ = window.update(cx, |_, window, cx| {
            select.update(cx, |state, cx| {
                state.set_items(items, window, cx);

                state.set_selected_index(new_index, window, cx);
            });
        });

        cx.notify();
    }

    pub fn close_subscription(&mut self) {
        let _ = self.update_info_sub.take();
    }

    pub fn open_port(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        if self.opening || self.closing || self.open_state {
            return;
        }
        // 更新选中的串口
        // 获取串口名
        let port_name = self.port_info_select.read(cx).selected_value();
        if port_name == None {
            window.open_alert_dialog(cx, |alert, _, _cx| {
                alert
                    .title("未选择串口")
                    .description("请选择串口后再尝试打开串口")
                    .button_props(DialogButtonProps::default().ok_text("关闭"))
                    .on_ok(|_, _window, _cx| true)
            });
            return;
        }

        let selected_port_name = port_name.unwrap().clone();
        // 获取当前选中的波特率
        let baud_rate = self.band_rate_select.read(cx).selected_value().unwrap();
        // 获取当前选中的校验位
        let parity = self.parity_select.read(cx).selected_value().unwrap();
        // 获取当前选中的数据位
        let data_bits = self.data_bit_select.read(cx).selected_value().unwrap();
        // 获取当前选中的停止位
        let stop_bits = self.stop_bit_select.read(cx).selected_value().unwrap();

        let read_timeout = ui_config::get()
            .get_port_panel_config()
            .get_read_timeout_timeout();

        if let Err(e) = self.ports.select_port(
            selected_port_name,
            baud_rate.clone(),
            parity.clone(),
            data_bits.clone(),
            stop_bits.clone(),
            self.flow_control_select
                .read(cx)
                .selected_value()
                .copied()
                .unwrap_or(FlowControlItem::None),
            read_timeout,
        ) {
            cx.emit(PortError {
                message: e.to_string(),
            });
            return;
        }
        let Some(builder) = self.ports.get_current_select_port().cloned() else {
            return;
        };
        self.opening = true;
        cx.notify();
        self.open_task = Some(cx.spawn(async move |entity, cx| {
            let result = tokio::task::spawn_blocking(move || {
                let port = builder.open().map_err(|e| e.to_string())?;
                let writer = port.try_clone().map_err(|e| e.to_string())?;
                Ok::<_, String>((port, writer))
            })
            .await;
            let _ = entity.update(cx, |panel, cx| {
                panel.opening = false;
                match result {
                    Ok(Ok((port, writer))) => panel.start_reader(port, writer, cx),
                    Ok(Err(message)) => cx.emit(PortError { message }),
                    Err(error) => cx.emit(PortError {
                        message: error.to_string(),
                    }),
                }
                cx.notify();
            });
        }));
    }

    fn start_reader(
        &mut self,
        port: Box<dyn ww_protocol::SerialPort>,
        port_handle: Box<dyn ww_protocol::SerialPort>,
        cx: &mut Context<Self>,
    ) {
        // 将端口句柄设置到 io_panel 中
        let _ = self.io_panel.update(cx, |io_panel, _cx| {
            io_panel.port_handle = Some(port_handle);
        });

        self.cancel_flag = Arc::new(AtomicBool::new(false));
        self.change_open_state(true, cx);
        // 有界队列通过背压限制内存，关闭时保留消费者直到后台释放串口。
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PortMessage>(128);

        // 在独立线程执行阻塞读取，避免卡住 UI 主线程
        let cancel_flag_cloned = self.cancel_flag.clone();
        std::thread::spawn(move || port_read_loop(port, tx, cancel_flag_cloned));

        // 在主线程异步消费读取结果
        let task = cx.spawn(async move |port_panel, cx| {
            while let Some(message) = rx.recv().await {
                match message {
                    PortMessage::Data(data) => {
                        let _ = port_panel.update(cx, |_this, cx| {
                            if _this.open_state {
                                cx.emit(ReceivedData { data });
                            }
                        });
                    }
                    PortMessage::Error(msg) => {
                        let _ = port_panel.update(cx, |this, cx| {
                            this.closing = true;
                            this.change_open_state(false, cx);
                            cx.emit(PortError { message: msg });
                            cx.notify();
                        });
                    }
                    PortMessage::Closed => {
                        loop {
                            let sending = port_panel.read_with(cx, |panel, cx| {
                                panel
                                    .io_panel
                                    .upgrade()
                                    .is_some_and(|io| !io.read(cx).last_send_completed)
                            });
                            if !sending.unwrap_or(false) {
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                        }
                        let _ = port_panel.update(cx, |this, cx| {
                            this.closing = false;
                            this.change_open_state(false, cx);
                        });
                        break;
                    }
                }
            }
        });

        self.receive_task = Some(task);

        // 更新界面
        cx.notify();
    }

    pub fn close_port(&mut self, cx: &mut Context<Self>) {
        self.closing = true;
        self.cancel_flag
            .store(true, std::sync::atomic::Ordering::Release);
        self.change_open_state(false, cx);
        cx.notify();
    }

    pub fn change_open_state(&mut self, new_open_state: bool, cx: &mut Context<Self>) {
        self.open_state = new_open_state;

        cx.emit(OpenStateChanged {
            open_state: new_open_state,
        });

        cx.notify();
    }

    fn focus_open_port_info(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.port_open_focus_handle.focus(window, cx);
    }
}

impl Render for PortPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let config = ui_config::get().get_common_config();
        div()
            .id("portPanel")
            .flex()
            .flex_col()
            .w_full()
            .gap_4()
            .when_some(self.enumeration_error.clone(), |view, error| {
                view.child(div().p_2().text_color(rgb(0xb42318)).child(error))
            })
            .child(
                // 串口选择信息和打开按钮
                div().p_2().bg(white()).shadow_md().rounded_md().child(
                    div()
                        .flex()
                        .w_full()
                        .items_center()
                        .justify_around()
                        .p_4()
                        .gap_4()
                        .overflow_hidden()
                        .flex_wrap()
                        .border_1()
                        .track_focus(&self.port_open_focus_handle)
                        .when_else(
                            self.port_open_focus_handle.is_focused(window),
                            |div| div.border_color(rgb(config.get_focus_border_color())),
                            |div| div.border_color(rgb(config.get_default_border_color())),
                        )
                        .rounded_md()
                        // .bg(white())
                        // .shadow_md()
                        .child(
                            Label::new(match self.port_info_select.read(cx).selected_value() {
                                Some(port_name) => format!("当前串口: {}", port_name),
                                None => "未选择串口...".into(),
                            })
                            .text_ellipsis()
                            .text_color(green()),
                        )
                        .child(
                            // 打开和关闭串口按钮
                            div().child(
                                Button::new("open/close")
                                    .size(window.rem_size() * 2.)
                                    .flex_shrink_0()
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        if this.open_state {
                                            this.close_port(cx);
                                        } else {
                                            this.open_port(cx, window);
                                        }
                                        this.focus_open_port_info(window, cx);
                                        cx.notify();
                                    }))
                                    .rounded_full()
                                    .p_1()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(self.open_state, |this| this.border_color(green()))
                                    .child(
                                        div()
                                            .size_full()
                                            .rounded_full()
                                            .when(self.open_state, |this| this.bg(green())),
                                    )
                                    .when_else(
                                        self.port_info_select.read(cx).selected_value().is_none()
                                            || self.closing
                                            || self.opening,
                                        |this| this.disabled(true).cursor_not_allowed(),
                                        |this| this.disabled(false).cursor_pointer(),
                                    ),
                            ),
                        ),
                ),
            )
            // 串口配置选项卡
            .child(
                div().p_2().bg(white()).shadow_md().rounded_md().child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .p_4()
                        .gap_4()
                        .items_start()
                        .justify_start()
                        .overflow_hidden()
                        .border_1()
                        .track_focus(&self.port_config_focus_handle)
                        .when_else(
                            self.port_config_focus_handle.is_focused(window)
                                || self.band_rate_select.focus_handle(cx).is_focused(window)
                                || self.data_bit_select.focus_handle(cx).is_focused(window)
                                || self.parity_select.focus_handle(cx).is_focused(window)
                                || self.port_info_select.focus_handle(cx).is_focused(window)
                                || self.stop_bit_select.focus_handle(cx).is_focused(window),
                            |div| div.border_color(rgb(config.get_focus_border_color())),
                            |div| div.border_color(rgb(config.get_default_border_color())),
                        )
                        .rounded_md()
                        // .bg(white())
                        // .shadow_md()
                        .child(
                            div()
                                .flex()
                                .w_full()
                                .items_center()
                                .overflow_hidden()
                                .flex_wrap()
                                .child(
                                    Label::new("串口号：")
                                        .w(px(LABLE_SIZE))
                                        // 这里设置flex_shrink_0和flex_grow_0是为了防止标签被压缩或拉伸
                                        .flex_shrink_0()
                                        .flex_grow_0(),
                                )
                                .child(
                                    // 这里套一层是为了正确的控制缩放关系，Select内部使用SizeFull，如果不再套一层布局会有问题
                                    // 如果需要完整显示Select，不让其超出父元素范围，可以加上overflow_hidden
                                    div().flex_1().child(
                                        Select::new(&self.port_info_select)
                                            .w_full()
                                            .when_else(
                                                !self.open_state,
                                                |select| select.cursor_pointer(),
                                                |select| select.cursor_not_allowed(),
                                            )
                                            .disabled(
                                                self.open_state || self.opening || self.closing,
                                            )
                                            .placeholder("选择Com"),
                                    ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .w_full()
                                .items_center()
                                .overflow_hidden()
                                .flex_wrap()
                                .child(
                                    Label::new("波特率：")
                                        .w(px(LABLE_SIZE))
                                        .flex_shrink_0()
                                        .flex_grow_0(),
                                )
                                .child(
                                    div().flex_1().child(
                                        Select::new(&self.band_rate_select)
                                            .w_full()
                                            .when_else(
                                                !self.open_state,
                                                |select| select.cursor_pointer(),
                                                |select| select.cursor_not_allowed(),
                                            )
                                            .disabled(
                                                self.open_state || self.opening || self.closing,
                                            )
                                            .placeholder("选择波特率"),
                                    ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .w_full()
                                .items_center()
                                .overflow_hidden()
                                .flex_wrap()
                                .child(
                                    Label::new("校验位：")
                                        .w(px(LABLE_SIZE))
                                        .flex_shrink_0()
                                        .flex_grow_0(),
                                )
                                .child(
                                    div().flex_1().child(
                                        Select::new(&self.parity_select)
                                            .w_full()
                                            .when_else(
                                                !self.open_state,
                                                |select| select.cursor_pointer(),
                                                |select| select.cursor_not_allowed(),
                                            )
                                            .disabled(
                                                self.open_state || self.opening || self.closing,
                                            )
                                            .placeholder("选择校验位"),
                                    ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .w_full()
                                .items_center()
                                .overflow_hidden()
                                .flex_wrap()
                                .child(
                                    Label::new("数据位数：")
                                        .w(px(LABLE_SIZE))
                                        .flex_shrink_0()
                                        .flex_grow_0(),
                                )
                                .child(
                                    div().flex_1().child(
                                        Select::new(&self.data_bit_select)
                                            .w_full()
                                            .when_else(
                                                !self.open_state,
                                                |select| select.cursor_pointer(),
                                                |select| select.cursor_not_allowed(),
                                            )
                                            .disabled(
                                                self.open_state || self.opening || self.closing,
                                            )
                                            .placeholder("选择数据位个数"),
                                    ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .w_full()
                                .items_center()
                                .overflow_hidden()
                                .flex_wrap()
                                .child(
                                    Label::new("停止位数：")
                                        .w(px(LABLE_SIZE))
                                        .flex_shrink_0()
                                        .flex_grow_0(),
                                )
                                .child(
                                    div().flex_1().child(
                                        Select::new(&self.stop_bit_select)
                                            .w_full()
                                            .when_else(
                                                !self.open_state,
                                                |select| select.cursor_pointer(),
                                                |select| select.cursor_not_allowed(),
                                            )
                                            .disabled(
                                                self.open_state || self.opening || self.closing,
                                            )
                                            .placeholder("选择停止位个数"),
                                    ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .w_full()
                                .items_center()
                                .flex_wrap()
                                .child(Label::new("流控：").w(px(LABLE_SIZE)).flex_shrink_0())
                                .child(
                                    div().flex_1().child(
                                        Select::new(&self.flow_control_select)
                                            .w_full()
                                            .disabled(self.open_state || self.closing)
                                            .placeholder("选择流控"),
                                    ),
                                ),
                        ),
                ),
            )
    }
}

impl Drop for PortPanel {
    fn drop(&mut self) {
        self.cancel_flag
            .store(true, std::sync::atomic::Ordering::Release);
    }
}
