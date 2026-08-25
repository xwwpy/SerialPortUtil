use gpui_component::select::SelectItem;
use serde::{Deserialize, Serialize};
use ww_protocol::{SerialPortInfo, SerialPortType};

#[derive(Debug, Clone)]
pub struct PortInfoItem {
    port_name: String,
    port_type: String,
    flag: bool,
}

impl PortInfoItem {
    pub fn get_type_info(port_type: &SerialPortType) -> String {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "u32")]
pub struct BaudRateItem {
    value: u32,
}

impl From<u32> for BaudRateItem {
    fn from(value: u32) -> Self {
        Self { value }
    }
}

impl BaudRateItem {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl SelectItem for BaudRateItem {
    type Value = u32;

    fn title(&self) -> gpui::SharedString {
        format!("{}", self.value()).into()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}
