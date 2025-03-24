use crate::wayland_ctx;

pub mod freeze_mode;
pub mod result_mode;
pub mod select_mode;

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum Mode {
    Freeze(CopyHook),
    PreSelect,
    Await,
    OnDraw,
    ShowResult,
    Output(CopyHook),
    Exit,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Freeze(CopyHook::default())
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(unused)]
pub enum CopyHook {
    #[default]
    Request,
    BufferDone,
    Ready,
}

#[allow(unused)]
pub trait ModeHandle {
    fn before(&mut self, wl_ctx: wayland_ctx::WaylandCtx);
    fn exec(&mut self, wl_ctx: wayland_ctx::WaylandCtx);
    fn after(&mut self, wl_ctx: wayland_ctx::WaylandCtx);
}
