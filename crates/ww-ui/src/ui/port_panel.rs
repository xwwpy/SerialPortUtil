use crate::model::port_model::{
    BaudRateItem, DataBitsItem, ParityItem, PortInfoItem, StopBitsItem, port_task,
};
use crate::ui_config;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    ParentElement, Render, Styled, Subscription, Task, TextOverflow, Window, blue, div, green, px,
    white,
};
use gpui_component::button::Button;
use gpui_component::label::Label;
use gpui_component::select::{SearchableVec, Select, SelectState};
use gpui_component::{Disableable, IndexPath, Sizable};

use ww_protocol::{get_ports, model::Ports};

pub struct PortPanel {
    window: AnyWindowHandle,
    ports: Ports,
    focus_handle: FocusHandle,
    open_state: bool, // 当前串口是否开启

    port_handle_task: Option<Task<()>>,

    update_info_sub: Option<Subscription>,

    port_info_select: Entity<SelectState<SearchableVec<PortInfoItem>>>,
    band_rate_select: Entity<SelectState<SearchableVec<BaudRateItem>>>,
    parity_select: Entity<SelectState<SearchableVec<ParityItem>>>,
    data_bit_select: Entity<SelectState<SearchableVec<DataBitsItem>>>,
    stop_bit_select: Entity<SelectState<SearchableVec<StopBitsItem>>>,
}

impl Focusable for PortPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PortPanel {
    pub fn new(
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
        update_info_sub: Subscription,
    ) -> Self {
        Self {
            window: window.window_handle(),
            ports: Ports::new(),
            focus_handle: cx.focus_handle(),
            open_state: false,
            port_handle_task: None,
            update_info_sub: Some(update_info_sub),
            port_info_select: cx
                .new(|cx| SelectState::new(SearchableVec::new(vec![]), None, window, cx)),
            band_rate_select: Self::default_baud_rate(window, cx),
            parity_select: Self::get_parity_items(window, cx),
            data_bit_select: Self::get_databits_items(window, cx),
            stop_bit_select: Self::get_stopbits_items(window, cx),
        }
    }

    pub fn default_baud_rate(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<BaudRateItem>>> {
        let config = ui_config::get();
        let vec = config
            .get_port_panel_config()
            .get_baud_rate_default_vec()
            .into();
        let default_select_value = config.get_port_panel_config().get_default_baud_rate();
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
        let vec = vec![ParityItem::None, ParityItem::Odd, ParityItem::Even].into();
        cx.new(move |cx| {
            let mut res = SelectState::new(vec, None, window, cx);
            res.set_selected_value(&ParityItem::None, window, cx);
            res
        })
    }

    pub fn get_databits_items(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<DataBitsItem>>> {
        let vec = vec![
            DataBitsItem::Five,
            DataBitsItem::Six,
            DataBitsItem::Seven,
            DataBitsItem::Eight,
        ]
        .into();
        cx.new(move |cx| {
            let mut res = SelectState::new(vec, None, window, cx);
            res.set_selected_value(&DataBitsItem::Eight, window, cx);
            res
        })
    }

    pub fn get_stopbits_items(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<StopBitsItem>>> {
        let vec = vec![StopBitsItem::One, StopBitsItem::Two].into();
        cx.new(move |cx| {
            let mut res = SelectState::new(vec, None, window, cx);
            res.set_selected_value(&StopBitsItem::One, window, cx);
            res
        })
    }

    pub fn update_info(&mut self, cx: &mut Context<Self>) {
        // 获取最新的串口信息
        let ports = get_ports();

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
        let selected_value = self.port_info_select.read(cx).selected_value();

        // 根据选中的串口信息计算新的索引和选中值
        // 只有当不存在的时候才会取消选中
        let tmp = match selected_value {
            Some(v) => ports
                .iter()
                .enumerate()
                .find(|(_, port)| port.port_name == *v),

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

    pub fn open_port(&mut self, cx: &mut Context<Self>) {
        // 更新选中的串口
        // 获取串口名
        let port_name = self.port_info_select.read(cx).selected_value();
        if port_name == None {
            // TODO 发送打开失败事件
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
            read_timeout,
        ) {
            tracing::error!("Failed to select port: {:?}", e);
            return;
        }
        // 打开串口
        self.open_state = true;
        let port = self.ports.open_port();
        if let Err(e) = port {
            tracing::error!("Failed to open port: {:?}", e);
            self.open_state = false;
            return;
        }

        let port = port.unwrap();

        let task = cx.spawn(async move |port_panel, cx| port_task(port, port_panel, cx).await);

        self.port_handle_task = Some(task);

        // 更新界面
        cx.notify();
    }

    pub fn close_port(&mut self, cx: &mut Context<Self>) {
        // 关闭异步任务
        self.port_handle_task.take();
        self.open_state = false;
        cx.notify();
    }
}

impl Render for PortPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        div()
            .id("portPanel")
            .flex()
            .flex_col()
            .h_full()
            .w_1_3()
            .p_2()
            .gap_4()
            .rounded_md()
            .border_1()
            .border_color(blue())
            .child(
                div()
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_around()
                    .p_4()
                    .gap_4()
                    .overflow_hidden()
                    .border_1()
                    .border_color(green())
                    .rounded_md()
                    .bg(white())
                    .shadow_md()
                    .child(Label::new(
                        match self.port_info_select.read(cx).selected_value() {
                            Some(port_name) => port_name.clone(),
                            None => "未选择串口...".into(),
                        },
                    ))
                    .child(
                        div().child(
                            Button::new("open/close")
                                .size(window.rem_size() * 2.)
                                .flex_shrink_0()
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    if this.open_state {
                                        this.close_port(cx);
                                    } else {
                                        this.open_port(cx);
                                    }
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
                                    self.port_info_select.read(cx).selected_value().is_none(),
                                    |this| this.disabled(true).cursor_not_allowed(),
                                    |this| this.disabled(false).cursor_pointer(),
                                ),
                        ),
                    ),
            )
            // 串口配置选项卡
            .child(
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
                    .border_color(green())
                    .rounded_md()
                    .bg(white())
                    .shadow_md()
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .child(
                                Label::new("串口号：")
                                    .w(px(100.))
                                    // 这里设置flex_shrink_0和flex_grow_0是为了防止标签被压缩或拉伸
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                // 这里套一层是为了正确的控制缩放关系，Select内部使用SizeFull，如果不再套一层布局会有问题
                                div().flex_1().overflow_hidden().child(
                                    Select::new(&self.port_info_select)
                                        .w_full()
                                        .text_ellipsis()
                                        .when_else(
                                            !self.open_state,
                                            |select| select.cursor_pointer(),
                                            |select| select.cursor_not_allowed(),
                                        )
                                        .disabled(self.open_state)
                                        .placeholder("选择Com"),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .child(
                                Label::new("波特率：")
                                    .w(px(100.))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().overflow_hidden().child(
                                    Select::new(&self.band_rate_select)
                                        .w_full()
                                        .text_ellipsis()
                                        .when_else(
                                            !self.open_state,
                                            |select| select.cursor_pointer(),
                                            |select| select.cursor_not_allowed(),
                                        )
                                        .disabled(self.open_state)
                                        .placeholder("选择波特率"),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .child(
                                Label::new("校验位：")
                                    .w(px(100.))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().overflow_hidden().child(
                                    Select::new(&self.parity_select)
                                        .w_full()
                                        .text_ellipsis()
                                        .when_else(
                                            !self.open_state,
                                            |select| select.cursor_pointer(),
                                            |select| select.cursor_not_allowed(),
                                        )
                                        .disabled(self.open_state)
                                        .placeholder("选择校验位"),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .child(
                                Label::new("数据位数：")
                                    .w(px(100.))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().overflow_hidden().child(
                                    Select::new(&self.data_bit_select)
                                        .w_full()
                                        .text_ellipsis()
                                        .when_else(
                                            !self.open_state,
                                            |select| select.cursor_pointer(),
                                            |select| select.cursor_not_allowed(),
                                        )
                                        .disabled(self.open_state)
                                        .placeholder("选择数据位个数"),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .child(
                                Label::new("停止位数：")
                                    .w(px(100.))
                                    .flex_shrink_0()
                                    .flex_grow_0(),
                            )
                            .child(
                                div().flex_1().overflow_hidden().child(
                                    Select::new(&self.stop_bit_select)
                                        .w_full()
                                        .text_ellipsis()
                                        .when_else(
                                            !self.open_state,
                                            |select| select.cursor_pointer(),
                                            |select| select.cursor_not_allowed(),
                                        )
                                        .disabled(self.open_state)
                                        .placeholder("选择停止位个数"),
                                ),
                            ),
                    ),
                // .child(
                //     div().flex().w_full().items_center().child(
                //         Label::new("数据流控：")
                //             .w(px(100.))
                //             .flex_shrink_0()
                //             .flex_grow_0(),
                //     ),
                // )
                // .child(
                //     div().flex().w_full().items_center().child(
                //         Label::new("流控信号：")
                //             .w(px(100.))
                //             .flex_shrink_0()
                //             .flex_grow_0(),
                //     ),
                // ),
            )
    }
}
