use directories::UserDirs;
use std::path::{Path, PathBuf};

pub struct Cli {
    pub no_cursor: bool,
    pub output_path: PathBuf,
}

impl Cli {
    pub fn new() -> Self {
        let mut output_path = if let Some(user_dirs) = UserDirs::new() {
            // 获取用户的图片目录
            user_dirs
                .picture_dir()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        } else {
            // 如果无法获取用户目录，使用当前目录作为回退
            PathBuf::from(".")
        };

        // 获取当前时间戳（以秒为单位）
        let start = std::time::SystemTime::now();
        let since_the_epoch = start
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_the_epoch.as_secs();

        // 构造完整的输出路径
        output_path.push(format!("foam_shot_{}.png", timestamp));
        println!("Output path: {}", output_path.display());

        Self {
            no_cursor: false,
            output_path,
        }
    }
}
