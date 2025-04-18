use smithay_client_toolkit::delegate_simple;
use wayland_client::QueueHandle;
use wayland_protocols::wp::fractional_scale::v1::client::{
    wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    wp_fractional_scale_v1::{self, WpFractionalScaleV1},
};

use crate::foamcore::FoamShot;

impl wayland_client::Dispatch<WpFractionalScaleV1, usize> for FoamShot {
    fn event(
        app: &mut FoamShot,
        _: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        data: &usize,
        _: &wayland_client::Connection,
        _qh: &QueueHandle<FoamShot>,
    ) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
            let mut foam_output = app.wlctx.foam_outputs.as_mut().unwrap().get_mut(*data);
            foam_output
                .as_mut()
                .unwrap()
                .scale
                .as_mut()
                .unwrap()
                .update_fraction(scale);
        }
    }
}

delegate_simple!(FoamShot, WpFractionalScaleManagerV1, 1);
