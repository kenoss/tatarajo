use crate::action::Action;
use crate::input::Keymap;
use crate::pointer::PointerElement;
use crate::render::{render_output, CustomRenderElement};
use crate::state::{post_repaint, take_presentation_feedback, Backend, CalloopData, SabiniwmState};
use crate::view::stackset::WorkspaceTag;
use smithay::backend::egl::EGLDevice;
use smithay::backend::renderer::damage::{Error as OutputDamageTrackerError, OutputDamageTracker};
use smithay::backend::renderer::element::AsRenderElements;
use smithay::backend::renderer::gles::GlesRenderer;
#[cfg(feature = "egl")]
use smithay::backend::renderer::ImportEgl;
use smithay::backend::renderer::{ImportDma, ImportMemWl};
use smithay::backend::winit::{self, WinitEvent, WinitGraphicsBackend};
use smithay::backend::SwapBuffersError;
use smithay::input::pointer::{CursorImageAttributes, CursorImageStatus};
use smithay::output::{Mode, PhysicalProperties, Subpixel};
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_protocols::wp::presentation_time::server::wp_presentation_feedback;
use smithay::reexports::wayland_server::protocol::wl_surface;
use smithay::reexports::wayland_server::Display;
use smithay::reexports::winit::platform::pump_events::PumpStatus;
use smithay::utils::{IsAlive, Scale, Transform};
use smithay::wayland::compositor;
use smithay::wayland::dmabuf::{DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufState};
use std::ffi::OsString;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::Duration;

pub const OUTPUT_NAME: &str = "winit";

pub struct WinitData {
    backend: WinitGraphicsBackend<GlesRenderer>,
    damage_tracker: OutputDamageTracker,
    dmabuf_state: (DmabufState, DmabufGlobal, Option<DmabufFeedback>),
    full_redraw: u8,
}

macro_rules! backend_data_winit_mut {
    ($state:ident) => {
        $state
            .backend_data
            .as_mut()
            .downcast_mut::<WinitData>()
            .unwrap()
    };
}

impl smithay::wayland::buffer::BufferHandler for WinitData {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {}
}

impl crate::state::DmabufHandlerDelegate for WinitData {
    fn dmabuf_state(&mut self) -> &mut smithay::wayland::dmabuf::DmabufState {
        &mut self.dmabuf_state.0
    }

    fn dmabuf_imported(
        &mut self,
        _global: &smithay::wayland::dmabuf::DmabufGlobal,
        dmabuf: smithay::backend::allocator::dmabuf::Dmabuf,
    ) -> bool {
        self.backend.renderer().import_dmabuf(&dmabuf, None).is_ok()
    }
}

impl Backend for WinitData {
    fn has_relative_motion(&self) -> bool {
        false
    }

    fn has_gesture(&self) -> bool {
        false
    }

    fn seat_name(&self) -> String {
        String::from("winit")
    }
    fn early_import(&mut self, _surface: &wl_surface::WlSurface) {}
    fn update_led_state(&mut self, _led_state: smithay::input::keyboard::LedState) {}
}

pub fn run_winit(workspace_tags: Vec<WorkspaceTag>, keymap: Keymap<Action>) {
    let mut event_loop = EventLoop::try_new().unwrap();
    let display = Display::new().unwrap();
    let mut display_handle = display.handle();

    #[cfg_attr(not(feature = "egl"), allow(unused_mut))]
    let (mut backend, mut winit) = match winit::init::<GlesRenderer>() {
        Ok(ret) => ret,
        Err(err) => {
            error!("Failed to initialize Winit backend: {}", err);
            return;
        }
    };
    let size = backend.window_size();

    let mode = Mode {
        size,
        refresh: 60_000,
    };
    let output = smithay::output::Output::new(
        OUTPUT_NAME.to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Smithay".into(),
            model: "Winit".into(),
        },
    );
    let _global = output.create_global::<SabiniwmState>(&display.handle());
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    let render_node = EGLDevice::device_for_display(backend.renderer().egl_context().display())
        .and_then(|device| device.try_get_render_node());

    let dmabuf_default_feedback = match render_node {
        Ok(Some(node)) => {
            let dmabuf_formats = backend.renderer().dmabuf_formats().collect::<Vec<_>>();
            let dmabuf_default_feedback = DmabufFeedbackBuilder::new(node.dev_id(), dmabuf_formats)
                .build()
                .unwrap();
            Some(dmabuf_default_feedback)
        }
        Ok(None) => {
            warn!("failed to query render node, dmabuf will use v3");
            None
        }
        Err(err) => {
            warn!(?err, "failed to egl device for display, dmabuf will use v3");
            None
        }
    };

    // if we failed to build dmabuf feedback we fall back to dmabuf v3
    // Note: egl on Mesa requires either v4 or wl_drm (initialized with bind_wl_display)
    let dmabuf_state = if let Some(default_feedback) = dmabuf_default_feedback {
        let mut dmabuf_state = DmabufState::new();
        let dmabuf_global = dmabuf_state.create_global_with_default_feedback::<SabiniwmState>(
            &display.handle(),
            &default_feedback,
        );
        (dmabuf_state, dmabuf_global, Some(default_feedback))
    } else {
        let dmabuf_formats = backend.renderer().dmabuf_formats().collect::<Vec<_>>();
        let mut dmabuf_state = DmabufState::new();
        let dmabuf_global =
            dmabuf_state.create_global::<SabiniwmState>(&display.handle(), dmabuf_formats);
        (dmabuf_state, dmabuf_global, None)
    };

    #[cfg(feature = "egl")]
    if backend
        .renderer()
        .bind_wl_display(&display.handle())
        .is_ok()
    {
        info!("EGL hardware-acceleration enabled");
    };

    let data = {
        let damage_tracker = OutputDamageTracker::from_output(&output);

        Box::new(WinitData {
            backend,
            damage_tracker,
            dmabuf_state,
            full_redraw: 0,
        })
    };
    let mut state = SabiniwmState::init(
        workspace_tags,
        keymap,
        display,
        event_loop.handle(),
        data,
        true,
    );
    state.shm_state.update_formats(
        backend_data_winit_mut!(state)
            .backend
            .renderer()
            .shm_formats(),
    );
    state.space.map_output(&output, (0, 0));

    if let Err(e) = state.xwayland.start(
        state.handle.clone(),
        None,
        std::iter::empty::<(OsString, OsString)>(),
        true,
        |_| {},
    ) {
        error!("Failed to start XWayland: {}", e);
    }

    info!("Initialization completed, starting the main loop.");

    let mut pointer_element = PointerElement::default();

    while state.running.load(Ordering::SeqCst) {
        let status = winit.dispatch_new_events(|event| match event {
            WinitEvent::Input(event) => {
                state.process_input_event(event);
            }
            WinitEvent::Resized { size, .. } => {
                // We only have one output
                let output = state.space.outputs().next().unwrap().clone();
                state.space.map_output(&output, (0, 0));
                let mode = Mode {
                    size,
                    refresh: 60_000,
                };
                output.change_current_state(Some(mode), None, None, None);
                output.set_preferred(mode);
                state
                    .view
                    .resize_output(size.to_logical(1), &mut state.space);
            }
            WinitEvent::CloseRequested | WinitEvent::Focus(_) | WinitEvent::Redraw => {}
        });

        if let PumpStatus::Exit(_) = status {
            state.running.store(false, Ordering::SeqCst);
            break;
        }

        // drawing logic
        {
            let backend_data = backend_data_winit_mut!(state);
            let backend = &mut backend_data.backend;

            let mut cursor_guard = state.cursor_status.lock().unwrap();

            // draw the cursor as relevant
            // reset the cursor if the surface is no longer alive
            let mut reset = false;
            if let CursorImageStatus::Surface(ref surface) = *cursor_guard {
                reset = !surface.alive();
            }
            if reset {
                *cursor_guard = CursorImageStatus::default_named();
            }
            let cursor_visible = !matches!(*cursor_guard, CursorImageStatus::Surface(_));

            pointer_element.set_status(cursor_guard.clone());

            let full_redraw = &mut backend_data.full_redraw;
            *full_redraw = full_redraw.saturating_sub(1);
            let space = &mut state.space;
            let damage_tracker = &mut backend_data.damage_tracker;

            let dnd_icon = state.dnd_icon.as_ref();

            let scale = Scale::from(output.current_scale().fractional_scale());
            let cursor_hotspot = if let CursorImageStatus::Surface(ref surface) = *cursor_guard {
                compositor::with_states(surface, |states| {
                    states
                        .data_map
                        .get::<Mutex<CursorImageAttributes>>()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .hotspot
                })
            } else {
                (0, 0).into()
            };
            let cursor_pos = state.pointer.current_location() - cursor_hotspot.to_f64();
            let cursor_pos_scaled = cursor_pos.to_physical(scale).to_i32_round();

            let render_res = backend.bind().and_then(|_| {
                let age = if *full_redraw > 0 {
                    0
                } else {
                    backend.buffer_age().unwrap_or(0)
                };

                let renderer = backend.renderer();

                let mut elements = Vec::<CustomRenderElement<GlesRenderer>>::new();

                elements.extend(pointer_element.render_elements(
                    renderer,
                    cursor_pos_scaled,
                    scale,
                    1.0,
                ));

                // draw the dnd icon if any
                if let Some(surface) = dnd_icon {
                    if surface.alive() {
                        elements.extend(AsRenderElements::<GlesRenderer>::render_elements(
                            &smithay::desktop::space::SurfaceTree::from_surface(surface),
                            renderer,
                            cursor_pos_scaled,
                            scale,
                            1.0,
                        ));
                    }
                }

                render_output(renderer, &output, space, elements, damage_tracker, age).map_err(
                    |err| match err {
                        OutputDamageTrackerError::Rendering(err) => err.into(),
                        _ => unreachable!(),
                    },
                )
            });

            match render_res {
                Ok(render_output_result) => {
                    let has_rendered = render_output_result.damage.is_some();
                    if let Some(damage) = render_output_result.damage {
                        if let Err(err) = backend.submit(Some(&*damage)) {
                            warn!("Failed to submit buffer: {}", err);
                        }
                    }

                    backend.window().set_cursor_visible(cursor_visible);

                    // Send frame events so that client start drawing their next frame
                    let time = state.clock.now();
                    post_repaint(
                        &output,
                        &render_output_result.states,
                        &state.space,
                        None,
                        time,
                    );

                    if has_rendered {
                        let mut output_presentation_feedback = take_presentation_feedback(
                            &output,
                            &state.space,
                            &render_output_result.states,
                        );
                        output_presentation_feedback.presented(
                            time,
                            output
                                .current_mode()
                                .map(|mode| Duration::from_secs_f64(1_000f64 / mode.refresh as f64))
                                .unwrap_or_default(),
                            0,
                            wp_presentation_feedback::Kind::Vsync,
                        )
                    }
                }
                Err(SwapBuffersError::ContextLost(err)) => {
                    error!("Critical Rendering Error: {}", err);
                    state.running.store(false, Ordering::SeqCst);
                }
                Err(err) => warn!("Rendering error: {}", err),
            }
        }

        let mut calloop_data = CalloopData {
            state,
            display_handle,
        };
        let result = event_loop.dispatch(Some(Duration::from_millis(1)), &mut calloop_data);
        CalloopData {
            state,
            display_handle,
        } = calloop_data;

        if result.is_err() {
            state.running.store(false, Ordering::SeqCst);
        } else {
            state.space.refresh();
            state.popups.cleanup();
            display_handle.flush_clients().unwrap();
        }
    }
}