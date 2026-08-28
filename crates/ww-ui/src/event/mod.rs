#[derive(Debug, Clone, PartialEq)]
pub struct UpdatePortsInfo;

pub struct ReceivedData {
    pub data: Vec<u8>,
}

pub struct PortError {
    pub message: String,
}

pub struct OpenStateChanged {
    pub open_state: bool,
}
