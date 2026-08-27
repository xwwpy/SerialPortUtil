pub struct Line {
    data: Vec<u8>,
}

impl Line {
    /// 创建一个空行
    pub fn new_empty() -> Self {
        Self { data: Vec::new() }
    }

    /// 往行尾追加一个字节
    pub fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }

    /// 获取该行的原始字节
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}
