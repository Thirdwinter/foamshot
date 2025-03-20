/// NOTE: 冻结屏幕的过程不可回退，一旦从PRELOAD转换到FREEZE(在接收到Event::Ready后)就不可回退
/// 这意味着FreezeMode中的screencopy_frame已经确定，对应的layer已经生成，位于下层（它会最先生成）
/// 之后生成默认的SelectMode，提供基本的layer用于选择（白色半透明），直到鼠标按下，进入Onselect状态
#[derive(Debug, Clone, Copy, Default)]
pub enum Action {
    #[default]
    PreLoad,
    Freeze,
    Onselect,
    AfterSelect,
    Exit,
}

///  NOTE: 创建一个简单的宏来提取Option中的some值
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
