use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("获取设备列表失败: {0}")]
    GetPortsError(Cow<'static, str>),
    #[error("需要先获取设备列表：{0}")]
    ToUpdateError(Cow<'static, str>),
    #[error("设备{0}未找到")]
    NotFoundPortError(Cow<'static, str>),
    #[error("打开端口失败: {0}")]
    OpenPortError(Cow<'static, str>),
}
