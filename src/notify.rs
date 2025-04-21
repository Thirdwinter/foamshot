use std::process::Command;

pub enum NotificationLevel {
    Info,
    Warn,
    Error,
}

impl NotificationLevel {
    fn to_urgency(&self) -> &str {
        match self {
            NotificationLevel::Info => "low",
            NotificationLevel::Warn => "normal",
            NotificationLevel::Error => "critical",
        }
    }
}

pub fn send<T: ToString>(
    level: NotificationLevel,
    summary: &str,
    body: T,
    image_path: T,
    allow_notify: bool,
) {
    if !allow_notify {
        return;
    }
    let urgency = level.to_urgency();

    Command::new("notify-send")
        .arg("--urgency")
        .arg(urgency)
        .arg(summary)
        .arg(body.to_string())
        .arg("-h")
        .arg(format!("string:image-path:{}", image_path.to_string()))
        .arg("-i")
        .arg(image_path.to_string())
        .arg("-a")
        .arg(env!("CARGO_PKG_NAME"))
        .output()
        .ok();
}
