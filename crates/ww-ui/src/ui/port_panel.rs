use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    ParentElement, Render, Styled, Subscription, blue, div,
};
use gpui_component::IndexPath;
use gpui_component::select::SelectItem;
use gpui_component::select::{SearchableVec, Select, SelectState};
use ww_protocol::{SerialPortInfo, SerialPortType, model::Ports};

#[derive(Debug, Clone)]
pub struct PortInfoItem {
    port_name: String,
    port_type: String,
    flag: bool,
}

impl PortInfoItem {
    fn get_type_info(port_type: &SerialPortType) -> String {
        match port_type {
            SerialPortType::UsbPort(usb_info) => usb_info
                .clone()
                .product
                .unwrap_or_else(|| "UnKnown".to_string()),
            _ => format!("{:?}", port_type),
        }
    }
}

impl From<&SerialPortInfo> for PortInfoItem {
    fn from(port: &SerialPortInfo) -> Self {
        Self {
            port_name: port.port_name.clone(),
            port_type: Self::get_type_info(&port.port_type),
            flag: match port.port_type {
                SerialPortType::UsbPort(_) => true,
                _ => false,
            },
        }
    }
}

impl SelectItem for PortInfoItem {
    type Value = String;

    fn title(&self) -> gpui::SharedString {
        // 为了统一显示信息格式
        if self.flag {
            format!("{}", self.port_type).into()
        } else {
            format!("{}({})", self.port_type, self.port_name).into()
        }
    }

    fn value(&self) -> &Self::Value {
        &self.port_name
    }
}

pub struct PortPanel {
    window: AnyWindowHandle,
    ports: Ports,
    focus_handle: FocusHandle,
    update_info_sub: Option<Subscription>,
    port_info_select: Entity<SelectState<SearchableVec<PortInfoItem>>>,
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
            update_info_sub: Some(update_info_sub),
            port_info_select: cx
                .new(|cx| SelectState::new(SearchableVec::new(vec![]), None, window, cx)),
        }
    }

    pub fn update_info(&mut self, ports: &[SerialPortInfo], cx: &mut Context<Self>) {
        self.ports.update_ports_info(ports.to_vec());

        let selected_value = self.port_info_select.read(cx).selected_value();

        let new_index = match selected_value {
            Some(v) => ports
                .iter()
                .enumerate()
                .find(|(_, port)| port.port_name == *v),

            None => None,
        };

        let new_index = match new_index {
            Some(v) => Some(IndexPath::new(v.0)),
            None => None,
        };

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
            .border_1()
            .border_color(blue())
            .child(
                div()
                    .flex()
                    .m_2()
                    .gap_2()
                    .items_start()
                    .justify_center()
                    .border_1()
                    .border_color(blue())
                    .child(
                        Select::new(&self.port_info_select)
                            .cursor_pointer()
                            .placeholder("选择Com"),
                    ),
            )
    }
}
