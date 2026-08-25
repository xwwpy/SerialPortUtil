use crate::common::color::white;
use crate::model::port_model::{BaudRateItem, PortInfoItem};
use crate::ui_config;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    ParentElement, Render, Styled, Subscription, Window, blue, div, green, px,
};
use gpui_component::IndexPath;
use gpui_component::label::Label;
use gpui_component::select::{SearchableVec, Select, SelectState};

use ww_protocol::{get_ports, model::Ports};

pub struct PortPanel {
    window: AnyWindowHandle,
    ports: Ports,
    focus_handle: FocusHandle,
    open_state: bool, // 当前串口是否开启
    update_info_sub: Option<Subscription>,
    port_info_select: Entity<SelectState<SearchableVec<PortInfoItem>>>,
    band_rate_select: Entity<SelectState<SearchableVec<BaudRateItem>>>,
    // date_flow_control_select: Entity<SelectState<SearchableVec>>,
    // parity_select: Entity<SelectState<SearchableVec>>,
    // data_bit_select: Entity<SelectState<SearchableVec>>,
    // stop_bit_select: Entity<SelectState<SearchableVec>>,
    // flow_control_signal_select: Entity<SelectState<SearchableVec>>,
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
            update_info_sub: Some(update_info_sub),
            port_info_select: cx
                .new(|cx| SelectState::new(SearchableVec::new(vec![]), None, window, cx)),
            band_rate_select: Self::default_baud_rate(window, cx),
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

    pub fn update_info(&mut self, cx: &mut Context<Self>) {
        // 获取最新的串口信息
        let ports = get_ports();

        // 如果没有发生变化，就直接返回
        if self
            .ports
            .get_ports()
            .unwrap()
            .iter()
            .zip(&ports)
            .any(|(old, new)| old.port_name != new.port_name)
        {
            tracing::info!("ports info not changed");
            return;
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
        let (selected_value, new_index) = match tmp {
            Some(v) => (Some(v.1.clone()), Some(IndexPath::new(v.0))),
            None => (None, None),
        };

        if let Some(selected_value) = selected_value {
            // 更新选中的串口
            if let Err(e) = self.ports.select_port(selected_value.port_name) {
                // 不会进入此流程
                tracing::error!("{}", e);
                return;
            }
        }

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
}

impl Render for PortPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        div()
            .id("portPanel")
            .h_full()
            .w_1_3()
            .p_2()
            .rounded_md()
            .border_1()
            .border_color(blue())
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
                                div().flex_1().child(
                                    Select::new(&self.port_info_select)
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
                                div().flex_1().child(
                                    Select::new(&self.band_rate_select)
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
                        div().flex().w_full().items_center().child(
                            Label::new("数据流控：")
                                .w(px(100.))
                                .flex_shrink_0()
                                .flex_grow_0(),
                        ),
                    )
                    .child(
                        div().flex().w_full().items_center().child(
                            Label::new("校验位：")
                                .w(px(100.))
                                .flex_shrink_0()
                                .flex_grow_0(),
                        ),
                    )
                    .child(
                        div().flex().w_full().items_center().child(
                            Label::new("数据位数：")
                                .w(px(100.))
                                .flex_shrink_0()
                                .flex_grow_0(),
                        ),
                    )
                    .child(
                        div().flex().w_full().items_center().child(
                            Label::new("停止位数：")
                                .w(px(100.))
                                .flex_shrink_0()
                                .flex_grow_0(),
                        ),
                    )
                    .child(
                        div().flex().w_full().items_center().child(
                            Label::new("流控信号：")
                                .w(px(100.))
                                .flex_shrink_0()
                                .flex_grow_0(),
                        ),
                    ),
            )
    }
}
