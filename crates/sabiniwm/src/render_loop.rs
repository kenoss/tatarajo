use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::calloop::{LoopHandle, RegistrationToken};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::time::{Duration, Instant};

pub(crate) struct RenderLoop<State> {
    loop_handle: LoopHandle<'static, State>,
    #[allow(clippy::type_complexity)]
    callback: Rc<Box<dyn Fn(&mut State)>>,
    inner: Rc<RefCell<Inner>>,
}

struct Inner {
    mode: LoopMode,
    /// Unit: 0.001Hz. E.g. about 60000 for 60Hz.
    refresh_rate: u32,
}

/// Represents loop's current mode.
///
/// Note that we need to manage a timer (corresponding to `registration_token`). A timer can be
/// dorroped:
///
/// - At the end of the callback, we need to choose to reregister or drop it. We can't keep it
///   without an explicit deadline/duration. We might use very huge duration to "pause" the loop
///   (especially, for WaitingVBlank), but we manage timers explicitly as the cost of
///   `LoopHandle::insert_source()` is very low. (See calloop's internal.)
/// - When the user of `RenderLoop` explicitly called `RenderLoop::stop()`.
#[derive(Debug, PartialEq, Eq)]
enum LoopMode {
    /// Stopped the loop.
    ///
    /// Needs explicit restart.
    Stopped,
    /// Starting/continuing the loop and waiting for the next timer event.
    WaitingForTimer {
        registration_token: RegistrationToken,
    },
    /// Callback is started.
    CallbackStarted {
        registration_token: RegistrationToken,
    },
    /// Rendered but not scanned-out.
    ///
    /// It will become `WaitingForTimer` with the given deadline at the end of loop.
    ContinueWithRefreshRate {
        registration_token: RegistrationToken,
        deadline: Instant,
    },
    /// Waiting for a VBlank, which will restart the loop.
    WaitingForVBlank,
}

impl LoopMode {
    fn transit(&mut self, next: LoopMode) {
        assert!(
            self.is_admissible_transition(&next),
            "LoopMode::transit({:?}, {:?}) is forbidden",
            self,
            next
        );

        *self = next;
    }

    fn is_admissible_transition(&self, next: &LoopMode) -> bool {
        use LoopMode::*;

        #[allow(clippy::match_like_matches_macro)]
        match (self, next) {
            (Stopped, WaitingForTimer { .. }) => true,
            (WaitingForTimer { .. }, Stopped | CallbackStarted { .. }) => true,
            (
                CallbackStarted { .. },
                Stopped
                | CallbackStarted { .. }
                | ContinueWithRefreshRate { .. }
                | WaitingForVBlank,
            ) => true,
            (ContinueWithRefreshRate { .. }, Stopped | WaitingForTimer { .. }) => true,
            (WaitingForVBlank, Stopped | WaitingForTimer { .. }) => true,
            _ => false,
        }
    }
}

impl<State> RenderLoop<State>
where
    State: 'static,
{
    pub fn new<F>(
        loop_handle: LoopHandle<'static, State>,
        output: &smithay::output::Output,
        callback: F,
    ) -> Self
    where
        F: Fn(&mut State) + 'static,
    {
        let refresh_rate: u32 = output
            .current_mode()
            .map(|mode| mode.refresh)
            .unwrap_or(60_000)
            .try_into()
            // Refresh rate is positive.
            .unwrap();
        // The unit is 0.001Hz. Check the value is in 0.5Hz -- 500Hz.
        assert!(500 < refresh_rate && refresh_rate < 500_000);

        let inner = Inner {
            mode: LoopMode::Stopped,
            refresh_rate,
        };
        Self {
            loop_handle,
            callback: Rc::new(Box::new(callback)),
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    #[cfg(test)]
    pub fn new_for_test<F>(
        loop_handle: LoopHandle<'static, State>,
        refresh_rate: u32,
        callback: F,
    ) -> Self
    where
        F: Fn(&mut State) + 'static,
    {
        let inner = Inner {
            mode: LoopMode::Stopped,
            refresh_rate,
        };
        Self {
            loop_handle,
            callback: Rc::new(Box::new(callback)),
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    #[inline]
    #[allow(clippy::type_complexity)]
    fn loop_(
        inner: &Rc<RefCell<Inner>>,
        callback: &Rc<Box<dyn Fn(&mut State)>>,
        state: &mut State,
    ) -> TimeoutAction {
        {
            let mut inner = inner.borrow_mut();
            let LoopMode::WaitingForTimer { registration_token } = inner.mode else {
                unreachable!();
            };
            inner
                .mode
                .transit(LoopMode::CallbackStarted { registration_token });
        }

        callback(state);

        {
            let mut inner = inner.borrow_mut();
            match inner.mode {
                LoopMode::Stopped => TimeoutAction::Drop,
                LoopMode::WaitingForTimer { .. } => unreachable!(),
                LoopMode::CallbackStarted { .. } => {
                    inner.mode.transit(LoopMode::WaitingForVBlank);
                    TimeoutAction::Drop
                }
                LoopMode::ContinueWithRefreshRate {
                    registration_token,
                    deadline,
                } => {
                    inner
                        .mode
                        .transit(LoopMode::WaitingForTimer { registration_token });
                    TimeoutAction::ToInstant(deadline)
                }
                LoopMode::WaitingForVBlank => unreachable!(),
            }
        }
    }

    pub fn start(&mut self) {
        let timer = Timer::from_deadline(Instant::now());
        self.start_aux(timer);
    }

    fn start_aux(&mut self, timer: Timer) {
        match self.inner.borrow().mode {
            LoopMode::Stopped | LoopMode::WaitingForVBlank => {}
            LoopMode::WaitingForTimer { .. }
            | LoopMode::CallbackStarted { .. }
            | LoopMode::ContinueWithRefreshRate { .. } => return,
        }

        let inner = self.inner.clone();
        let callback = self.callback.clone();
        let registration_token = self
            .loop_handle
            .insert_source(timer, move |_, _, state| {
                Self::loop_(&inner, &callback, state)
            })
            .unwrap();

        let mut inner = self.inner.borrow_mut();
        inner
            .mode
            .transit(LoopMode::WaitingForTimer { registration_token });
    }

    #[allow(unused)]
    pub fn stop(&mut self) {
        let mut inner = self.inner.borrow_mut();
        match inner.mode {
            LoopMode::Stopped => {}
            LoopMode::WaitingForTimer { registration_token } => {
                self.loop_handle.remove(registration_token);
                inner.mode.transit(LoopMode::Stopped);
            }
            LoopMode::CallbackStarted { .. } | LoopMode::ContinueWithRefreshRate { .. } => {
                // The timer is dropped at the next loop end.
                inner.mode.transit(LoopMode::Stopped);
            }
            LoopMode::WaitingForVBlank => {
                inner.mode.transit(LoopMode::Stopped);
            }
        }
    }

    pub fn on_render_frame(&mut self, should_schedule_render: bool) {
        let mut inner = self.inner.borrow_mut();

        let registration_token = match inner.mode {
            LoopMode::Stopped | LoopMode::WaitingForTimer { .. } | LoopMode::WaitingForVBlank => {
                unreachable!("on_render_frame() should be called in RenderLoop::loop_()");
            }
            LoopMode::CallbackStarted { registration_token } => registration_token,
            LoopMode::ContinueWithRefreshRate { .. } => {
                unreachable!(
                    "on_render_frame() should be called exactly once in RenderLoop::loop_()"
                );
            }
        };

        if should_schedule_render {
            // If not submitted, continue the loop.
            //
            // TODO: Pause the loop if no need to render.

            let deadline = Self::next_deadline(&inner).unwrap();
            inner.mode.transit(LoopMode::ContinueWithRefreshRate {
                registration_token,
                deadline,
            });
        }
    }

    pub fn on_vblank(&mut self) {
        let timer = {
            let inner = self.inner.borrow_mut();

            match inner.mode {
                // It is possible that `.stop()` is called before it, e.g. by
                // `smithay::backend::session::Event::SessionEvent::PauseSession`.
                LoopMode::Stopped => return,
                // IIUC, this can be occur in the initial rendering.
                //
                // TODO: Investigate it.
                LoopMode::WaitingForTimer { .. } => return,
                LoopMode::WaitingForVBlank => {}
                LoopMode::CallbackStarted { .. } | LoopMode::ContinueWithRefreshRate { .. } => {
                    unreachable!("on_vblank() should follow on_render_frame(false)");
                }
            };

            let deadline = Self::next_deadline(&inner).unwrap();
            Timer::from_deadline(deadline)
        };
        self.start_aux(timer);
    }

    fn next_deadline(inner: &RefMut<'_, Inner>) -> Option<Instant> {
        // TODO:
        //
        // - Subtract a duration for sabiniwm's render so that we can submit a next frame before
        //   VSync. See also
        //   https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L1305
        // - Use `last_render_ended_at` for base point.
        let duration =
            Duration::from_micros((1_000_000f32 * 1000.0 / inner.refresh_rate as f32) as u64);
        Instant::now().checked_add(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::calloop::{EventLoop, LoopSignal};

    #[test]
    fn smoke_test() {
        struct TestState {
            render_loop: RenderLoop<TestState>,
            loop_handle: LoopHandle<'static, TestState>,
            loop_signal: LoopSignal,
            n: usize,
        }

        let mut event_loop = EventLoop::try_new().unwrap();

        let mut render_loop =
            RenderLoop::new_for_test(event_loop.handle(), 60_000, |state: &mut TestState| {
                if state.n % 3 == 0 {
                    state.render_loop.on_render_frame(false);

                    let timer = Timer::from_duration(Duration::from_millis(16));
                    state
                        .loop_handle
                        .insert_source(timer, |_, _, state| {
                            state.render_loop.on_vblank();
                            TimeoutAction::Drop
                        })
                        .unwrap();
                } else {
                    state.render_loop.on_render_frame(true);
                }

                state.n -= 1;
                if state.n == 0 {
                    state.loop_signal.stop();
                }
            });
        render_loop.start();

        let mut state = TestState {
            render_loop,
            loop_handle: event_loop.handle(),
            loop_signal: event_loop.get_signal(),
            n: 10,
        };

        event_loop.run(None, &mut state, |_| {}).unwrap();
    }
}
