use chrono::Local;
use clap::Parser;
use directories::UserDirs;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version, long_about = None)]
struct CliArgs {
    /// The directory path where the output file is located. The default is the XDG user image path
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,

    /// Output file name, supports time formatting placeholders (such as %Y, %m, %d, %H, %M, %S)
    #[arg(short = 'n', long, default_value_t = Self::default_name())]
    name: String,

    /// Whether to display the mouse when taking a screenshot. The default value is false
    #[arg(long, default_value_t = false)]
    show_cursor: bool,

    /// Whether to automatically copy the screenshot to the clipboard, requires wl-copy, default is false
    #[arg(long, default_value_t = false)]
    no_copy: bool,

    /// Whether to freeze the screen before taking a screenshot, the default is false
    #[arg(long, default_value_t = false)]
    no_freeze: bool,

    /// Whether to skip automatic full screen capture in interactive mode, the default value is false
    #[arg(long, default_value_t = false)]
    full_screen: bool,

    /// Whether to enter edit mode after taking a screenshot, the default is false
    #[arg(long, default_value_t = false)]
    edit: bool,

    /// disable desktop notify, the default is false
    #[arg(long, default_value_t = false)]
    no_notify: bool,
}

impl CliArgs {
    fn default_name() -> String {
        format!("foamshot-{}.png", Local::now().format("%Y-%m-%d-%H-%M-%S"))
    }
}

#[derive(Default, Debug, Clone)]
pub enum ImageType {
    #[default]
    Png,
    Jpg,
}

#[derive(Debug)]
#[allow(unused)]
pub struct FoamConfig {
    /// 输出路径
    pub output_path: PathBuf,
    /// 输出类型，默认为 png
    pub image_type: ImageType,
    /// 截图是否显示鼠标
    pub cursor: bool,
    /// 截图后是否自动复制到剪贴板
    pub auto_copy: bool,

    /// TODO: 截图后是否进入编辑模式
    pub edit: bool,
    /// 截图前是否冻结屏幕
    pub freeze: bool,
    /// 是否跳过交互模式自动截全屏
    pub full_screen: bool,

    pub allow_notify: bool,
}

impl Default for FoamConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl FoamConfig {
    pub fn new() -> Self {
        let args = CliArgs::parse();

        // 构造完整的输出路径
        let formatted_path =
            Self::format_path(args.path.unwrap_or(Self::generate_default_output_path()));
        let formatted_name = Self::replace_time_specifiers(&args.name);
        let (final_path, final_name) = Self::validate_path(&formatted_path, &formatted_name);
        let mut output_path = final_path;
        output_path.push(final_name);

        let image_type = Self::detect_image_type(&mut output_path);

        FoamConfig {
            output_path,
            image_type,
            cursor: args.show_cursor,
            edit: args.edit,
            auto_copy: !args.no_copy,
            freeze: !args.no_freeze,
            full_screen: args.full_screen,
            allow_notify: !args.no_notify,
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
        let path = UserDirs::new()
            .and_then(|ud| ud.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        path
    }

    fn validate_path(dir_path: &PathBuf, filename: &str) -> (PathBuf, String) {
        // 验证并创建目录
        let final_path = if !dir_path.exists() {
            match fs::create_dir_all(dir_path) {
                Ok(_) => dir_path.clone(),
                Err(_) => {
                    // 如果创建失败，使用默认路径
                    UserDirs::new()
                        .and_then(|ud| ud.picture_dir().map(|p| p.to_path_buf()))
                        .unwrap_or_else(|| PathBuf::from("."))
                }
            }
        } else {
            dir_path.clone()
        };

        // 将路径转换为绝对路径
        let absolute_path = fs::canonicalize(&final_path).unwrap_or_else(|_| final_path.clone());

        // 处理文件名
        let mut final_name = String::from(filename);
        let mut counter = 0;

        // 获取文件名和扩展名
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("foamshot");

        let ext = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");

        // 检查文件是否存在，如果存在则添加递增的数字
        while absolute_path.join(&final_name).exists() {
            counter += 1;
            final_name = format!("{}-{}.{}", stem, counter, ext);
        }

        (absolute_path, final_name)
    }

    fn detect_image_type(path: &mut PathBuf) -> ImageType {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => ImageType::Jpg,
                "png" => ImageType::Png,
                _ => {
                    // 如果无法识别后缀，将其设置为 .png
                    if let Some(parent) = path.parent() {
                        if let Some(file_stem) = path.file_stem() {
                            let mut new_path = parent.to_path_buf();
                            new_path.push(format!("{}.png", file_stem.to_string_lossy()));
                            *path = new_path;
                        } else {
                            let mut new_path = parent.to_path_buf();
                            new_path.push("screenshot.png");
                            *path = new_path;
                        }
                    }
                    ImageType::Png
                }
            }
        } else {
            // 如果没有后缀，添加 .png
            if let Some(parent) = path.parent() {
                if let Some(file_stem) = path.file_stem() {
                    let mut new_path = parent.to_path_buf();
                    new_path.push(format!("{}.png", file_stem.to_string_lossy()));
                    *path = new_path;
                } else {
                    let mut new_path = parent.to_path_buf();
                    new_path.push("screenshot.png");
                    *path = new_path;
                }
            }
            ImageType::Png
        }
    }
}
