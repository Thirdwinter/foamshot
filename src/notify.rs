use std::process::Command;

#[allow(unused)]
pub enum NotificationLevel {
    Info,
    Warn,
    Error,
}

impl NotificationLevel {
    fn to_summary(&self) -> &str {
        match self {
            NotificationLevel::Info => "Information",
            NotificationLevel::Warn => "Warning",
            NotificationLevel::Error => "Error",
        }
    }
}

pub fn send<T: ToString>(level: NotificationLevel, body: T) {
    let summary = level.to_summary();

    let urgency = match level {
        NotificationLevel::Info => "low",
        NotificationLevel::Warn => "normal",
        NotificationLevel::Error => "critical",
    };

    Command::new("notify-send")
        .arg("--urgency")
        .arg(urgency)
        .arg(summary)
        .arg(body.to_string())
        .output()
        .ok();
}
