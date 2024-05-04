use crate::util::Id;
use crate::util::{FocusedVec, NonEmptyFocusedVec};
use crate::view::window::Window;

pub struct StackSet {
    pub workspaces: NonEmptyFocusedVec<Workspace>,
}

pub struct Workspace {
    // tag: String,
    pub stack: FocusedVec<Id<Window>>,
}

impl StackSet {
    pub(super) fn new() -> Self {
        let workspace = Workspace {
            stack: FocusedVec::default(),
        };
        Self {
            workspaces: NonEmptyFocusedVec::new(vec![workspace], 0),
        }
    }

    pub fn workspaces(&self) -> &NonEmptyFocusedVec<Workspace> {
        &self.workspaces
    }
}

impl Workspace {
    pub fn stack(&self) -> &FocusedVec<Id<Window>> {
        &self.stack
    }
}
