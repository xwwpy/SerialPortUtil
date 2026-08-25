pub mod common;
pub mod model;

pub use serialport::SerialPort;
pub use serialport::SerialPortInfo;
pub use serialport::SerialPortType;
pub use serialport::{DataBits, Parity, StopBits};

use serialport::available_ports;

pub fn get_ports() -> Vec<SerialPortInfo> {
    available_ports().unwrap()
}

#[test]
fn print_ports() {
    let ports = available_ports().unwrap();

    println!("{:?}", ports)
}
