use crate::focus::{KeyboardFocusTarget, PointerFocusTarget};
use crate::state::{Backend, ClientState, SabiniwmState};
use smithay::desktop::space::SpaceElement;
use smithay::desktop::utils::surface_primary_scanout_output;
use smithay::desktop::{PopupKind, PopupManager};
use smithay::input::keyboard::LedState;
use smithay::input::pointer::{CursorImageStatus, PointerHandle};
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::
    zxdg_toplevel_decoration_v1::Mode as DecorationMode;
use smithay::reexports::wayland_protocols::xdg::decoration::{self as xdg_decoration};
use smithay::reexports::wayland_server::protocol::wl_data_source::WlDataSource;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Resource;
use smithay::utils::Rectangle;
use smithay::wayland::compositor::{get_parent, with_states};
use smithay::wayland::fractional_scale::{with_fractional_scale, FractionalScaleHandler};
use smithay::wayland::input_method::{InputMethodHandler, PopupSurface};
use smithay::wayland::keyboard_shortcuts_inhibit::{
    KeyboardShortcutsInhibitHandler, KeyboardShortcutsInhibitState, KeyboardShortcutsInhibitor,
};
use smithay::wayland::output::OutputHandler;
use smithay::wayland::pointer_constraints::{with_pointer_constraint, PointerConstraintsHandler};
use smithay::wayland::seat::WaylandFocus;
use smithay::wayland::security_context::{
    SecurityContext, SecurityContextHandler, SecurityContextListenerSource,
};
use smithay::wayland::selection::data_device::{
    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
    ServerDndGrabHandler,
};
use smithay::wayland::selection::primary_selection::{
    set_primary_focus, PrimarySelectionHandler, PrimarySelectionState,
};
use smithay::wayland::selection::wlr_data_control::{DataControlHandler, DataControlState};
use smithay::wayland::selection::{SelectionHandler, SelectionSource, SelectionTarget};
use smithay::wayland::shell::xdg::decoration::XdgDecorationHandler;
use smithay::wayland::shell::xdg::{ToplevelSurface, XdgToplevelSurfaceData};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::xdg_activation::{
    XdgActivationHandler, XdgActivationState, XdgActivationToken, XdgActivationTokenData,
};
use smithay::wayland::xdg_foreign::{XdgForeignHandler, XdgForeignState};
use smithay::wayland::xwayland_keyboard_grab::XWaylandKeyboardGrabHandler;
use std::os::unix::io::OwnedFd;
use std::sync::Arc;

smithay::delegate_compositor!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> DataDeviceHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl<BackendData> ClientDndGrabHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn started(
        &mut self,
        _source: Option<WlDataSource>,
        icon: Option<WlSurface>,
        _seat: Seat<Self>,
    ) {
        self.dnd_icon = icon;
    }
    fn dropped(&mut self, _seat: Seat<Self>) {
        self.dnd_icon = None;
    }
}

impl<BackendData> ServerDndGrabHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {
        unreachable!("Anvil doesn't do server-side grabs");
    }
}

smithay::delegate_data_device!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> OutputHandler for SabiniwmState<BackendData> where BackendData: Backend {}

smithay::delegate_output!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> SelectionHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    type SelectionUserData = ();

    fn new_selection(
        &mut self,
        ty: SelectionTarget,
        source: Option<SelectionSource>,
        _seat: Seat<Self>,
    ) {
        if let Some(xwm) = self.xwm.as_mut() {
            if let Err(err) = xwm.new_selection(ty, source.map(|source| source.mime_types())) {
                warn!(?err, ?ty, "Failed to set Xwayland selection");
            }
        }
    }

    fn send_selection(
        &mut self,
        ty: SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
        _seat: Seat<Self>,
        _user_data: &(),
    ) {
        if let Some(xwm) = self.xwm.as_mut() {
            if let Err(err) = xwm.send_selection(ty, mime_type, fd, self.handle.clone()) {
                warn!(?err, "Failed to send primary (X11 -> Wayland)");
            }
        }
    }
}

impl<BackendData> PrimarySelectionHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.primary_selection_state
    }
}

smithay::delegate_primary_selection!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> DataControlHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn data_control_state(&self) -> &DataControlState {
        &self.data_control_state
    }
}

smithay::delegate_data_control!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> ShmHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

smithay::delegate_shm!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> SeatHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    type KeyboardFocus = KeyboardFocusTarget;
    type PointerFocus = PointerFocusTarget;
    type TouchFocus = PointerFocusTarget;

    fn seat_state(&mut self) -> &mut SeatState<SabiniwmState<BackendData>> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, target: Option<&KeyboardFocusTarget>) {
        let dh = &self.display_handle;

        let wl_surface = target.and_then(WaylandFocus::wl_surface);

        let focus = wl_surface.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, focus.clone());
        set_primary_focus(dh, seat, focus);
    }
    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        *self.cursor_status.lock().unwrap() = image;
    }

    fn led_state_changed(&mut self, _seat: &Seat<Self>, led_state: LedState) {
        self.backend_data.update_led_state(led_state)
    }
}

smithay::delegate_seat!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_tablet_manager!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_text_input_manager!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> InputMethodHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn new_popup(&mut self, surface: PopupSurface) {
        if let Err(err) = self.popups.track_popup(PopupKind::from(surface)) {
            warn!("Failed to track popup: {}", err);
        }
    }

    fn dismiss_popup(&mut self, surface: PopupSurface) {
        if let Some(parent) = surface.get_parent().map(|parent| parent.surface.clone()) {
            let _ = PopupManager::dismiss_popup(&parent, &PopupKind::from(surface));
        }
    }

    fn parent_geometry(&self, parent: &WlSurface) -> Rectangle<i32, smithay::utils::Logical> {
        self.space
            .elements()
            .find_map(|window| {
                (window.wl_surface().as_ref() == Some(parent)).then(|| window.geometry())
            })
            .unwrap_or_default()
    }
}

smithay::delegate_input_method_manager!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> KeyboardShortcutsInhibitHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn keyboard_shortcuts_inhibit_state(&mut self) -> &mut KeyboardShortcutsInhibitState {
        &mut self.keyboard_shortcuts_inhibit_state
    }

    fn new_inhibitor(&mut self, inhibitor: KeyboardShortcutsInhibitor) {
        // Just grant the wish for everyone
        inhibitor.activate();
    }
}

smithay::delegate_keyboard_shortcuts_inhibit!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_virtual_keyboard_manager!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_pointer_gestures!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_relative_pointer!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> PointerConstraintsHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn new_constraint(&mut self, surface: &WlSurface, pointer: &PointerHandle<Self>) {
        // XXX region
        if pointer
            .current_focus()
            .and_then(|x| x.wl_surface())
            .as_ref()
            == Some(surface)
        {
            with_pointer_constraint(surface, pointer, |constraint| {
                constraint.unwrap().activate();
            });
        }
    }
}

smithay::delegate_pointer_constraints!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_viewporter!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> XdgActivationHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn activation_state(&mut self) -> &mut XdgActivationState {
        &mut self.xdg_activation_state
    }

    fn token_created(&mut self, _token: XdgActivationToken, data: XdgActivationTokenData) -> bool {
        if let Some((serial, seat)) = data.serial {
            let keyboard = self.seat.get_keyboard().unwrap();
            Seat::from_resource(&seat) == Some(self.seat.clone())
                && keyboard
                    .last_enter()
                    .map(|last_enter| serial.is_no_older_than(&last_enter))
                    .unwrap_or(false)
        } else {
            false
        }
    }

    fn request_activation(
        &mut self,
        _token: XdgActivationToken,
        token_data: XdgActivationTokenData,
        surface: WlSurface,
    ) {
        if token_data.timestamp.elapsed().as_secs() < 10 {
            // Just grant the wish
            let w = self
                .space
                .elements()
                .find(|window| window.wl_surface().map(|s| s == surface).unwrap_or(false))
                .cloned();
            if let Some(window) = w {
                self.space.raise_element(&window, true);
            }
        }
    }
}

smithay::delegate_xdg_activation!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> XdgDecorationHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        // Set the default to client side
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ClientSide);
        });
    }
    fn request_mode(&mut self, toplevel: ToplevelSurface, mode: DecorationMode) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;

        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(match mode {
                DecorationMode::ServerSide => Mode::ServerSide,
                _ => Mode::ClientSide,
            });
        });

        let initial_configure_sent = with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if initial_configure_sent {
            toplevel.send_pending_configure();
        }
    }
    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ClientSide);
        });
        let initial_configure_sent = with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if initial_configure_sent {
            toplevel.send_pending_configure();
        }
    }
}

smithay::delegate_xdg_decoration!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_xdg_shell!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_layer_shell!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);
smithay::delegate_presentation!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> FractionalScaleHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn new_fractional_scale(
        &mut self,
        surface: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) {
        // Here we can set the initial fractional scale
        //
        // First we look if the surface already has a primary scan-out output, if not
        // we test if the surface is a subsurface and try to use the primary scan-out output
        // of the root surface. If the root also has no primary scan-out output we just try
        // to use the first output of the toplevel.
        // If the surface is the root we also try to use the first output of the toplevel.
        //
        // If all the above tests do not lead to a output we just use the first output
        // of the space (which in case of anvil will also be the output a toplevel will
        // initially be placed on)
        #[allow(clippy::redundant_clone)]
        let mut root = surface.clone();
        while let Some(parent) = get_parent(&root) {
            root = parent;
        }

        with_states(&surface, |states| {
            let primary_scanout_output = surface_primary_scanout_output(&surface, states)
                .or_else(|| {
                    if root != surface {
                        with_states(&root, |states| {
                            surface_primary_scanout_output(&root, states).or_else(|| {
                                self.window_for_surface(&root).and_then(|window| {
                                    self.space.outputs_for_element(&window).first().cloned()
                                })
                            })
                        })
                    } else {
                        self.window_for_surface(&root).and_then(|window| {
                            self.space.outputs_for_element(&window).first().cloned()
                        })
                    }
                })
                .or_else(|| self.space.outputs().next().cloned());
            if let Some(output) = primary_scanout_output {
                with_fractional_scale(states, |fractional_scale| {
                    fractional_scale.set_preferred_scale(output.current_scale().fractional_scale());
                });
            }
        });
    }
}

smithay::delegate_fractional_scale!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> SecurityContextHandler for SabiniwmState<BackendData>
where
    BackendData: Backend + 'static,
{
    fn context_created(
        &mut self,
        source: SecurityContextListenerSource,
        security_context: SecurityContext,
    ) {
        self.handle
            .insert_source(source, move |client_stream, _, data| {
                let client_state = ClientState {
                    security_context: Some(security_context.clone()),
                    ..ClientState::default()
                };
                if let Err(err) = data
                    .display_handle
                    .insert_client(client_stream, Arc::new(client_state))
                {
                    warn!("Error adding wayland client: {}", err);
                };
            })
            .expect("Failed to init wayland socket source");
    }
}

smithay::delegate_security_context!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> XWaylandKeyboardGrabHandler for SabiniwmState<BackendData>
where
    BackendData: Backend + 'static,
{
    fn keyboard_focus_for_xsurface(&self, surface: &WlSurface) -> Option<KeyboardFocusTarget> {
        let elem = self
            .space
            .elements()
            .find(|elem| elem.wl_surface().as_ref() == Some(surface))?;
        Some(KeyboardFocusTarget::Window(elem.0.clone()))
    }
}

smithay::delegate_xwayland_keyboard_grab!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> XdgForeignHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn xdg_foreign_state(&mut self) -> &mut XdgForeignState {
        &mut self.xdg_foreign_state
    }
}

smithay::delegate_xdg_foreign!(@<BackendData: Backend + 'static> SabiniwmState<BackendData>);

impl<BackendData> smithay::wayland::dmabuf::DmabufHandler for SabiniwmState<BackendData>
where
    BackendData: Backend,
{
    fn dmabuf_state(&mut self) -> &mut smithay::wayland::dmabuf::DmabufState {
        self.backend_data.dmabuf_state()
    }

    fn dmabuf_imported(
        &mut self,
        global: &smithay::wayland::dmabuf::DmabufGlobal,
        dmabuf: smithay::backend::allocator::dmabuf::Dmabuf,
        notifier: smithay::wayland::dmabuf::ImportNotifier,
    ) {
        if self.backend_data.dmabuf_imported(global, dmabuf) {
            let _ = notifier.successful::<SabiniwmState<BackendData>>();
        } else {
            notifier.failed();
        }
    }
}

smithay::delegate_dmabuf!(@<BackendData: Backend> SabiniwmState<BackendData>);
