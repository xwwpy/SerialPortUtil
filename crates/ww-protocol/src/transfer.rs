use std::io::{self, Write};

pub struct TransferResult {
    pub bytes_written: usize,
    pub error: Option<io::Error>,
}

pub fn write_bytes(
    writer: &mut impl Write,
    data: &[u8],
    cancelled: impl Fn() -> bool,
) -> TransferResult {
    let mut result = TransferResult {
        bytes_written: 0,
        error: None,
    };
    while result.bytes_written < data.len() {
        if cancelled() {
            result.error = Some(io::Error::new(io::ErrorKind::Interrupted, "发送已取消"));
            break;
        }
        let end = (result.bytes_written + 4096).min(data.len());
        match writer.write(&data[result.bytes_written..end]) {
            Ok(0) => {
                result.error = Some(io::Error::new(io::ErrorKind::WriteZero, "串口未写入数据"));
                break;
            }
            Ok(n) => result.bytes_written += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                result.error = Some(e);
                break;
            }
        }
    }
    result
}

pub fn parse_hex(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut high = None;
    for c in text.chars().filter(|c| !c.is_whitespace()) {
        let n = c
            .to_digit(16)
            .ok_or_else(|| format!("无效十六进制字符：{c}"))? as u8;
        match high.take() {
            Some(h) => out.push((h << 4) | n),
            None => high = Some(n),
        }
    }
    if high.is_some() {
        return Err("十六进制输入必须包含完整的字节（偶数位）".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PartialWriter {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl Write for PartialWriter {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            let n = data.len().min(self.limit);
            self.bytes.extend_from_slice(&data[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn partial_writes_send_every_byte_exactly_once() {
        let data: Vec<u8> = (0..140).collect();
        let mut writer = PartialWriter {
            bytes: vec![],
            limit: 3,
        };
        let result = write_bytes(&mut writer, &data, || false);
        assert_eq!(writer.bytes, data);
        assert_eq!(result.bytes_written, 140);
        assert!(result.error.is_none());
    }

    #[test]
    fn zero_write_returns_an_error_without_spinning() {
        let mut writer = PartialWriter {
            bytes: vec![],
            limit: 0,
        };
        let result = write_bytes(&mut writer, &[1], || false);
        assert_eq!(result.error.unwrap().kind(), io::ErrorKind::WriteZero);
    }

    #[test]
    fn cancellation_retains_successful_byte_count() {
        let mut writer = PartialWriter {
            bytes: vec![],
            limit: 3,
        };
        let calls = std::cell::Cell::new(0);
        let result = write_bytes(&mut writer, &[1; 10], || {
            calls.set(calls.get() + 1);
            calls.get() > 2
        });
        assert_eq!(result.bytes_written, 6);
        assert_eq!(result.error.unwrap().kind(), io::ErrorKind::Interrupted);
    }

    #[test]
    fn strict_hex_accepts_whitespace_and_rejects_invalid_or_odd_input() {
        assert_eq!(parse_hex("01 ab\nFF").unwrap(), [1, 171, 255]);
        assert!(parse_hex("12GG").is_err());
        assert!(parse_hex("ABC").is_err());
        assert!(parse_hex("0x12").is_err());
    }
}
