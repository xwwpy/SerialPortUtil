use config::ConfigError;

#[derive(thiserror::Error, Debug)]
pub enum UIError {
    #[error("读取配置项出错: {0}")]
    ConfigError(#[from] ConfigError),
}
