use crate::wayland_ctx::WaylandCtx;
use std::io::Write;
use std::process::{Command, Stdio};

// TODO: the batter way
pub fn save_to_wl_clipboard(wl_ctx: &mut WaylandCtx) -> Result<(), Box<dyn std::error::Error>> {
    if !wl_ctx.config.auto_copy {
        return Ok(());
    }

    // 读取文件内容
    let file_content = std::fs::read(&wl_ctx.config.output_path)
        .map_err(|e| format!("读取文件失败: {}", e))
        .unwrap();

    // 创建 wl-copy 进程，设置标准输入
    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动wl-copy失败: {}", e))
        .unwrap();

    // 写入数据到 wl-copy 的标准输入
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(&file_content)
            .map_err(|e| format!("写入数据到wl-copy失败: {}", e))
            .unwrap();
    }

    child.wait().expect("等待 wl-copy 进程结束失败");

    Ok(())
}
