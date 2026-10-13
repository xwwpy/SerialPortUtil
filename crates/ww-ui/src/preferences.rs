use std::path::{Path, PathBuf};

use gpui::{App, BorrowAppContext, Global};
use serde::{Deserialize, Serialize};

use crate::model::config_panel::{AutoAppendItem, Supported};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Standard,
    Analysis,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Layout {
    pub port: bool,
    pub encoding: bool,
    pub font: bool,
    pub receive: bool,
    pub transmit: bool,
    pub editor: bool,
    pub waveform: bool,
    pub status: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            port: true,
            encoding: true,
            font: true,
            receive: true,
            transmit: true,
            editor: true,
            waveform: false,
            status: true,
        }
    }
}

impl Layout {
    pub fn preset(preset: Preset) -> Self {
        match preset {
            Preset::Standard => Self::default(),
            Preset::Analysis => Self {
                font: false,
                transmit: false,
                waveform: true,
                ..Self::default()
            },
            Preset::Compact => Self {
                font: false,
                encoding: false,
                transmit: false,
                ..Self::default()
            },
        }
    }
    pub fn has_config(&self) -> bool {
        self.port || self.encoding || self.font
    }
    pub fn has_work(&self) -> bool {
        self.receive || self.transmit || self.editor || self.waveform
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub version: u32,
    pub layout: Layout,
    pub port_name: String,
    pub baud_rate: u32,
    pub parity: String,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub flow_control: String,
    pub encoding: Supported,
    pub decoding: Supported,
    pub append: AutoAppendItem,
    pub font_family: String,
    pub font_size: u32,
    pub interval_ms: u64,
    pub auto_scroll: bool,
    pub timestamp: bool,
    pub simple_time: bool,
    pub chart_protocol: String,
    pub channels: [bool; 16],
}

impl Default for Preferences {
    fn default() -> Self {
        let config = crate::ui_config::get();
        Self {
            version: 1,
            layout: Layout::default(),
            port_name: String::new(),
            baud_rate: config.get_port_panel_config().get_default_baud_rate(),
            parity: "none".into(),
            data_bits: 8,
            stop_bits: 1,
            flow_control: "none".into(),
            encoding: config.get_common_config().get_encoding().into(),
            decoding: config.get_common_config().get_decoding().into(),
            append: config
                .get_common_config()
                .get_default_auto_tx_append_item()
                .into(),
            font_family: config.get_common_config().get_font_family(),
            font_size: config.get_common_config().get_font_size(),
            interval_ms: 1000,
            auto_scroll: true,
            timestamp: true,
            simple_time: true,
            chart_protocol: "csv".into(),
            channels: [true; 16],
        }
    }
}

impl Preferences {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("不支持的设置版本".into());
        }
        if !(10..=3_600_000).contains(&self.interval_ms) {
            return Err("定时发送间隔应为 10–3600000 ms".into());
        }
        if !(1..=4_000_000).contains(&self.baud_rate) {
            return Err("波特率超出范围".into());
        }
        if !(5..=8).contains(&self.data_bits) || !(1..=2).contains(&self.stop_bits) {
            return Err("无效数据位或停止位".into());
        }
        if !["none", "odd", "even"].contains(&self.parity.as_str()) {
            return Err("无效校验位".into());
        }
        if !["none", "hardware", "software"].contains(&self.flow_control.as_str()) {
            return Err("无效流控".into());
        }
        if !(10..=28).contains(&self.font_size) || self.font_family.trim().is_empty() {
            return Err("无效字体设置".into());
        }
        if !["csv", "justfloat"].contains(&self.chart_protocol.as_str()) {
            return Err("无效解析协议".into());
        }
        Ok(())
    }
    pub fn from_json(data: &str) -> Result<Self, String> {
        let settings: Self = serde_json::from_str(data).map_err(|e| e.to_string())?;
        settings.validate()?;
        Ok(settings)
    }
}

pub struct PreferenceStore {
    pub settings: Preferences,
    pub message: Option<String>,
    path: PathBuf,
}
impl Global for PreferenceStore {}

impl PreferenceStore {
    pub fn load(path: PathBuf) -> Self {
        let (settings, message) = match std::fs::read_to_string(&path) {
            Ok(data) => match Preferences::from_json(&data) {
                Ok(settings) => (settings, None),
                Err(error) => (
                    Preferences::default(),
                    Some(format!("用户设置无效，已使用默认设置：{error}")),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (Preferences::default(), None)
            }
            Err(error) => (
                Preferences::default(),
                Some(format!("无法读取用户设置：{error}")),
            ),
        };
        Self {
            settings,
            message,
            path,
        }
    }
    pub fn default_path() -> PathBuf {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("SerialPortUtil")
            .join("preferences.json")
    }
    pub fn get(cx: &App) -> &Preferences {
        &cx.global::<Self>().settings
    }
    pub fn update(cx: &mut App, update: impl FnOnce(&mut Preferences)) {
        cx.update_global::<Self, _>(|store, _| {
            update(&mut store.settings);
            store.message = store
                .settings
                .validate()
                .and_then(|_| save(&store.path, &store.settings))
                .err()
                .map(|e| format!("设置未保存：{e}"));
        });
    }
}

fn save(path: &Path, settings: &Preferences) -> Result<(), String> {
    let directory = path.parent().ok_or("设置路径缺少父目录")?;
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&temporary, data).map_err(|e| e.to_string())?;
    std::fs::rename(temporary, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_prioritize_their_workflow() {
        let compact = Layout::preset(Preset::Compact);
        assert!(compact.receive && compact.editor);
        assert!(!compact.font && !compact.transmit && !compact.encoding);
        let analysis = Layout::preset(Preset::Analysis);
        assert!(analysis.waveform);
        assert!(!analysis.font && !analysis.transmit);
    }
    #[test]
    fn unsupported_versions_and_bad_intervals_are_rejected() {
        assert!(Preferences::from_json(r#"{"version":2}"#).is_err());
        assert!(Preferences::from_json(r#"{"interval_ms":0}"#).is_err());
        assert!(Preferences::from_json(r#"{"data_bits":9}"#).is_err());
        assert!(Preferences::from_json(r#"{"parity":"unknown"}"#).is_err());
    }
    #[test]
    fn round_trip_preserves_hidden_panels_without_enabling_other_panels() {
        let mut settings = Preferences::default();
        settings.layout = Layout {
            port: false,
            encoding: false,
            font: false,
            receive: false,
            transmit: false,
            editor: false,
            waveform: false,
            status: false,
        };
        let restored = Preferences::from_json(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert!(!restored.layout.has_config());
        assert!(!restored.layout.has_work());
        assert_eq!(restored.interval_ms, 1000);
    }
    #[test]
    fn atomic_save_can_replace_a_previous_setting() {
        let path =
            std::env::temp_dir().join(format!("serial-preferences-{}.json", std::process::id()));
        let mut settings = Preferences::default();
        save(&path, &settings).unwrap();
        settings.layout.font = false;
        save(&path, &settings).unwrap();
        let restored = PreferenceStore::load(path.clone());
        assert!(!restored.settings.layout.font);
        std::fs::remove_file(path).unwrap();
    }
}
