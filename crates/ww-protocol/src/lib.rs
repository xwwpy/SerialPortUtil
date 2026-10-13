pub mod common;
pub mod model;
pub mod transfer;

pub use serialport::SerialPort;
pub use serialport::SerialPortInfo;
pub use serialport::SerialPortType;
pub use serialport::{DataBits, FlowControl, Parity, StopBits};

use serialport::available_ports;

pub fn get_ports() -> Result<Vec<SerialPortInfo>, common::error::ProtocolError> {
    available_ports()
        .map_err(|error| common::error::ProtocolError::GetPortsError(error.to_string().into()))
}

#[test]
fn print_ports() {
    let ports = available_ports().unwrap();

    println!("{:?}", ports)
}
