use crate::state::SabiniwmState;
use crate::view::layout_node::{LayoutMessage, LayoutMessageI};
use dyn_clone::DynClone;

pub trait ActionFnI: std::fmt::Debug + DynClone {
    fn into_action(self) -> Action
    where
        Self: Sized + 'static,
    {
        Action::ActionFn(self.into())
    }
    fn exec(&self, state: &mut SabiniwmState);
}

dyn_clone::clone_trait_object!(ActionFnI);

#[derive(Debug, Clone)]
pub struct ActionFn {
    inner: Box<dyn ActionFnI>,
}

impl<T> From<T> for ActionFn
where
    T: ActionFnI + 'static,
{
    fn from(x: T) -> Self {
        Self { inner: Box::new(x) }
    }
}

impl ActionFn {
    fn exec(&self, state: &mut SabiniwmState) {
        self.inner.exec(state);
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Spawn(String),
    LayoutMessage(LayoutMessage),
    ActionFn(ActionFn),
}

impl From<LayoutMessage> for Action {
    fn from(x: LayoutMessage) -> Self {
        Self::LayoutMessage(x)
    }
}

impl<T> From<T> for Action
where
    T: LayoutMessageI,
{
    fn from(x: T) -> Self {
        Self::LayoutMessage(x.into())
    }
}

impl Action {
    pub fn spawn(s: impl ToString) -> Self {
        Action::Spawn(s.to_string())
    }
}

impl SabiniwmState {
    pub(crate) fn process_action(&mut self, action: &Action) {
        info!("{:?}", action);
        match action {
            Action::Spawn(s) => {
                let _ = std::process::Command::new("/bin/sh")
                    .arg("-c")
                    .arg(s)
                    .spawn();
            }
            Action::LayoutMessage(message) => {
                self.inner
                    .view
                    .handle_layout_message(message, &mut self.inner.space);
                self.reflect_focus_from_stackset(None);
            }
            Action::ActionFn(f) => {
                f.exec(self);
                self.inner.view.layout(&mut self.inner.space);
                self.reflect_focus_from_stackset(None);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionQuitSabiniwm;

impl ActionFnI for ActionQuitSabiniwm {
    fn exec(&self, state: &mut SabiniwmState) {
        state.inner.loop_signal.stop();
    }
}

#[derive(Debug, Clone)]
pub struct ActionChangeVt(pub i32);

impl ActionFnI for ActionChangeVt {
    fn exec(&self, state: &mut SabiniwmState) {
        state.backend.change_vt(self.0);
    }
}

#[derive(Debug, Clone)]
pub enum ActionMoveFocus {
    Next,
    Prev,
}

impl ActionFnI for ActionMoveFocus {
    fn exec(&self, state: &mut SabiniwmState) {
        let count = match self {
            Self::Next => 1,
            Self::Prev => -1,
        };
        state.inner.view.update_stackset_with(|stackset| {
            let stack = &mut stackset.workspaces.focus_mut().stack;
            let i = stack.mod_plus_focused_index(count);
            stack.set_focused_index(i);
        });
    }
}

#[derive(Debug, Clone)]
pub enum ActionWindowSwap {
    Next,
    Prev,
}

impl ActionFnI for ActionWindowSwap {
    fn exec(&self, state: &mut SabiniwmState) {
        let count = match self {
            Self::Next => 1,
            Self::Prev => -1,
        };
        state.inner.view.update_stackset_with(|stackset| {
            let stack = &mut stackset.workspaces.focus_mut().stack;

            if stack.is_empty() {
                return;
            }

            let mut stack = stack.as_mut();
            let i = stack.focus;
            let j = stack.mod_plus_focused_index(count);
            stack.vec.swap(i, j);
            stack.focus = j;
            stack.commit();
        });
    }
}

#[derive(Debug, Clone)]
pub enum ActionWorkspaceFocus {
    Next,
    Prev,
}

impl ActionFnI for ActionWorkspaceFocus {
    fn exec(&self, state: &mut SabiniwmState) {
        let count = match self {
            Self::Next => 1,
            Self::Prev => -1,
        };
        state.inner.view.update_stackset_with(|stackset| {
            let workspaces = &mut stackset.workspaces;
            let i = workspaces.mod_plus_focused_index(count);
            workspaces.set_focused_index(i);
        });
    }
}

#[derive(Debug, Clone)]
pub enum ActionWorkspaceFocusNonEmpty {
    Next,
    Prev,
}

impl ActionFnI for ActionWorkspaceFocusNonEmpty {
    fn exec(&self, state: &mut SabiniwmState) {
        let direction = match self {
            Self::Next => 1,
            Self::Prev => -1,
        };
        state.inner.view.update_stackset_with(|stackset| {
            let workspaces = &mut stackset.workspaces;
            for d in 1..workspaces.len() {
                let i = workspaces.mod_plus_focused_index(direction * d as isize);
                if !workspaces.as_vec()[i].stack.is_empty() {
                    workspaces.set_focused_index(i);
                    return;
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
pub enum ActionWindowMoveToWorkspace {
    Next,
    Prev,
}

impl ActionFnI for ActionWindowMoveToWorkspace {
    fn exec(&self, state: &mut SabiniwmState) {
        let count = match self {
            Self::Next => 1,
            Self::Prev => -1,
        };
        state.inner.view.update_stackset_with(|stackset| {
            let mut workspaces = stackset.workspaces.as_mut();

            let mut src = workspaces.vec[workspaces.focus].stack.as_mut();
            let window = src.vec.remove(src.focus);
            src.focus = src.focus.min(src.vec.len().saturating_sub(1));
            src.commit();

            workspaces.focus = workspaces.mod_plus_focused_index(count);

            let dst = workspaces.vec[workspaces.focus].stack.as_mut();
            dst.vec.insert(dst.focus, window);
            dst.commit();

            workspaces.commit();
        });
    }
}

#[derive(Debug, Clone)]
pub struct ActionWindowKill {}

impl ActionFnI for ActionWindowKill {
    fn exec(&self, state: &mut SabiniwmState) {
        use smithay::desktop::WindowSurface;

        let Some(window) = state.inner.view.focused_window_mut() else {
            return;
        };

        match window.smithay_window().underlying_surface() {
            WindowSurface::Wayland(w) => w.send_close(),
            WindowSurface::X11(w) => {
                let _ = w.close();
            }
        };
    }
}
