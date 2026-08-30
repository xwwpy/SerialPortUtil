use std::sync::{Arc, atomic::AtomicBool};

use gpui_component::select::SelectItem;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use ww_protocol::{DataBits, Parity, SerialPort, SerialPortInfo, SerialPortType, StopBits};

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

#[derive(Debug, Clone, PartialEq)]
pub enum ParityItem {
    None,
    Odd,
    Even,
}

impl Into<Parity> for ParityItem {
    fn into(self) -> Parity {
        match self {
            Self::None => Parity::None,
            Self::Odd => Parity::Odd,
            Self::Even => Parity::Even,
        }
    }
}

impl SelectItem for ParityItem {
    type Value = Self;

    fn title(&self) -> gpui::SharedString {
        match self {
            Self::None => "None".into(),
            Self::Odd => "Odd".into(),
            Self::Even => "Even".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataBitsItem {
    Five,
    Six,
    Seven,
    Eight,
}

impl SelectItem for DataBitsItem {
    type Value = Self;

    fn title(&self) -> gpui::SharedString {
        match self {
            Self::Five => "5".into(),
            Self::Six => "6".into(),
            Self::Seven => "7".into(),
            Self::Eight => "8".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

impl Into<DataBits> for DataBitsItem {
    fn into(self) -> DataBits {
        match self {
            Self::Five => DataBits::Five,
            Self::Six => DataBits::Six,
            Self::Seven => DataBits::Seven,
            Self::Eight => DataBits::Eight,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopBitsItem {
    One,
    Two,
}

impl Into<StopBits> for StopBitsItem {
    fn into(self) -> StopBits {
        match self {
            Self::One => StopBits::One,
            Self::Two => StopBits::Two,
        }
    }
}

impl SelectItem for StopBitsItem {
    type Value = Self;

    fn title(&self) -> gpui::SharedString {
        match self {
            Self::One => "1".into(),
            Self::Two => "2".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

/// 后台读取线程与主线程之间传递的消息
pub enum PortMessage {
    Data(Vec<u8>),
    Error(String),
}

pub fn port_read_loop(
    mut port_handle: Box<dyn SerialPort>,
    tx: UnboundedSender<PortMessage>,
    cancel_flag: Arc<AtomicBool>,
) {
    let mut buf = [0u8; 1024];
    loop {
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        match port_handle.read(&mut buf) {
            Ok(n) if n > 0 => {
                if tx.send(PortMessage::Data(buf[..n].to_vec())).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                let _ = tx.send(PortMessage::Error(e.to_string()));
                break;
            }
        }
    }
    drop(port_handle);
    cancel_flag.store(false, std::sync::atomic::Ordering::Relaxed);
}
