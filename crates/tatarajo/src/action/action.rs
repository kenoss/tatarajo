use crate::state::TatarajoState;
use crate::view::layout_node::{LayoutMessage, LayoutMessageI};
use dyn_clone::DynClone;

pub trait ActionFnI: std::fmt::Debug + DynClone {
    fn into_action(self) -> Action
    where
        Self: Sized + 'static,
    {
        Action::ActionFn(self.into())
    }
    fn exec(&self, state: &mut TatarajoState);
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
    fn exec(&self, state: &mut TatarajoState) {
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

impl TatarajoState {
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
