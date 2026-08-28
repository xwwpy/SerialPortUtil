use std::sync::OnceLock;

use config::{Config, File};
use gpui::{Size, size};
use serde::{Deserialize, Serialize};

use crate::common::error::UIError;
use crate::model::port_model::BaudRateItem;

static UI_CONFIG: OnceLock<UIConfig> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct PortPanelConfig {
    port_update_interval: Option<u64>,
    default_baud_rate: Option<u32>,
    baud_rate_default_vec: Option<Vec<BaudRateItem>>,
    // 串口读取超时时间，单位毫秒
    read_timeout: Option<u64>,
}

impl PortPanelConfig {
    pub fn get_port_update_interval(&self) -> u64 {
        self.port_update_interval.unwrap_or_else(|| {
            tracing::info!("没有配置端口更新间隔，使用默认值：1");
            1
        })
    }

    pub fn get_default_baud_rate(&self) -> u32 {
        self.default_baud_rate.unwrap_or_else(|| {
            tracing::info!("没有配置默认波特率，使用默认值：115200");
            115200
        })
    }

    pub fn get_read_timeout_timeout(&self) -> u64 {
        self.read_timeout.unwrap_or_else(|| {
            tracing::info!("没有配置打开串口超时时间，使用默认值：1000");
            1000
        })
    }

    pub fn get_baud_rate_default_vec(&self) -> Vec<BaudRateItem> {
        self.baud_rate_default_vec.as_ref().unwrap().clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct LogConfig {
    log_dir: Option<String>,
    log_level: Option<String>,
}

impl LogConfig {
    pub fn get_log_dir(&self) -> String {
        self.log_dir.clone().unwrap_or_else(|| {
            tracing::info!("没有配置Log的目标文件夹，使用默认值：Log");
            "Log".to_string()
        })
    }

    pub fn get_log_level(&self) -> String {
        self.log_level.clone().unwrap_or_else(|| {
            tracing::info!("没有配置输出日志的等级，使用默认值: info");
            "info".to_string()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct CommonConfig {
    font_family: Option<String>,
    font_size: Option<u32>,
    default_border_color: Option<u32>,
    focus_border_color: Option<u32>,
    encoding: Option<String>,
    decoding: Option<String>,
    max_lines: Option<usize>,
}

impl CommonConfig {
    pub fn get_font_family(&self) -> String {
        self.font_family.clone().unwrap_or_else(|| {
            tracing::info!("没有配置字体，使用默认值：JetBrains Mono");
            "JetBrains Mono".to_string()
        })
    }

    pub fn get_font_size(&self) -> u32 {
        self.font_size.unwrap_or_else(|| {
            tracing::info!("没有配置字体大小，使用默认值：14");
            14
        })
    }

    pub fn get_default_border_color(&self) -> u32 {
        self.default_border_color.unwrap_or_else(|| {
            tracing::info!("没有配置默认边框颜色，使用默认值：0xFFFFFF");
            0xFFFFFF
        })
    }

    pub fn get_focus_border_color(&self) -> u32 {
        self.focus_border_color.unwrap_or_else(|| {
            tracing::info!("没有配置聚焦边框颜色，使用默认值：0x8A2BE2");
            0x8A2BE2
        })
    }

    pub fn get_encoding(&self) -> String {
        self.encoding.clone().unwrap_or_else(|| {
            tracing::info!("没有配置默认encoding，使用默认值：Utf8");
            "Utf8".into()
        })
    }

    pub fn get_decoding(&self) -> String {
        self.decoding.clone().unwrap_or_else(|| {
            tracing::info!("没有配置默认decoding，使用默认值：Utf8");
            "Utf8".into()
        })
    }

    pub fn get_max_lines(&self) -> usize {
        self.max_lines.clone().unwrap_or_else(|| {
            tracing::info!("没有配置默认max_lines，使用默认值：50000");
            50000
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct IoPanelConfig {
    pub default_output: Option<String>,
    pub input_placeholder: Option<String>,
}

impl IoPanelConfig {
    pub fn get_default_output(&self) -> String {
        self.default_output.clone().unwrap_or_else(|| {
            tracing::info!("没有配置默认串口发送数据为空的占位值，使用默认值：");
            "".to_string()
        })
    }

    pub fn get_input_placeholder(&self) -> String {
        self.input_placeholder.clone().unwrap_or_else(|| {
            tracing::info!("没有配置用户输入占位符，使用默认值：");
            "".to_string()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct AuthorInfoConfig {
    author_name: Option<String>,
    author_email: Option<String>,
}

impl AuthorInfoConfig {
    pub fn get_author_name(&self) -> String {
        self.author_name.clone().unwrap_or_else(|| {
            tracing::info!("没有配置作者名称，使用默认值：ww");
            "ww".to_string()
        })
    }

    pub fn get_author_email(&self) -> String {
        self.author_email.clone().unwrap_or_else(|| {
            tracing::info!("没有配置作者邮箱，使用默认值：example@email");
            "example@email".to_string()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct UIConfig {
    window_size: Option<Size<f32>>,
    log_config: LogConfig,
    author_info: AuthorInfoConfig,
    port_panel_config: PortPanelConfig,
    io_panel_config: IoPanelConfig,
    common_config: CommonConfig,
}

impl UIConfig {
    pub fn get_log_config(&self) -> &LogConfig {
        &self.log_config
    }
    pub fn get_window_size(&self) -> Size<f32> {
        self.window_size.clone().unwrap_or_else(|| {
            tracing::info!("没有配置窗口大小，使用默认值1200,700");
            size(1200., 700.)
        })
    }
    pub fn get_author_info(&self) -> &AuthorInfoConfig {
        &self.author_info
    }

    pub fn get_port_panel_config(&self) -> &PortPanelConfig {
        &self.port_panel_config
    }

    pub fn get_io_panel_config(&self) -> &IoPanelConfig {
        &self.io_panel_config
    }

    pub fn get_common_config(&self) -> &CommonConfig {
        &self.common_config
    }
}

fn config_init() -> Result<UIConfig, UIError> {
    Ok(Config::builder()
        .add_source(
            File::with_name("config")
                .format(config::FileFormat::Yaml)
                .required(true),
        )
        .build()?
        .try_deserialize::<UIConfig>()?)
}

pub fn get() -> &'static UIConfig {
    &UI_CONFIG.get_or_init(|| {
        let res = config_init();
        match res {
            Err(e) => {
                eprintln!("解析配置文件出错：{e}，使用默认配置过渡");
                UIConfig::default()
            }
            Ok(config) => {
                eprintln!("解析配置文件完成!");
                config
            }
        }
    })
}
