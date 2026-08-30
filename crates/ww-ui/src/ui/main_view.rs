use std::time::Duration;

use crate::event::{OpenStateChanged, ReceivedData, UpdatePortsInfo};
use crate::ui::config_panel::{FontConfigPanel, TxRxConfigPanel};
use crate::ui::info::Info;
use crate::ui::io_panel::IoPanel;
use crate::ui::port_panel::PortPanel;
use crate::ui::title_bar::TitleBar;
use crate::ui_config;

use gpui::prelude::FluentBuilder;
use gpui::{
    AppContext, AsyncApp, Context, Entity, EventEmitter, FocusHandle, InteractiveElement,
    IntoElement, ParentElement, Render, Styled, WeakEntity, Window, div, rgb, white,
};
use gpui_component::scroll::ScrollableElement;

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
}

impl MainView {
    pub fn new(cx: &mut Context<Self>, window: &mut gpui::Window) -> Self {
        let main_view_entity = cx.entity();

        let io_panel = cx.new(|cx| IoPanel::new(cx, window));

        let io_panel_weak = io_panel.downgrade();

        let port_panel = cx.new(|cx| {
            let update_info_sub = cx.subscribe(
                &main_view_entity,
                |this: &mut PortPanel, _main_view, _event: &UpdatePortsInfo, cx| {
                    this.update_info(cx);
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
                    drop(this.port_handle.take());
                    cx.notify();
                },
            );

            io_panel._receive_data_subscription = Some(receive_data_subscription);
            io_panel._open_state_observer_subscription = Some(open_state_subscription);
        });

        Self {
            title_bar: cx.new(|cx| TitleBar::new(cx)),
            left_focus_handle: cx.focus_handle(),
            right_focus_handle: cx.focus_handle(),
            port_panel: port_panel,
            font_config_panel: cx.new(|cx| FontConfigPanel::new(window, cx)),
            tx_rx_config_panel: cx.new(|cx| TxRxConfigPanel::new(window, cx, io_panel.clone())),
            io_panel: io_panel,
            info: cx.new(|cx| Info::new(cx)),
        }
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let config = ui_config::get().get_common_config();
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_around()
            .gap_1()
            // title bar
            .child(
                div()
                    .id("TitleBar")
                    .w_full()
                    .flex()
                    .flex_grow(0.)
                    .flex_shrink_0()
                    .justify_center()
                    .items_center()
                    .border_1()
                    .border_color(rgb(config.get_default_border_color()))
                    .bg(white())
                    .shadow_md()
                    .rounded_md()
                    .child(self.title_bar.clone()),
            )
            // main content
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_grow_1()
                    .min_h_0()
                    .justify_center()
                    .items_center()
                    .rounded_md()
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .gap_2()
                            .p_2()
                            .items_center()
                            .justify_start()
                            .child(
                                // 这里套一层是为了解决加上overflow_y_scrollbar导致的宽度异常
                                div().h_full().w_1_3().child(
                                    div()
                                        .size_full()
                                        .flex()
                                        .flex_col()
                                        .gap_4()
                                        .p_2()
                                        .track_focus(&self.left_focus_handle)
                                        .overflow_y_scrollbar()
                                        .rounded_md()
                                        .border_1()
                                        .when_else(
                                            self.left_focus_handle.is_focused(window),
                                            |div| {
                                                div.border_color(rgb(
                                                    config.get_focus_border_color()
                                                ))
                                            },
                                            |div| {
                                                div.border_color(rgb(
                                                    config.get_default_border_color()
                                                ))
                                            },
                                        )
                                        .child(self.port_panel.clone())
                                        .child(self.tx_rx_config_panel.clone())
                                        .child(self.font_config_panel.clone()),
                                ),
                            )
                            .child(
                                div().h_full().w_2_3().child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .size_full()
                                        .gap_4()
                                        .track_focus(&self.right_focus_handle)
                                        .overflow_y_scrollbar()
                                        .rounded_md()
                                        .border_1()
                                        .when_else(
                                            self.right_focus_handle.is_focused(window),
                                            |div| {
                                                div.border_color(rgb(
                                                    config.get_focus_border_color()
                                                ))
                                            },
                                            |div| {
                                                div.border_color(rgb(
                                                    config.get_default_border_color()
                                                ))
                                            },
                                        )
                                        .child(self.io_panel.clone()),
                                ),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_grow_0()
                    .flex_shrink_0()
                    .justify_center()
                    .items_center()
                    .border_1()
                    .border_color(rgb(config.get_default_border_color()))
                    .bg(white())
                    .shadow_md()
                    .rounded_md()
                    .window_control_area(gpui::WindowControlArea::Drag)
                    .child(self.info.clone()),
            )
    }
}

async fn update_ports_info(main_view_entity: WeakEntity<MainView>, cx: &mut AsyncApp) {
    let interval = ui_config::get()
        .get_port_panel_config()
        .get_port_update_interval();
    loop {
        let emit_res = main_view_entity.update(cx, move |_, cx| {
            cx.emit(UpdatePortsInfo {});
        });
        if let Err(e) = emit_res {
            tracing::error!("更新端口信息失败：{e}");
        }
        tokio::time::sleep(Duration::from_millis(interval)).await;
    }
}
