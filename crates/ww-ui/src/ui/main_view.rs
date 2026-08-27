use std::time::Duration;

use crate::event::UpdatePortsInfo;
use crate::ui::config_panel::ConfigPanel;
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

impl EventEmitter<UpdatePortsInfo> for MainView {}

pub struct MainView {
    title_bar: Entity<TitleBar>,
    left_focus_handle: FocusHandle,
    right_focus_handle: FocusHandle,
    port_panel: Entity<PortPanel>,
    io_panel: Entity<IoPanel>,
    config_panel: Entity<ConfigPanel>,
    info: Entity<Info>,
}

impl MainView {
    pub fn new(cx: &mut Context<Self>, window: &mut gpui::Window) -> Self {
        let main_view_entity = cx.entity();
        let port_panel = cx.new(|cx| {
            let update_info_sub = cx.subscribe(
                &main_view_entity,
                |this: &mut PortPanel, _main_view, _event: &UpdatePortsInfo, cx| {
                    this.update_info(cx);
                },
            );
            PortPanel::new(window, cx, update_info_sub)
        });
        let port_panel_cloned = port_panel.clone();
        cx.spawn(update_ports_info).detach();
        Self {
            title_bar: cx.new(|cx| TitleBar::new(cx)),
            left_focus_handle: cx.focus_handle(),
            right_focus_handle: cx.focus_handle(),
            port_panel: port_panel,
            config_panel: cx.new(|cx| ConfigPanel::new(window, cx)),
            io_panel: cx.new(|cx| IoPanel::new(cx, port_panel_cloned)),
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
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .p_2()
                                    .h_full()
                                    .w_1_3()
                                    .track_focus(&self.left_focus_handle)
                                    .rounded_md()
                                    .border_1()
                                    .when_else(
                                        self.left_focus_handle.is_focused(window),
                                        |div| {
                                            div.border_color(rgb(config.get_focus_border_color()))
                                        },
                                        |div| {
                                            div.border_color(rgb(config.get_default_border_color()))
                                        },
                                    )
                                    .child(self.port_panel.clone())
                                    .child(self.config_panel.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .h_full()
                                    .w_2_3()
                                    .track_focus(&self.right_focus_handle)
                                    .rounded_md()
                                    .border_1()
                                    .when_else(
                                        self.right_focus_handle.is_focused(window),
                                        |div| {
                                            div.border_color(rgb(config.get_focus_border_color()))
                                        },
                                        |div| {
                                            div.border_color(rgb(config.get_default_border_color()))
                                        },
                                    )
                                    .child(self.io_panel.clone()),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_grow(0.)
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
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}
