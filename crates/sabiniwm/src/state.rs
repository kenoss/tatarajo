use crate::action::Action;
use crate::backend::udev::UdevBackend;
use crate::backend::winit::WinitBackend;
use crate::backend::BackendI;
use crate::cursor::Cursor;
use crate::input::{KeySeq, Keymap};
use crate::input_event::FocusUpdateDecider;
use crate::view::stackset::WorkspaceTag;
use crate::view::view::View;
use crate::view::window::Window;
use smithay::backend::renderer::element::utils::select_dmabuf_feedback;
use smithay::backend::renderer::element::{
    default_primary_scanout_output_compare, RenderElementStates,
};
use smithay::desktop::utils::{
    surface_presentation_feedback_flags_from_states, surface_primary_scanout_output,
    update_surface_primary_scanout_output, OutputPresentationFeedback,
};
use smithay::desktop::{PopupManager, Space};
use smithay::input::pointer::{CursorImageStatus, PointerHandle};
use smithay::input::{Seat, SeatState};
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction};
use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use smithay::reexports::wayland_server::{Display, DisplayHandle};
use smithay::utils::{Clock, Monotonic, Point, Rectangle, Size};
use smithay::wayland::compositor::{CompositorClientState, CompositorState};
use smithay::wayland::dmabuf::DmabufFeedback;
use smithay::wayland::fractional_scale::with_fractional_scale;
use smithay::wayland::input_method::InputMethodManagerState;
use smithay::wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitState;
use smithay::wayland::pointer_constraints::PointerConstraintsState;
use smithay::wayland::pointer_gestures::PointerGesturesState;
use smithay::wayland::relative_pointer::RelativePointerManagerState;
use smithay::wayland::security_context::{SecurityContext, SecurityContextState};
use smithay::wayland::selection::data_device::DataDeviceState;
use smithay::wayland::selection::primary_selection::PrimarySelectionState;
use smithay::wayland::selection::wlr_data_control::DataControlState;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shm::ShmState;
use smithay::wayland::socket::ListeningSocketSource;
use smithay::wayland::tablet_manager::{TabletManagerState, TabletSeatTrait};
use smithay::wayland::text_input::TextInputManagerState;
use smithay::wayland::virtual_keyboard::VirtualKeyboardManagerState;
use smithay::wayland::xdg_activation::XdgActivationState;
use smithay::wayland::xdg_foreign::XdgForeignState;
use smithay::wayland::xwayland_keyboard_grab::XWaylandKeyboardGrabState;
use smithay::xwayland::{X11Wm, XWayland, XWaylandEvent};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    pub security_context: Option<SecurityContext>,
}

impl ClientData for ClientState {
    /// Notification that a client was initialized
    fn initialized(&self, _client_id: ClientId) {}
    /// Notification that a client is disconnected
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub struct SabiniwmState {
    pub(crate) backend: Box<dyn BackendI>,
    pub(crate) inner: InnerState,
}

pub(crate) struct InnerState {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, SabiniwmState>,
    pub loop_signal: LoopSignal,

    // desktop
    pub space: Space<Window>,
    pub popups: PopupManager,

    // smithay state
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub layer_shell_state: WlrLayerShellState,
    pub primary_selection_state: PrimarySelectionState,
    pub data_control_state: DataControlState,
    pub seat_state: SeatState<SabiniwmState>,
    pub keyboard_shortcuts_inhibit_state: KeyboardShortcutsInhibitState,
    pub shm_state: ShmState,
    pub xdg_activation_state: XdgActivationState,
    pub xdg_shell_state: XdgShellState,
    pub xdg_foreign_state: XdgForeignState,

    pub dnd_icon: Option<wayland_server::protocol::wl_surface::WlSurface>,

    // input-related fields
    pub cursor_status: Arc<Mutex<CursorImageStatus>>,
    pub seat_name: String,
    pub seat: Seat<SabiniwmState>,
    pub clock: Clock<Monotonic>,
    pub pointer: PointerHandle<SabiniwmState>,

    pub xwayland: XWayland,
    pub xwm: Option<X11Wm>,
    pub xdisplay: Option<u32>,

    pub keymap: Keymap<Action>,
    pub keyseq: KeySeq,
    pub view: View,
    pub focus_update_decider: FocusUpdateDecider,
}

pub(crate) struct SabiniwmStateWithConcreteBackend<'a, Backend>
where
    Backend: BackendI,
{
    pub backend: &'a mut Backend,
    pub inner: &'a mut InnerState,
}

impl SabiniwmState {
    pub fn run(workspace_tags: Vec<WorkspaceTag>, keymap: Keymap<Action>) -> anyhow::Result<()> {
        let event_loop = EventLoop::try_new().unwrap();

        let use_udev = matches!(
            std::env::var("DISPLAY"),
            Err(std::env::VarError::NotPresent)
        ) && matches!(
            std::env::var("WAYLAND_DISPLAY"),
            Err(std::env::VarError::NotPresent)
        );

        let backend: Box<dyn BackendI> = if use_udev {
            Box::new(UdevBackend::new(event_loop.handle().clone())?)
        } else {
            Box::new(WinitBackend::new(event_loop.handle().clone())?)
        };

        let mut state = Self::new(
            workspace_tags,
            keymap,
            event_loop.handle(),
            event_loop.get_signal(),
            backend,
        );

        state.backend.init(&mut state.inner);

        // TODO: Unify them if possible.
        if use_udev {
            UdevBackend::run(state, event_loop);
        } else {
            WinitBackend::run(state, event_loop);
        }

        Ok(())
    }

    fn new(
        workspace_tags: Vec<WorkspaceTag>,
        keymap: Keymap<Action>,
        loop_handle: LoopHandle<'static, SabiniwmState>,
        loop_signal: LoopSignal,
        backend: Box<dyn BackendI>,
    ) -> SabiniwmState {
        let display = Display::new().unwrap();
        let dh = display.handle();

        let clock = Clock::new();

        // init wayland clients
        let source = ListeningSocketSource::new_auto().unwrap();
        let socket_name = source.socket_name().to_string_lossy().into_owned();
        loop_handle
            .insert_source(source, |client_stream, _, state| {
                if let Err(err) = state
                    .inner
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                {
                    warn!("Error adding wayland client: {}", err);
                };
            })
            .expect("Failed to init wayland socket source");
        std::env::set_var("WAYLAND_DISPLAY", &socket_name);
        info!(?socket_name, "Listening on wayland socket");

        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, state| {
                    // Safety: we don't drop the display
                    unsafe {
                        display.get_mut().dispatch_clients(state).unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("Failed to init wayland server source");

        // init globals
        let compositor_state = CompositorState::new::<Self>(&dh);
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        let layer_shell_state = WlrLayerShellState::new::<Self>(&dh);
        let primary_selection_state = PrimarySelectionState::new::<Self>(&dh);
        let data_control_state =
            DataControlState::new::<Self, _>(&dh, Some(&primary_selection_state), |_| true);
        let mut seat_state = SeatState::new();
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let xdg_activation_state = XdgActivationState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let xdg_foreign_state = XdgForeignState::new::<Self>(&dh);
        TextInputManagerState::new::<Self>(&dh);
        InputMethodManagerState::new::<Self, _>(&dh, |_client| true);
        VirtualKeyboardManagerState::new::<Self, _>(&dh, |_client| true);
        if backend.has_relative_motion() {
            RelativePointerManagerState::new::<Self>(&dh);
        }
        PointerConstraintsState::new::<Self>(&dh);
        if backend.has_gesture() {
            PointerGesturesState::new::<Self>(&dh);
        }
        TabletManagerState::new::<Self>(&dh);
        SecurityContextState::new::<Self, _>(&dh, |client| {
            client
                .get_data::<ClientState>()
                .map_or(true, |client_state| client_state.security_context.is_none())
        });

        // init input
        let seat_name = backend.seat_name();
        let mut seat = seat_state.new_wl_seat(&dh, seat_name.clone());

        let cursor_status = Arc::new(Mutex::new(CursorImageStatus::default_named()));
        let pointer = seat.add_pointer();

        let xkb_config = smithay::input::keyboard::XkbConfig {
            layout: "custom",
            ..Default::default()
        };
        seat.add_keyboard(xkb_config, 200, 60).unwrap();

        let cursor_status2 = cursor_status.clone();
        seat.tablet_seat()
            .on_cursor_surface(move |_tool, new_status| {
                // TODO: tablet tools should have their own cursors
                *cursor_status2.lock().unwrap() = new_status;
            });

        let keyboard_shortcuts_inhibit_state = KeyboardShortcutsInhibitState::new::<Self>(&dh);

        let xwayland = {
            XWaylandKeyboardGrabState::new::<Self>(&dh);

            let (xwayland, channel) = XWayland::new(&dh);
            let dh = dh.clone();
            let ret = loop_handle.insert_source(channel, move |event, _, state| match event {
                XWaylandEvent::Ready {
                    connection,
                    client,
                    client_fd: _,
                    display,
                } => {
                    let mut wm = X11Wm::start_wm(
                        state.inner.loop_handle.clone(),
                        dh.clone(),
                        connection,
                        client,
                    )
                    .expect("Failed to attach X11 Window Manager");
                    let cursor = Cursor::load();
                    let image = cursor.get_image(1, Duration::ZERO);
                    wm.set_cursor(
                        &image.pixels_rgba,
                        Size::from((image.width as u16, image.height as u16)),
                        Point::from((image.xhot as u16, image.yhot as u16)),
                    )
                    .expect("Failed to set xwayland default cursor");
                    std::env::set_var("DISPLAY", format!(":{}", display));
                    state.inner.xwm = Some(wm);
                    state.inner.xdisplay = Some(display);
                }
                XWaylandEvent::Exited => {
                    let _ = state.inner.xwm.take();
                }
            });
            if let Err(e) = ret {
                error!(
                    "Failed to insert the XWaylandSource into the event loop: {}",
                    e
                );
            }
            xwayland
        };

        let rect = Rectangle::from_loc_and_size((0, 0), (1280, 720));
        let view = View::new(rect, workspace_tags);

        SabiniwmState {
            backend,
            inner: InnerState {
                display_handle: dh,
                loop_handle,
                loop_signal,
                space: Space::default(),
                popups: PopupManager::default(),
                compositor_state,
                data_device_state,
                layer_shell_state,
                primary_selection_state,
                data_control_state,
                seat_state,
                keyboard_shortcuts_inhibit_state,
                shm_state,
                xdg_activation_state,
                xdg_shell_state,
                xdg_foreign_state,
                dnd_icon: None,
                cursor_status,
                seat_name,
                seat,
                pointer,
                clock,
                xwayland,
                xwm: None,
                xdisplay: None,

                keymap,
                keyseq: KeySeq::new(),
                view,
                focus_update_decider: FocusUpdateDecider::new(),
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct SurfaceDmabufFeedback<'a> {
    pub render_feedback: &'a DmabufFeedback,
    pub scanout_feedback: &'a DmabufFeedback,
}

pub(crate) fn post_repaint(
    output: &smithay::output::Output,
    render_element_states: &RenderElementStates,
    space: &Space<crate::view::window::Window>,
    dmabuf_feedback: Option<SurfaceDmabufFeedback<'_>>,
    time: Duration,
) {
    let throttle = Some(Duration::from_secs(1));

    space.elements().for_each(|window| {
        window.smithay_window().with_surfaces(|surface, states| {
            let primary_scanout_output = update_surface_primary_scanout_output(
                surface,
                output,
                states,
                render_element_states,
                default_primary_scanout_output_compare,
            );

            if let Some(output) = primary_scanout_output {
                with_fractional_scale(states, |fraction_scale| {
                    fraction_scale.set_preferred_scale(output.current_scale().fractional_scale());
                });
            }
        });

        if space.outputs_for_element(window).contains(output) {
            window.smithay_window().send_frame(
                output,
                time,
                throttle,
                surface_primary_scanout_output,
            );
            if let Some(dmabuf_feedback) = dmabuf_feedback {
                window.smithay_window().send_dmabuf_feedback(
                    output,
                    surface_primary_scanout_output,
                    |surface, _| {
                        select_dmabuf_feedback(
                            surface,
                            render_element_states,
                            dmabuf_feedback.render_feedback,
                            dmabuf_feedback.scanout_feedback,
                        )
                    },
                );
            }
        }
    });
    let map = smithay::desktop::layer_map_for_output(output);
    for layer_surface in map.layers() {
        layer_surface.with_surfaces(|surface, states| {
            let primary_scanout_output = update_surface_primary_scanout_output(
                surface,
                output,
                states,
                render_element_states,
                default_primary_scanout_output_compare,
            );

            if let Some(output) = primary_scanout_output {
                with_fractional_scale(states, |fraction_scale| {
                    fraction_scale.set_preferred_scale(output.current_scale().fractional_scale());
                });
            }
        });

        layer_surface.send_frame(output, time, throttle, surface_primary_scanout_output);
        if let Some(dmabuf_feedback) = dmabuf_feedback {
            layer_surface.send_dmabuf_feedback(
                output,
                surface_primary_scanout_output,
                |surface, _| {
                    select_dmabuf_feedback(
                        surface,
                        render_element_states,
                        dmabuf_feedback.render_feedback,
                        dmabuf_feedback.scanout_feedback,
                    )
                },
            );
        }
    }
}

pub(crate) fn take_presentation_feedback(
    output: &smithay::output::Output,
    space: &Space<crate::view::window::Window>,
    render_element_states: &RenderElementStates,
) -> OutputPresentationFeedback {
    let mut output_presentation_feedback = OutputPresentationFeedback::new(output);

    space.elements().for_each(|window| {
        if space.outputs_for_element(window).contains(output) {
            window.smithay_window().take_presentation_feedback(
                &mut output_presentation_feedback,
                surface_primary_scanout_output,
                |surface, _| {
                    surface_presentation_feedback_flags_from_states(surface, render_element_states)
                },
            );
        }
    });
    let map = smithay::desktop::layer_map_for_output(output);
    for layer_surface in map.layers() {
        layer_surface.take_presentation_feedback(
            &mut output_presentation_feedback,
            surface_primary_scanout_output,
            |surface, _| {
                surface_presentation_feedback_flags_from_states(surface, render_element_states)
            },
        );
    }

    output_presentation_feedback
}
