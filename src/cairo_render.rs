//! INFO: Simple wrapping of repeated drawing processes
use cairo::{Context, ImageSurface};

pub fn draw_base(canvas: &mut [u8], w: i32, h: i32) -> cairo::Context {
    let cairo_surface = unsafe {
        ImageSurface::create_for_data_unsafe(
            canvas.as_mut_ptr(),
            cairo::Format::ARgb32,
            w,
            h,
            w * 4,
        )
        .expect("创建 Cairo ImageSurface 失败")
    };
    let cr = Context::new(&cairo_surface).expect("创建 Cairo 画布失败");
    cr.set_source_rgba(0.8, 0.8, 0.8, 0.3);
    cr.paint().unwrap();
    cr
}
