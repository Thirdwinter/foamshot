use crate::wayland_ctx::WaylandCtx;
use std::io::Write;
use std::process::{Command, Stdio};

// TODO: the batter way
pub fn save_to_wl_clipboard(wl_ctx: &mut WaylandCtx) -> Result<(), Box<dyn std::error::Error>> {
    if !wl_ctx.config.auto_copy {
        return Ok(());
    }

    let file_content =
        std::fs::read(&wl_ctx.config.output_path).map_err(|e| format!("读取文件失败: {}", e))?;

    let mut child = Command::new("wl-copy")
        .arg("--type")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动wl-copy失败: {}", e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(&file_content)
            .map_err(|e| format!("写入数据到wl-copy失败: {}", e))?;
    }

    child
        .wait()
        .map_err(|e| format!("等待 wl-copy 进程结束失败: {}", e))?;

    // 然后复制文件路径为文本类型
    let file_path = wl_ctx.config.output_path.to_string_lossy();
    let mut child = Command::new("wl-copy")
        .arg("--primary") // 使用主选区存储路径
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动wl-copy (path)失败: {}", e))?;

    // 写入路径到 wl-copy 的标准输入
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(file_path.as_bytes())
            .map_err(|e| format!("写入路径到wl-copy失败: {}", e))?;
    }

    child
        .wait()
        .map_err(|e| format!("等待 wl-copy (path)进程结束失败: {}", e))?;

    Ok(())
}
