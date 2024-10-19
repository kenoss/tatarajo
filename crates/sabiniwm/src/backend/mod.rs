#[cfg(feature = "udev")]
pub mod udev;
#[cfg(feature = "winit")]
pub mod winit;

pub(crate) trait DmabufHandlerDelegate: smithay::wayland::buffer::BufferHandler {
    fn dmabuf_state(&mut self) -> &mut smithay::wayland::dmabuf::DmabufState;
    fn dmabuf_imported(
        &mut self,
        global: &smithay::wayland::dmabuf::DmabufGlobal,
        dmabuf: smithay::backend::allocator::dmabuf::Dmabuf,
    ) -> bool;
}

pub(crate) trait Backend: downcast::Any + DmabufHandlerDelegate {
    fn init(&mut self, inner: &mut crate::state::InnerState);
    fn has_relative_motion(&self) -> bool;
    fn has_gesture(&self) -> bool;
    fn seat_name(&self) -> String;
    fn early_import(&mut self, surface: &wayland_server::protocol::wl_surface::WlSurface);
    fn update_led_state(&mut self, led_state: smithay::input::keyboard::LedState);
}

downcast::downcast!(dyn Backend);
