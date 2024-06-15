use super::WindowElement;
use crate::focus::PointerFocusTarget;
use crate::state::SabiniwmState;
use smithay::input::pointer::{
    AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
    GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent,
    GestureSwipeUpdateEvent, GrabStartData as PointerGrabStartData, MotionEvent, PointerGrab,
    PointerInnerHandle, RelativeMotionEvent,
};
use smithay::input::touch::{GrabStartData as TouchGrabStartData, TouchGrab};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::utils::{Logical, Point, Serial, Size};
use smithay::xwayland::xwm::ResizeEdge as X11ResizeEdge;

pub struct PointerMoveSurfaceGrab {
    pub start_data: PointerGrabStartData<SabiniwmState>,
    pub window: WindowElement,
    pub initial_window_location: Point<i32, Logical>,
}

impl PointerGrab<SabiniwmState> for PointerMoveSurfaceGrab {
    fn motion(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        _focus: Option<(PointerFocusTarget, Point<i32, Logical>)>,
        event: &MotionEvent,
    ) {
        // While the grab is active, no client has pointer focus
        handle.motion(data, None, event);

        let delta = event.location - self.start_data.location;
        let new_location = self.initial_window_location.to_f64() + delta;

        data.space
            .map_element(self.window.clone(), new_location.to_i32_round(), true);
    }

    fn relative_motion(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        focus: Option<(PointerFocusTarget, Point<i32, Logical>)>,
        event: &RelativeMotionEvent,
    ) {
        handle.relative_motion(data, focus, event);
    }

    fn button(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &ButtonEvent,
    ) {
        handle.button(data, event);
        if handle.current_pressed().is_empty() {
            // No more buttons are pressed, release the grab.
            handle.unset_grab(data, event.serial, event.time, true);
        }
    }

    fn axis(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        details: AxisFrame,
    ) {
        handle.axis(data, details)
    }

    fn frame(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
    ) {
        handle.frame(data);
    }

    fn gesture_swipe_begin(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GestureSwipeBeginEvent,
    ) {
        handle.gesture_swipe_begin(data, event);
    }

    fn gesture_swipe_update(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GestureSwipeUpdateEvent,
    ) {
        handle.gesture_swipe_update(data, event);
    }

    fn gesture_swipe_end(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GestureSwipeEndEvent,
    ) {
        handle.gesture_swipe_end(data, event);
    }

    fn gesture_pinch_begin(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GesturePinchBeginEvent,
    ) {
        handle.gesture_pinch_begin(data, event);
    }

    fn gesture_pinch_update(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GesturePinchUpdateEvent,
    ) {
        handle.gesture_pinch_update(data, event);
    }

    fn gesture_pinch_end(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GesturePinchEndEvent,
    ) {
        handle.gesture_pinch_end(data, event);
    }

    fn gesture_hold_begin(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GestureHoldBeginEvent,
    ) {
        handle.gesture_hold_begin(data, event);
    }

    fn gesture_hold_end(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut PointerInnerHandle<'_, SabiniwmState>,
        event: &GestureHoldEndEvent,
    ) {
        handle.gesture_hold_end(data, event);
    }

    fn start_data(&self) -> &PointerGrabStartData<SabiniwmState> {
        &self.start_data
    }
}

pub struct TouchMoveSurfaceGrab {
    pub start_data: TouchGrabStartData<SabiniwmState>,
    pub window: WindowElement,
    pub initial_window_location: Point<i32, Logical>,
}

impl TouchGrab<SabiniwmState> for TouchMoveSurfaceGrab {
    fn down(
        &mut self,
        _data: &mut SabiniwmState,
        _handle: &mut smithay::input::touch::TouchInnerHandle<'_, SabiniwmState>,
        _focus: Option<(
            <SabiniwmState as smithay::input::SeatHandler>::TouchFocus,
            Point<i32, Logical>,
        )>,
        _event: &smithay::input::touch::DownEvent,
        _seq: Serial,
    ) {
    }

    fn up(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut smithay::input::touch::TouchInnerHandle<'_, SabiniwmState>,
        event: &smithay::input::touch::UpEvent,
        seq: Serial,
    ) {
        if event.slot != self.start_data.slot {
            return;
        }

        handle.up(data, event, seq);
        handle.unset_grab(data);
    }

    fn motion(
        &mut self,
        data: &mut SabiniwmState,
        _handle: &mut smithay::input::touch::TouchInnerHandle<'_, SabiniwmState>,
        _focus: Option<(
            <SabiniwmState as smithay::input::SeatHandler>::TouchFocus,
            Point<i32, Logical>,
        )>,
        event: &smithay::input::touch::MotionEvent,
        _seq: Serial,
    ) {
        if event.slot != self.start_data.slot {
            return;
        }

        let delta = event.location - self.start_data.location;
        let new_location = self.initial_window_location.to_f64() + delta;
        data.space
            .map_element(self.window.clone(), new_location.to_i32_round(), true);
    }

    fn frame(
        &mut self,
        _data: &mut SabiniwmState,
        _handle: &mut smithay::input::touch::TouchInnerHandle<'_, SabiniwmState>,
        _seq: Serial,
    ) {
    }

    fn cancel(
        &mut self,
        data: &mut SabiniwmState,
        handle: &mut smithay::input::touch::TouchInnerHandle<'_, SabiniwmState>,
        seq: Serial,
    ) {
        handle.cancel(data, seq);
        handle.unset_grab(data);
    }

    fn start_data(&self) -> &smithay::input::touch::GrabStartData<SabiniwmState> {
        &self.start_data
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ResizeEdge: u32 {
        const NONE = 0;
        const TOP = 1;
        const BOTTOM = 2;
        const LEFT = 4;
        const TOP_LEFT = 5;
        const BOTTOM_LEFT = 6;
        const RIGHT = 8;
        const TOP_RIGHT = 9;
        const BOTTOM_RIGHT = 10;
    }
}

impl From<xdg_toplevel::ResizeEdge> for ResizeEdge {
    #[inline]
    fn from(x: xdg_toplevel::ResizeEdge) -> Self {
        Self::from_bits(x as u32).unwrap()
    }
}

impl From<ResizeEdge> for xdg_toplevel::ResizeEdge {
    #[inline]
    fn from(x: ResizeEdge) -> Self {
        Self::try_from(x.bits()).unwrap()
    }
}

impl From<X11ResizeEdge> for ResizeEdge {
    fn from(edge: X11ResizeEdge) -> Self {
        match edge {
            X11ResizeEdge::Bottom => ResizeEdge::BOTTOM,
            X11ResizeEdge::BottomLeft => ResizeEdge::BOTTOM_LEFT,
            X11ResizeEdge::BottomRight => ResizeEdge::BOTTOM_RIGHT,
            X11ResizeEdge::Left => ResizeEdge::LEFT,
            X11ResizeEdge::Right => ResizeEdge::RIGHT,
            X11ResizeEdge::Top => ResizeEdge::TOP,
            X11ResizeEdge::TopLeft => ResizeEdge::TOP_LEFT,
            X11ResizeEdge::TopRight => ResizeEdge::TOP_RIGHT,
        }
    }
}

/// Information about the resize operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ResizeData {
    /// The edges the surface is being resized with.
    pub edges: ResizeEdge,
    /// The initial window location.
    pub initial_window_location: Point<i32, Logical>,
    /// The initial window size (geometry width and height).
    pub initial_window_size: Size<i32, Logical>,
}

/// State of the resize operation.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ResizeState {
    /// The surface is not being resized.
    #[default]
    NotResizing,
    /// The surface is currently being resized.
    Resizing(ResizeData),
    /// The resize has finished, and the surface needs to ack the final configure.
    WaitingForFinalAck(ResizeData, Serial),
    /// The resize has finished, and the surface needs to commit its final state.
    WaitingForCommit(ResizeData),
}
