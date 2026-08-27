use encoding_rs::Encoding;
use gpui::SharedString;
use gpui_component::select::SelectItem;

#[derive(Debug, Clone)]
pub struct FontFamilyItem {
    pub font_family: String,
}

impl SelectItem for FontFamilyItem {
    type Value = String;

    fn title(&self) -> gpui::SharedString {
        self.font_family.clone().into()
    }

    fn value(&self) -> &Self::Value {
        &self.font_family
    }
}

/// encoding_rs 支持的编码方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Supported {
    Utf8,
    Gbk,
    Gb18030,
    Big5,
    ShiftJis,
    EucJp,
    Iso2022Jp,
    EucKr,
    Windows1252,
    Hex,
}

impl From<&str> for Supported {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "utf-8" | "utf8" => Self::Utf8,
            "gbk" | "gb2312" => Self::Gbk,
            "gb18030" => Self::Gb18030,
            "big5" => Self::Big5,
            "shift_jis" | "shift-jis" | "sjis" => Self::ShiftJis,
            "euc-jp" | "eucjp" => Self::EucJp,
            "iso-2022-jp" | "iso2022jp" => Self::Iso2022Jp,
            "euc-kr" | "euckr" => Self::EucKr,
            "windows-1252" | "cp1252" | "latin1" => Self::Windows1252,
            "hex" | "16进制" => Self::Hex,
            _ => Self::Utf8, // 默认 UTF-8
        }
    }
}

impl From<String> for Supported {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<SharedString> for Supported {
    fn from(value: SharedString) -> Self {
        value.as_ref().into()
    }
}

impl Supported {
    /// 编码的显示名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Gbk => "GBK",
            Self::Gb18030 => "GB18030",
            Self::Big5 => "Big5",
            Self::ShiftJis => "Shift_JIS",
            Self::EucJp => "EUC-JP",
            Self::Iso2022Jp => "ISO-2022-JP",
            Self::EucKr => "EUC-KR",
            Self::Windows1252 => "Windows-1252",
            Self::Hex => "HEX",
        }
    }

    /// 对应的 encoding_rs 编码，Hex 返回 None
    pub fn encoding(&self) -> Option<&'static Encoding> {
        match self {
            Self::Utf8 => Some(encoding_rs::UTF_8),
            Self::Gbk => Some(encoding_rs::GBK),
            Self::Gb18030 => Some(encoding_rs::GB18030),
            Self::Big5 => Some(encoding_rs::BIG5),
            Self::ShiftJis => Some(encoding_rs::SHIFT_JIS),
            Self::EucJp => Some(encoding_rs::EUC_JP),
            Self::Iso2022Jp => Some(encoding_rs::ISO_2022_JP),
            Self::EucKr => Some(encoding_rs::EUC_KR),
            Self::Windows1252 => Some(encoding_rs::WINDOWS_1252),
            Self::Hex => None,
        }
    }

    /// 所有支持的编码（用于填充下拉框）
    pub fn all() -> Vec<Self> {
        vec![
            Self::Utf8,
            Self::Gbk,
            Self::Gb18030,
            Self::Big5,
            Self::ShiftJis,
            Self::EucJp,
            Self::Iso2022Jp,
            Self::EucKr,
            Self::Windows1252,
            Self::Hex,
        ]
    }
}

/// 解码项（接收数据）
#[derive(Debug, Clone)]
pub struct DecodingItem {
    encoding: Supported,
}

impl From<Supported> for DecodingItem {
    fn from(encoding: Supported) -> Self {
        Self::new(encoding)
    }
}

impl DecodingItem {
    pub fn new(encoding: Supported) -> Self {
        Self { encoding }
    }

    pub fn encoding(&self) -> Supported {
        self.encoding
    }
}

impl SelectItem for DecodingItem {
    type Value = Supported;

    fn title(&self) -> gpui::SharedString {
        self.encoding.name().into()
    }

    fn value(&self) -> &Self::Value {
        &self.encoding
    }
}

/// 编码项（发送数据）
#[derive(Debug, Clone)]
pub struct EncodingItem {
    encoding: Supported,
}

impl From<Supported> for EncodingItem {
    fn from(encoding: Supported) -> Self {
        Self::new(encoding)
    }
}

impl EncodingItem {
    pub fn new(encoding: Supported) -> Self {
        Self { encoding }
    }

    pub fn encoding(&self) -> Supported {
        self.encoding
    }
}

impl SelectItem for EncodingItem {
    type Value = Supported;

    fn title(&self) -> gpui::SharedString {
        self.encoding.name().into()
    }

    fn value(&self) -> &Self::Value {
        &self.encoding
    }
}
