#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum Action {
    PRELOAD,
    FREEZE,
    Onselect,
    AfterSelect,
    EXIT,
}

impl Default for Action {
    fn default() -> Self {
        Action::PRELOAD
    }
}

// NOTE: 创建一个简单的宏来提取Option中的some值
#[macro_export]
macro_rules! check_options {
    ($($expr:expr),+) => {{
        (
            $(
                $expr.unwrap_or_else(|| {
                    eprintln!("Error: option is None");
                    panic!("Option is None");
                }),
            )+
        )
    }};
}
