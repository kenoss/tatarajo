use crate::state::SabiniwmState;
use smithay::backend::input::KeyState;
use smithay::desktop::{LayerSurface, PopupKind, WindowSurface};
use smithay::input::keyboard::{KeysymHandle, ModifiersState};
use smithay::input::pointer::{
    AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
    GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent,
    GestureSwipeUpdateEvent, MotionEvent, RelativeMotionEvent,
};
use smithay::input::touch::{DownEvent, OrientationEvent, ShapeEvent, UpEvent};
use smithay::input::{Seat, SeatHandler};
use smithay::reexports::wayland_server::backend::ObjectId;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Serial;
use smithay::wayland::seat::WaylandFocus;
use smithay::xwayland::X11Surface;

#[thin_delegate::external_trait_def]
mod __external_trait_def {
    #[thin_delegate::register]
    pub trait IsAlive {
        /// Check if object is alive
        fn alive(&self) -> bool;
    }

    #[thin_delegate::register]
    pub trait KeyboardTarget<D>: IsAlive + PartialEq + Clone + fmt::Debug + Send
    where
        D: SeatHandler,
    {
        /// Keyboard focus of a given seat was assigned to this handler
        fn enter(&self, seat: &Seat<D>, data: &mut D, keys: Vec<KeysymHandle<'_>>, serial: Serial);
        /// The keyboard focus of a given seat left this handler
        fn leave(&self, seat: &Seat<D>, data: &mut D, serial: Serial);
        /// A key was pressed on a keyboard from a given seat
        fn key(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            key: KeysymHandle<'_>,
            state: KeyState,
            serial: Serial,
            time: u32,
        );
        /// Hold modifiers were changed on a keyboard from a given seat
        fn modifiers(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            modifiers: ModifiersState,
            serial: Serial,
        );
    }

    #[thin_delegate::register]
    pub trait PointerTarget<D>: IsAlive + PartialEq + Clone + fmt::Debug + Send
    where
        D: SeatHandler,
    {
        /// A pointer of a given seat entered this handler
        fn enter(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent);
        /// A pointer of a given seat moved over this handler
        fn motion(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent);
        /// A pointer of a given seat that provides relative motion moved over this handler
        fn relative_motion(&self, seat: &Seat<D>, data: &mut D, event: &RelativeMotionEvent);
        /// A pointer of a given seat clicked a button
        fn button(&self, seat: &Seat<D>, data: &mut D, event: &ButtonEvent);
        /// A pointer of a given seat scrolled on an axis
        fn axis(&self, seat: &Seat<D>, data: &mut D, frame: AxisFrame);
        /// End of a pointer frame
        fn frame(&self, seat: &Seat<D>, data: &mut D);
        /// A pointer of a given seat started a swipe gesture
        fn gesture_swipe_begin(&self, seat: &Seat<D>, data: &mut D, event: &GestureSwipeBeginEvent);
        /// A pointer of a given seat updated a swipe gesture
        fn gesture_swipe_update(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            event: &GestureSwipeUpdateEvent,
        );
        /// A pointer of a given seat ended a swipe gesture
        fn gesture_swipe_end(&self, seat: &Seat<D>, data: &mut D, event: &GestureSwipeEndEvent);
        /// A pointer of a given seat started a pinch gesture
        fn gesture_pinch_begin(&self, seat: &Seat<D>, data: &mut D, event: &GesturePinchBeginEvent);
        /// A pointer of a given seat updated a pinch gesture
        fn gesture_pinch_update(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            event: &GesturePinchUpdateEvent,
        );
        /// A pointer of a given seat ended a pinch gesture
        fn gesture_pinch_end(&self, seat: &Seat<D>, data: &mut D, event: &GesturePinchEndEvent);
        /// A pointer of a given seat started a hold gesture
        fn gesture_hold_begin(&self, seat: &Seat<D>, data: &mut D, event: &GestureHoldBeginEvent);
        /// A pointer of a given seat ended a hold gesture
        fn gesture_hold_end(&self, seat: &Seat<D>, data: &mut D, event: &GestureHoldEndEvent);
        /// A pointer of a given seat left this handler
        fn leave(&self, seat: &Seat<D>, data: &mut D, serial: Serial, time: u32);
        /// A pointer of a given seat moved from another handler to this handler
        fn replace(
            &self,
            replaced: <D as SeatHandler>::PointerFocus,
            seat: &Seat<D>,
            data: &mut D,
            event: &MotionEvent,
        ) {
            PointerTarget::<D>::leave(&replaced, seat, data, event.serial, event.time);
            data.cursor_image(seat, CursorImageStatus::default_named());
            PointerTarget::<D>::enter(self, seat, data, event);
        }
    }

    #[thin_delegate::register]
    pub trait TouchTarget<D>: IsAlive + PartialEq + Clone + fmt::Debug + Send
    where
        D: SeatHandler,
    {
        /// A new touch point has appeared on the target.
        ///
        /// This touch point is assigned a unique ID. Future events from this touch point reference this ID.
        /// The ID ceases to be valid after a touch up event and may be reused in the future.
        fn down(&self, seat: &Seat<D>, data: &mut D, event: &DownEvent, seq: Serial);

        /// The touch point has disappeared.
        ///
        /// No further events will be sent for this touch point and the touch point's ID
        /// is released and may be reused in a future touch down event.
        fn up(&self, seat: &Seat<D>, data: &mut D, event: &UpEvent, seq: Serial);

        /// A touch point has changed coordinates.
        // fn motion(&self, seat: &Seat<D>, data: &mut D, event: &MotionEvent, seq: Serial);
        fn motion(
            &self,
            seat: &Seat<D>,
            data: &mut D,
            event: &smithay::input::touch::MotionEvent,
            seq: Serial,
        );

        /// Indicates the end of a set of events that logically belong together.
        fn frame(&self, seat: &Seat<D>, data: &mut D, seq: Serial);

        /// Touch session cancelled.
        ///
        /// Touch cancellation applies to all touch points currently active on this target.
        /// The client is responsible for finalizing the touch points, future touch points on
        /// this target may reuse the touch point ID.
        fn cancel(&self, seat: &Seat<D>, data: &mut D, seq: Serial);

        /// Sent when a touch point has changed its shape.
        ///
        /// A touch point shape is approximated by an ellipse through the major and minor axis length.
        /// The major axis length describes the longer diameter of the ellipse, while the minor axis
        /// length describes the shorter diameter. Major and minor are orthogonal and both are specified
        /// in surface-local coordinates. The center of the ellipse is always at the touch point location
        /// as reported by [`TouchTarget::down`] or [`TouchTarget::motion`].
        fn shape(&self, seat: &Seat<D>, data: &mut D, event: &ShapeEvent, seq: Serial);

        /// Sent when a touch point has changed its orientation.
        ///
        /// The orientation describes the clockwise angle of a touch point's major axis to the positive surface
        /// y-axis and is normalized to the -180 to +180 degree range. The granularity of orientation depends
        /// on the touch device, some devices only support binary rotation values between 0 and 90 degrees.
        fn orientation(&self, seat: &Seat<D>, data: &mut D, event: &OrientationEvent, seq: Serial);
    }
}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::register]
pub enum KeyboardFocusTarget {
    Window(smithay::desktop::Window),
    LayerSurface(smithay::desktop::LayerSurface),
    Popup(smithay::desktop::PopupKind),
}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl smithay::utils::IsAlive for KeyboardFocusTarget {}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def, scheme = |f| {
    match self {
        Self::Window(w) => match w.underlying_surface() {
            smithay::desktop::WindowSurface::Wayland(s) => f(s.wl_surface()),
            smithay::desktop::WindowSurface::X11(s) => f(s),
        }
        Self::LayerSurface(s) => f(s.wl_surface()),
        Self::Popup(p) => f(p.wl_surface()),
    }
})]
impl smithay::input::keyboard::KeyboardTarget<SabiniwmState> for KeyboardFocusTarget {}

#[derive(Debug, Clone, PartialEq)]
#[thin_delegate::register]
pub enum PointerFocusTarget {
    WlSurface(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface),
    X11Surface(smithay::xwayland::X11Surface),
}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl smithay::utils::IsAlive for PointerFocusTarget {}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl smithay::input::pointer::PointerTarget<SabiniwmState> for PointerFocusTarget {}

#[thin_delegate::derive_delegate(external_trait_def = __external_trait_def)]
impl smithay::input::touch::TouchTarget<SabiniwmState> for PointerFocusTarget {}

impl From<PointerFocusTarget> for WlSurface {
    fn from(target: PointerFocusTarget) -> Self {
        target.wl_surface().unwrap()
    }
}

impl WaylandFocus for PointerFocusTarget {
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

impl WaylandFocus for KeyboardFocusTarget {
    fn wl_surface(&self) -> Option<WlSurface> {
        match self {
            KeyboardFocusTarget::Window(w) => w.wl_surface(),
            KeyboardFocusTarget::LayerSurface(l) => Some(l.wl_surface().clone()),
            KeyboardFocusTarget::Popup(p) => Some(p.wl_surface().clone()),
        }
    }
}

impl From<WlSurface> for PointerFocusTarget {
    fn from(value: WlSurface) -> Self {
        PointerFocusTarget::WlSurface(value)
    }
}

impl From<&WlSurface> for PointerFocusTarget {
    fn from(value: &WlSurface) -> Self {
        PointerFocusTarget::from(value.clone())
    }
}

impl From<PopupKind> for PointerFocusTarget {
    fn from(value: PopupKind) -> Self {
        PointerFocusTarget::from(value.wl_surface())
    }
}

impl From<X11Surface> for PointerFocusTarget {
    fn from(value: X11Surface) -> Self {
        PointerFocusTarget::X11Surface(value)
    }
}

impl From<&X11Surface> for PointerFocusTarget {
    fn from(value: &X11Surface) -> Self {
        PointerFocusTarget::from(value.clone())
    }
}

impl From<smithay::desktop::Window> for KeyboardFocusTarget {
    fn from(w: smithay::desktop::Window) -> Self {
        KeyboardFocusTarget::Window(w)
    }
}

impl From<LayerSurface> for KeyboardFocusTarget {
    fn from(l: LayerSurface) -> Self {
        KeyboardFocusTarget::LayerSurface(l)
    }
}

impl From<PopupKind> for KeyboardFocusTarget {
    fn from(p: PopupKind) -> Self {
        KeyboardFocusTarget::Popup(p)
    }
}

impl From<KeyboardFocusTarget> for PointerFocusTarget {
    fn from(value: KeyboardFocusTarget) -> Self {
        match value {
            KeyboardFocusTarget::Window(w) => match w.underlying_surface() {
                WindowSurface::Wayland(w) => PointerFocusTarget::from(w.wl_surface()),
                WindowSurface::X11(s) => PointerFocusTarget::from(s),
            },
            KeyboardFocusTarget::LayerSurface(surface) => {
                PointerFocusTarget::from(surface.wl_surface())
            }
            KeyboardFocusTarget::Popup(popup) => PointerFocusTarget::from(popup.wl_surface()),
        }
    }
}
