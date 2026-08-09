use anyhow::anyhow;
use gpui::{AssetSource, Result, SharedString};
use std::{borrow::Cow, path::Path};

/// 组合资产源：先查本地文件，查不到则委托给 gpui-component-assets
pub struct AppAssets {
    base_dir: String,
    component: gpui_component_assets::Assets,
}

impl AppAssets {
    pub fn new(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
            component: gpui_component_assets::Assets::new(""),
        }
    }
}

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        // 先查本地文件
        let local = Path::new(&self.base_dir).join(path);
        tracing::info!(
            "AppAssets::load 查找: {local:?} (exists: {})",
            local.exists()
        );
        if local.exists() {
            let data = std::fs::read(&local).map_err(|e| anyhow!("读取资源失败 {local:?}: {e}"))?;
            return Ok(Some(Cow::Owned(data)));
        }
        // 查不到再委托给 gpui-component-assets
        self.component.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut result = self.component.list(path)?;
        let dir = Path::new(&self.base_dir).join(path);
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        result.push(name.into());
                    }
                }
            }
        }
        Ok(result)
    }
}
