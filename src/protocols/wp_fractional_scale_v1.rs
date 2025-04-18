use wayland_client::{QueueHandle, protocol::wl_surface::WlSurface};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::{
    self, WpFractionalScaleV1,
};

use crate::foamshot::FoamShot;

impl wayland_client::Dispatch<WpFractionalScaleV1, WlSurface> for FoamShot {
    fn event(
        app: &mut FoamShot,
        _: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        surface: &WlSurface,
        _: &wayland_client::Connection,
        _qh: &QueueHandle<FoamShot>,
    ) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
            // w.lock().unwrap().update_fraction(scale, app);
        }
    }
}
