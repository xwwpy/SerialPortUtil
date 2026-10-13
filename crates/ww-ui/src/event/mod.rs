#[derive(Debug, Clone)]
pub struct UpdatePortsInfo {
    pub ports: Result<Vec<ww_protocol::SerialPortInfo>, String>,
}

pub struct ReceivedData {
    pub data: Vec<u8>,
}

pub struct PortError {
    pub message: String,
}

pub struct OpenStateChanged {
    pub open_state: bool,
}
