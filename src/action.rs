use crate::view::layout_node::{LayoutMessage, LayoutMessageI};
use crate::Sabiniwm;
use dyn_clone::DynClone;

pub trait ActionFnI: std::fmt::Debug + DynClone {
    fn into_action(self) -> Action
    where
        Self: Sized + 'static,
    {
        Action::ActionFn(self.into())
    }
    fn exec(&self, state: &mut Sabiniwm);
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
    fn exec(&self, state: &mut Sabiniwm) {
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

impl Sabiniwm {
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
                self.view.handle_layout_message(message, &mut self.space);
            }
            Action::ActionFn(f) => {
                f.exec(self);
                self.view.layout(&mut self.space);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActionMoveFocus {
    Next,
    Prev,
}

impl ActionFnI for ActionMoveFocus {
    fn exec(&self, state: &mut Sabiniwm) {
        let d = match self {
            Self::Next => 1,
            Self::Prev => -1,
        };
        state.view.focus_next_window(d);
        state.view.reflect_focus(&mut state.space);
    }
}
