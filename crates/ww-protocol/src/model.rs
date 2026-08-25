use std::borrow::Cow;

use crate::common::error::ProtocolError;
use serialport::{SerialPortBuilder, SerialPortInfo};

#[derive(Debug, Default)]
pub struct Ports {
    current_select_port: Option<Port>,
    ports: Option<Vec<SerialPortInfo>>,
}

impl Ports {
    pub fn new() -> Self {
        Self {
            current_select_port: None,
            ports: Some(vec![]),
        }
    }

    pub fn get_ports(&self) -> Option<&Vec<SerialPortInfo>> {
        self.ports.as_ref()
    }

    pub fn update_ports_info(self: &mut Self, ports: Vec<SerialPortInfo>) {
        self.ports = Some(ports);
    }

    pub fn select_port(self: &mut Self, port_name: String) -> Result<(), ProtocolError> {
        if self.ports == None {
            return Err(ProtocolError::ToUpdateError("设备列表为空".into()));
        }

        let port_info = self
            .ports
            .as_ref()
            .unwrap()
            .iter()
            .find(|port_info| port_info.port_name == port_name);

        if port_info == None {
            return Err(ProtocolError::NotFoundPortError(port_name.into()));
        }

        let mut current_port = Port::default();
        current_port.open(port_name);
        self.current_select_port = Some(current_port);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Port {
    port_handle: Option<SerialPortBuilder>,
}

impl Port {
    fn open<'a>(self: &mut Self, port_name: impl Into<Cow<'a, str>>) {
        let builder = serialport::new(port_name, 115200);
        self.port_handle = Some(builder)
    }
}
