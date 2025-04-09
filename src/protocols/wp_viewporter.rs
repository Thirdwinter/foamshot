use wayland_client::{Dispatch, Proxy};
use wayland_protocols::wp::viewporter::client::{wp_viewport, wp_viewporter};

use crate::foamshot::FoamShot;

// NOTE: ne events
#[allow(unused_variables)]
impl Dispatch<wp_viewporter::WpViewporter, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wp_viewporter::WpViewporter,
        event: <wp_viewporter::WpViewporter as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}

// NOTE: ne events
#[allow(unused_variables)]
impl Dispatch<wp_viewport::WpViewport, ()> for FoamShot {
    fn event(
        app: &mut Self,
        proxy: &wp_viewport::WpViewport,
        event: <wp_viewport::WpViewport as Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        // todo!()
    }
}
