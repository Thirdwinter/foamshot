use chrono::Local;
use clap::Parser;
use directories::UserDirs;
use log::info;
use std::path::PathBuf; // 引入 chrono 库用于时间处理

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// show cursor when screen freeze, default to false
    #[arg(long, default_value_t = false)]
    show_cursor: bool,

    /// output path, default to xdg user picture dir, supports format specifiers like %Y, %m, %d, %H, %M, %S
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// disable quickshot, default to true
    #[arg(long = "no-quickshot")]
    no_quickshot: bool,

    #[arg(long, default_value_t = false)]
    no_copy: bool,
}

pub struct Cli {
    pub no_cursor: bool,
    pub output_path: PathBuf,
    pub quickshot: bool,
    pub auto_copy: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    pub fn new() -> Self {
        let args = CliArgs::parse();

        // 动态生成默认输出路径（如果未提供）
        let output_path = args
            .output_path
            .map(Self::format_path)
            .unwrap_or_else(Self::generate_default_output_path);

        Cli {
            no_cursor: !args.show_cursor,
            output_path,
            quickshot: !args.no_quickshot,
            auto_copy: !args.no_copy,
        }
    }

    fn format_path(path: PathBuf) -> PathBuf {
        let path_str = path.to_string_lossy().to_string();
        let formatted_path = Self::replace_time_specifiers(&path_str);
        PathBuf::from(formatted_path)
    }

    fn replace_time_specifiers(path_str: &str) -> String {
        let now = Local::now();
        let mut formatted = path_str.to_string();

        formatted = formatted.replace("%Y", &now.format("%Y").to_string());
        formatted = formatted.replace("%m", &now.format("%m").to_string());
        formatted = formatted.replace("%d", &now.format("%d").to_string());
        formatted = formatted.replace("%H", &now.format("%H").to_string());
        formatted = formatted.replace("%M", &now.format("%M").to_string());
        formatted = formatted.replace("%S", &now.format("%S").to_string());

        formatted
    }

    fn generate_default_output_path() -> PathBuf {
        let mut path = UserDirs::new()
            .and_then(|ud| ud.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        // 生成格式化的时间字符串
        let now = Local::now();
        let time_str = now.format("%Y-%m-%d-%H-%M-%S").to_string();

        path.push(format!("foam_shot-{}.png", time_str));
        info!("output path: {}", path.display());

        path
    }
}

pub struct Config {
    pub cli: Cli,
    pub full_screen_output: bool,
}
