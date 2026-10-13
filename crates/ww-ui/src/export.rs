use gpui::{AnyWindowHandle, App, Window};
use gpui_component::{WindowExt, dialog::DialogButtonProps, notification::Notification};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

pub fn write_export(path: &Path, data: &[u8], overwrite: bool) -> io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(path)?;
    file.write_all(data)?;
    file.sync_all()
}

pub fn request_export(data: String, name: &str, window: &mut Window, cx: &mut App) {
    let dialog = cx.prompt_for_new_path(
        &std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir()),
        Some(name),
    );
    let handle = window.window_handle();
    cx.spawn(async move |cx| match dialog.await {
        Ok(Ok(Some(path))) => {
            let _ = handle.update(cx, move |_, window, cx| {
                if path.exists() {
                    window.open_alert_dialog(cx, move |alert, _, _| {
                        let path = path.clone();
                        let data = data.clone();
                        alert
                            .title("覆盖已有文件？")
                            .description(format!("将替换 {} 的内容", path.display()))
                            .button_props(
                                DialogButtonProps::default()
                                    .ok_text("覆盖")
                                    .cancel_text("取消")
                                    .show_cancel(true),
                            )
                            .on_ok(move |_, _, cx| {
                                start_write(path.clone(), data.clone(), true, handle, cx);
                                true
                            })
                    });
                } else {
                    start_write(path, data, false, handle, cx);
                }
            });
        }
        Ok(Ok(None)) => {}
        result => {
            let _ = handle.update(cx, |_, window, cx| {
                window.push_notification(
                    Notification::error(format!("无法打开保存对话框：{result:?}")),
                    cx,
                )
            });
        }
    })
    .detach();
}

fn start_write(
    path: PathBuf,
    data: String,
    overwrite: bool,
    window: AnyWindowHandle,
    cx: &mut App,
) {
    cx.spawn(async move |cx| {
        let shown_path = path.display().to_string();
        let result =
            tokio::task::spawn_blocking(move || write_export(&path, data.as_bytes(), overwrite))
                .await;
        let _ = window.update(cx, |_, window, cx| {
            let notification = match result {
                Ok(Ok(())) => Notification::success(format!("已导出：{shown_path}")),
                Ok(Err(e)) => Notification::error(format!("导出失败：{e}")),
                Err(e) => Notification::error(format!("导出任务失败：{e}")),
            };
            window.push_notification(notification, cx);
        });
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_requires_permission_to_overwrite() {
        let path = std::env::temp_dir().join(format!("serial-export-{}.txt", std::process::id()));
        std::fs::write(&path, "original").unwrap();
        assert_eq!(
            write_export(&path, b"new", false).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        write_export(&path, b"new", true).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        std::fs::remove_file(path).unwrap();
    }
}
