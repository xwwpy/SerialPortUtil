use std::time::Duration;

use crate::event::UpdatePortsInfo;
use crate::ui::info::Info;
use crate::ui::io_panel::IoPanel;
use crate::ui::port_panel::PortPanel;
use crate::ui::title_bar::TitleBar;
use crate::ui_config;

use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, AsyncApp, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, WeakEntity, Window, blue, div,
    px, rgb,
};

impl EventEmitter<UpdatePortsInfo> for MainView {}

pub struct MainView {
    focus_handle: FocusHandle,
    title_bar: Entity<TitleBar>,
    port_panel: Entity<PortPanel>,
    io_panel: Entity<IoPanel>,
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
        cx.spawn(update_ports_info).detach();
        Self {
            focus_handle: cx.focus_handle(),
            title_bar: cx.new(|cx| TitleBar::new(cx)),
            port_panel: port_panel,
            io_panel: cx.new(|cx| IoPanel::new(cx)),
            info: cx.new(|cx| Info::new(cx)),
        }
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_around()
            .gap_1()
            .child(
                div()
                    .id("TitleBar")
                    .w_full()
                    .flex()
                    .flex_grow(0.)
                    .justify_center()
                    .items_center()
                    .border_color(blue())
                    .border_1()
                    .rounded_md()
                    .child(self.title_bar.clone()),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_grow(1.)
                    .justify_center()
                    .items_center()
                    .track_focus(&self.focus_handle)
                    .when_else(
                        self.focus_handle.is_focused(window),
                        |div| div.border_color(rgb(0x8A2BE2)),
                        |div| div.border_color(blue()),
                    )
                    .border(px(1.3))
                    .rounded_md()
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_start()
                            .child(self.port_panel.clone())
                            .child(self.io_panel.clone()),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_grow(0.)
                    .justify_center()
                    .items_center()
                    .border_color(blue())
                    .border_1()
                    .rounded_md()
                    .child(self.info.clone()),
            )
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
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
