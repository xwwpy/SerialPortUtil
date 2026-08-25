use crate::common::error::ProtocolError;

use serialport::{DataBits, Parity, SerialPort, SerialPortBuilder, SerialPortInfo, StopBits};

#[derive(Debug, Default)]
pub struct Ports {
    current_select_port: Option<SerialPortBuilder>,
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

    pub fn get_current_select_port(&self) -> Option<&SerialPortBuilder> {
        self.current_select_port.as_ref()
    }

    pub fn update_ports_info(self: &mut Self, ports: Vec<SerialPortInfo>) {
        self.ports = Some(ports);
    }

    pub fn select_port(
        self: &mut Self,
        port_name: String,
        baud_rate: u32,
        parity: impl Into<Parity>,
        databits: impl Into<DataBits>,
        stopbits: impl Into<StopBits>,
        open_port_timeout: u64,
    ) -> Result<(), ProtocolError> {
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

        let current_port = serialport::new(port_name, baud_rate)
            .parity(parity.into())
            .data_bits(databits.into())
            .stop_bits(stopbits.into())
            .timeout(std::time::Duration::from_millis(open_port_timeout));

        self.current_select_port = Some(current_port);
        Ok(())
    }

    pub fn open_port(self: &mut Self) -> Result<Box<dyn SerialPort>, ProtocolError> {
        if let Some(port) = self.current_select_port.clone() {
            let res = port.open();
            if let Err(e) = res {
                return Err(ProtocolError::OpenPortError(e.to_string().into()));
            }
            return Ok(res.unwrap());
        }
        Err(ProtocolError::OpenPortError("未知错误".into()))
    }
}
