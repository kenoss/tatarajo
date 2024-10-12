use crate::state::SabiniwmState;
use smithay::desktop::{LayerSurface, PopupKind, WindowSurface};
use smithay::input::Seat;
use smithay::reexports::wayland_server::backend::ObjectId;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::seat::WaylandFocus;
use smithay::xwayland::X11Surface;

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::register]
pub enum KeyboardFocusTarget {
    Window(smithay::desktop::Window),
    LayerSurface(smithay::desktop::LayerSurface),
    Popup(smithay::desktop::PopupKind),
}

impl From<smithay::desktop::Window> for KeyboardFocusTarget {
    fn from(x: smithay::desktop::Window) -> Self {
        KeyboardFocusTarget::Window(x)
    }
}

impl From<LayerSurface> for KeyboardFocusTarget {
    fn from(x: LayerSurface) -> Self {
        KeyboardFocusTarget::LayerSurface(x)
    }
}

impl From<PopupKind> for KeyboardFocusTarget {
    fn from(x: PopupKind) -> Self {
        KeyboardFocusTarget::Popup(x)
    }
}

#[thin_delegate::derive_delegate(external_trait_def = crate::external_trait_def::smithay::utils)]
impl smithay::utils::IsAlive for KeyboardFocusTarget {}

#[thin_delegate::derive_delegate(
    external_trait_def = crate::external_trait_def::smithay::input::keyboard,
    scheme = |f| {
        match self {
            Self::Window(w) => match w.underlying_surface() {
                smithay::desktop::WindowSurface::Wayland(s) => f(s.wl_surface()),
                smithay::desktop::WindowSurface::X11(s) => f(s),
            }
            Self::LayerSurface(l) => f(l.wl_surface()),
            Self::Popup(p) => f(p.wl_surface()),
        }
    }
)]
impl smithay::input::keyboard::KeyboardTarget<SabiniwmState> for KeyboardFocusTarget {}

impl smithay::wayland::seat::WaylandFocus for KeyboardFocusTarget {
    fn wl_surface(&self) -> Option<WlSurface> {
        match self {
            KeyboardFocusTarget::Window(w) => w.wl_surface(),
            KeyboardFocusTarget::LayerSurface(l) => Some(l.wl_surface().clone()),
            KeyboardFocusTarget::Popup(p) => Some(p.wl_surface().clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::register]
pub enum PointerFocusTarget {
    WlSurface(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface),
    X11Surface(smithay::xwayland::X11Surface),
}

impl From<WlSurface> for PointerFocusTarget {
    fn from(x: WlSurface) -> Self {
        PointerFocusTarget::WlSurface(x)
    }
}

impl From<&WlSurface> for PointerFocusTarget {
    fn from(x: &WlSurface) -> Self {
        PointerFocusTarget::from(x.clone())
    }
}

impl From<X11Surface> for PointerFocusTarget {
    fn from(x: X11Surface) -> Self {
        PointerFocusTarget::X11Surface(x)
    }
}

impl From<&X11Surface> for PointerFocusTarget {
    fn from(x: &X11Surface) -> Self {
        PointerFocusTarget::from(x.clone())
    }
}

impl From<PopupKind> for PointerFocusTarget {
    fn from(x: PopupKind) -> Self {
        PointerFocusTarget::from(x.wl_surface())
    }
}

impl From<PointerFocusTarget> for WlSurface {
    fn from(x: PointerFocusTarget) -> Self {
        x.wl_surface().unwrap()
    }
}

impl From<KeyboardFocusTarget> for PointerFocusTarget {
    fn from(x: KeyboardFocusTarget) -> Self {
        match x {
            KeyboardFocusTarget::Window(w) => match w.underlying_surface() {
                WindowSurface::Wayland(s) => PointerFocusTarget::from(s.wl_surface()),
                WindowSurface::X11(s) => PointerFocusTarget::from(s),
            },
            KeyboardFocusTarget::LayerSurface(l) => PointerFocusTarget::from(l.wl_surface()),
            KeyboardFocusTarget::Popup(p) => PointerFocusTarget::from(p.wl_surface()),
        }
    }
}

#[thin_delegate::derive_delegate(external_trait_def = crate::external_trait_def::smithay::utils)]
impl smithay::utils::IsAlive for PointerFocusTarget {}

#[thin_delegate::derive_delegate(external_trait_def = crate::external_trait_def::smithay::input::pointer)]
impl smithay::input::pointer::PointerTarget<SabiniwmState> for PointerFocusTarget {}

#[thin_delegate::derive_delegate(external_trait_def = crate::external_trait_def::smithay::input::touch)]
impl smithay::input::touch::TouchTarget<SabiniwmState> for PointerFocusTarget {}

impl smithay::wayland::seat::WaylandFocus for PointerFocusTarget {
    fn wl_surface(&self) -> Option<WlSurface> {
        match self {
            PointerFocusTarget::WlSurface(w) => w.wl_surface(),
            PointerFocusTarget::X11Surface(w) => w.wl_surface(),
        }
    }

    fn same_client_as(&self, object_id: &ObjectId) -> bool {
        match self {
            PointerFocusTarget::WlSurface(w) => w.same_client_as(object_id),
            PointerFocusTarget::X11Surface(w) => w.same_client_as(object_id),
        }
    }
}
